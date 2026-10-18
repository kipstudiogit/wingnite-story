use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use crate::lore_guardian::sync_story_embedding;
use crate::models::{
    AuditLogEntry, EnterpriseStoryItem, FtsResultItem, SaveStoryPayload, TextMetricsResult,
};
use crate::vault::VaultService;
use rusqlite::params;
use std::collections::{HashMap, HashSet};
use tauri::ipc::Channel;
use tauri::Manager;
use tauri::State;

#[tauri::command]
pub async fn fetch_stories(
    state: State<'_, EnterpriseDbPool>,
) -> Result<Vec<EnterpriseStoryItem>, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let mut stmt = conn.prepare_cached(
            "SELECT id, title, content, category, revision, content_hash, updated_at 
             FROM stories 
             ORDER BY updated_at DESC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(EnterpriseStoryItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    category: row.get(3)?,
                    revision: row.get(4)?,
                    content_hash: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<EnterpriseStoryItem>, _>>()?;
        Ok(rows)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn get_stories(
    state: State<'_, EnterpriseDbPool>,
) -> Result<Vec<EnterpriseStoryItem>, AppError> {
    fetch_stories(state).await
}

#[tauri::command]
pub async fn mutate_story(
    state: State<'_, EnterpriseDbPool>,
    payload: SaveStoryPayload,
) -> Result<EnterpriseStoryItem, AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut conn = db.lock();
        let tx = conn.transaction()?;

        let mut existing_revision = 0i64;
        let mut target_id = payload.id.clone();

        if target_id.is_empty() {
            target_id = uuid::Uuid::new_v4().to_string();
        } else {
            let mut check_stmt = tx.prepare_cached("SELECT revision FROM stories WHERE id = ?1")?;
            let mut rows = check_stmt.query(params![target_id])?;
            if let Some(row) = rows.next()? {
                existing_revision = row.get(0)?;
            }
        }

        let next_revision = existing_revision + 1;
        let now = chrono::Utc::now().timestamp();
        let hash = VaultService::calculate_hash(&payload.content);

        tx.execute(
            "INSERT INTO stories (id, title, content, category, revision, content_hash, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                content = excluded.content,
                category = excluded.category,
                revision = excluded.revision,
                content_hash = excluded.content_hash,
                updated_at = excluded.updated_at",
            params![
                target_id,
                payload.title,
                payload.content,
                payload.category,
                next_revision,
                hash,
                now
            ],
        )?;

        let audit_id = uuid::Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO audit_ledger (id, entity_id, action, operator, revision, timestamp)
             VALUES (?1, ?2, 'MUTATE_STORY', ?3, ?4, ?5)",
            params![audit_id, target_id, payload.operator, next_revision, now],
        )?;

        sync_story_embedding(&tx, &target_id, &payload.content)?;

        tx.commit()?;

        Ok(EnterpriseStoryItem {
            id: target_id,
            title: payload.title,
            content: payload.content,
            category: payload.category,
            revision: next_revision,
            content_hash: hash,
            updated_at: now,
        })
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn save_story(
    state: State<'_, EnterpriseDbPool>,
    story: EnterpriseStoryItem,
) -> Result<EnterpriseStoryItem, AppError> {
    let payload = SaveStoryPayload {
        id: story.id,
        title: story.title,
        content: story.content,
        category: story.category,
        expected_revision: 0,
        operator: "Legacy-Bridge".to_string(),
    };
    mutate_story(state, payload).await
}

#[tauri::command]
pub async fn delete_story(
    state: State<'_, EnterpriseDbPool>,
    id: String,
    operator: Option<String>,
) -> Result<(), AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut conn = db.lock();
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM stories WHERE id = ?1", params![id])?;

        let audit_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let op = operator.unwrap_or_else(|| "System".to_string());
        tx.execute(
            "INSERT INTO audit_ledger (id, entity_id, action, operator, revision, timestamp)
             VALUES (?1, ?2, 'DELETE_STORY', ?3, 0, ?4)",
            params![audit_id, id, op, now],
        )?;

        tx.commit()?;
        Ok(())
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn execute_fts_search(
    state: State<'_, EnterpriseDbPool>,
    query: String,
) -> Result<Vec<FtsResultItem>, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let sanitized: String = query
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect();

        let terms: Vec<String> = sanitized
            .split_whitespace()
            .map(|t| format!("\"{}\"*", t.replace('"', "\"\"")))
            .collect();

        if terms.is_empty() {
            return Ok(Vec::new());
        }

        let fts_expr = terms.join(" ");
        let mut stmt = conn.prepare_cached(
            "SELECT id, title, category, snippet(stories_fts, 2, '<b>', '</b>', '...', 15), bm25(stories_fts)
             FROM stories_fts
             WHERE stories_fts MATCH ?1
             ORDER BY bm25(stories_fts)
             LIMIT 50",
        )?;

        let records = stmt
            .query_map(params![fts_expr], |row| {
                Ok(FtsResultItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    category: row.get(2)?,
                    snippet: row.get(3)?,
                    rank: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<FtsResultItem>, _>>()?;

        Ok(records)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn stream_audit_trail(
    state: State<'_, EnterpriseDbPool>,
    channel: Channel<AuditLogEntry>,
) -> Result<(), AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let mut stmt = conn.prepare_cached(
            "SELECT id, entity_id, action, operator, revision, timestamp 
             FROM audit_ledger 
             ORDER BY timestamp DESC 
             LIMIT 150",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(AuditLogEntry {
                id: row.get(0)?,
                entity_id: row.get(1)?,
                action: row.get(2)?,
                operator: row.get(3)?,
                revision: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })?;

        for entry in rows {
            let record = entry?;
            channel
                .send(record)
                .map_err(|e| AppError::Streaming(e.to_string()))?;
        }

        Ok(())
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn analyze_text_metrics(text: String) -> Result<TextMetricsResult, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let char_count = text.chars().count();
        let mut word_count = 0usize;
        let mut word_freq: HashMap<String, usize> = HashMap::new();
        let mut unique_words = HashSet::new();

        for word in text.split_whitespace() {
            word_count += 1;
            let normalized: String = word
                .chars()
                .filter(|c| c.is_alphanumeric())
                .flat_map(|c| c.to_lowercase())
                .collect();

            if normalized.chars().count() > 3 {
                unique_words.insert(normalized.clone());
                *word_freq.entry(normalized).or_insert(0) += 1;
            }
        }

        let reading_time_minutes = (word_count as f64 / 200.0).max(0.1);

        let mut in_quote = false;
        let mut dialogue_chars = 0usize;
        for c in text.chars() {
            if c == '«' || c == '"' {
                in_quote = true;
            } else if c == '»' || (in_quote && c == '"') {
                in_quote = false;
            } else if in_quote {
                dialogue_chars += 1;
            }
        }

        let dialogue_ratio = if char_count > 0 {
            ((dialogue_chars as f64 / char_count as f64) * 100.0).min(100.0)
        } else {
            0.0
        };

        let mut sentence_count = 0usize;
        let mut current_sentence_len = 0usize;
        let mut sentence_lengths = Vec::new();

        for c in text.chars() {
            if c == '.' || c == '!' || c == '?' || c == '…' {
                if current_sentence_len > 0 {
                    sentence_lengths.push(current_sentence_len as f64);
                    sentence_count += 1;
                    current_sentence_len = 0;
                }
            } else if c.is_whitespace() {
                if current_sentence_len > 0 {
                    current_sentence_len += 1;
                }
            } else {
                current_sentence_len += 1;
            }
        }

        let sentence_cadence = if sentence_count > 0 {
            let mean: f64 = sentence_lengths.iter().sum::<f64>() / sentence_count as f64;
            let variance: f64 = sentence_lengths
                .iter()
                .map(|l| (l - mean).powi(2))
                .sum::<f64>()
                / sentence_count as f64;
            variance.sqrt().min(40.0)
        } else {
            0.0
        };

        let vocabulary_richness = if word_count > 0 {
            ((unique_words.len() as f64 / word_count as f64) * 100.0).min(100.0)
        } else {
            0.0
        };

        let mut freq_vec: Vec<(String, usize)> = word_freq.into_iter().collect();
        freq_vec.sort_by(|a, b| b.1.cmp(&a.1));
        let repetition_warnings = freq_vec
            .into_iter()
            .filter(|(_, count)| *count >= 4)
            .take(6)
            .map(|(word, count)| format!("{}: {} раз", word, count))
            .collect();

        Ok(TextMetricsResult {
            word_count,
            char_count,
            reading_time_minutes,
            dialogue_ratio,
            sentence_cadence,
            vocabulary_richness,
            repetition_warnings,
        })
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn compile_book_edition(
    state: State<'_, EnterpriseDbPool>,
) -> Result<String, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let mut stmt = conn.prepare_cached(
            "SELECT id, title, content, category, revision, content_hash, updated_at 
             FROM stories 
             ORDER BY updated_at ASC",
        )?;
        let stories = stmt
            .query_map([], |row| {
                Ok(EnterpriseStoryItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    category: row.get(3)?,
                    revision: row.get(4)?,
                    content_hash: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<EnterpriseStoryItem>, _>>()?;

        let mut html = String::from("<!DOCTYPE html><html lang=\"ru\"><head><meta charset=\"UTF-8\"><title>Wingnite Story Edition</title><style>body{font-family:'Literata',Georgia,serif;max-width:760px;margin:40px auto;line-height:1.9;color:#1a1a1a;padding:20px;}h1,h2{font-family:'Playfair Display',serif;}h1{text-align:center;font-size:3em;margin-top:2em;}h2{page-break-before:always;font-size:2em;border-bottom:1px solid #ddd;padding-bottom:10px;margin-top:2em;}.toc{background:#f9f9f9;padding:25px;border-radius:10px;margin-bottom:50px;}p{text-indent:1.8em;margin-bottom:1em;text-align:justify;}</style></head><body><h1>Wingnite Story</h1><div class=\"toc\"><h2>Оглавление</h2><ul>");
        for s in &stories {
            html.push_str(&format!("<li>[{}] {}</li>", s.category, s.title));
        }
        html.push_str("</ul></div>");
        for s in &stories {
            html.push_str(&format!("<h2>{}</h2><p><em>Акт: {}</em></p>", s.title, s.category));
            for paragraph in s.content.lines() {
                let trimmed = paragraph.trim();
                if !trimmed.is_empty() {
                    html.push_str(&format!("<p>{}</p>", trimmed));
                }
            }
        }
        html.push_str("</body></html>");
        Ok(html)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn generate_twist() -> Result<String, AppError> {
    use rand::seq::IndexedRandom;
    let mut rng = rand::rng();

    let catalysts = [
        "Тайный советник ордена",
        "Преданный оруженосец",
        "Беглый картограф империи",
        "Торговец забытыми артефактами",
        "Странник из зеркального измерения",
        "Истинный наследник трона",
    ];

    let twists = [
        "невольно взламывает защитный барьер цитадели",
        "находит доказательства того, что война окончилась век назад",
        "теряет способность лгать перед лицом инквизиции",
        "обнаруживает в древнем фолианте портрет собственного предка",
        "похищает ключ от крипты забвения ради спасения врага",
        "снимает маску, обнажая метку павшего демиурга",
    ];

    let climaxes = [
        "и теперь обратный отсчет необратим.",
        "что ставит под удар всю экспедицию.",
        "пока на границе собираются тени прошлого.",
        "хотя цена этой правды уже оплачена чужой кровью.",
        "вынуждая стороны заключить шаткое перемирие.",
    ];

    let c = catalysts.choose(&mut rng).unwrap_or(&"Герой");
    let t = twists.choose(&mut rng).unwrap_or(&"совершает ошибку");
    let cl = climaxes.choose(&mut rng).unwrap_or(&"навсегда.");

    Ok(format!("💡 ПОВОРОТ: {} {}, {}", c, t, cl))
}

#[tauri::command]
pub async fn export_manuscript(
    state: State<'_, EnterpriseDbPool>,
) -> Result<String, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let mut stmt = conn.prepare_cached(
            "SELECT id, title, content, category, revision, content_hash, updated_at 
             FROM stories 
             ORDER BY updated_at ASC",
        )?;
        let stories = stmt
            .query_map([], |row| {
                Ok(EnterpriseStoryItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    category: row.get(3)?,
                    revision: row.get(4)?,
                    content_hash: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<EnterpriseStoryItem>, _>>()?;

        let mut doc =
            String::from("# Wingnite Story\n*by K. I. P. Studio*\n\n---\n\n## Оглавление\n");
        for s in &stories {
            doc.push_str(&format!("* [{}] {}\n", s.category, s.title));
        }
        doc.push_str("\n---\n\n");
        for s in &stories {
            doc.push_str(&format!("## {}\n\n*Категория: {}*\n\n", s.title, s.category));
            doc.push_str(&s.content);
            doc.push_str("\n\n---\n\n");
        }
        Ok(doc)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn save_export_file(
    app: tauri::AppHandle,
    filename: String,
    content: String,
) -> Result<String, AppError> {
    let target_dir = app
        .path()
        .download_dir()
        .or_else(|_| app.path().document_dir())
        .or_else(|_| app.path().app_data_dir())
        .map_err(|e| AppError::Path(e.to_string()))?;

    let target_path = target_dir.join(&filename);
    std::fs::write(&target_path, content.as_bytes())?;

    let _ = tauri_plugin_opener::reveal_item_in_dir(&target_path);

    Ok(target_path.to_string_lossy().to_string())
}