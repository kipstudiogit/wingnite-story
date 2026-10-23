use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use crate::lore_guardian::sync_story_embedding;
use crate::models::{
    AuditLogEntry, EnterpriseStoryItem, EntityRelationItem, FtsResultItem, ProjectItem,
    SaveProjectPayload, SaveStoryPayload, TextMetricsResult, TimelineEventItem,
    UpdateStatusPayload, WstoryPackage,
};
use crate::vault::VaultService;
use rusqlite::params;
use std::collections::{HashMap, HashSet};
use tauri::ipc::Channel;
use tauri::Manager;
use tauri::State;

#[tauri::command]
pub async fn fetch_projects(
    state: State<'_, EnterpriseDbPool>,
) -> Result<Vec<ProjectItem>, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let mut stmt = conn.prepare_cached(
            "SELECT id, title, description, created_at, updated_at 
             FROM projects 
             ORDER BY updated_at DESC",
        )?;

        let rows = stmt
            .query_map([], |row| {
                Ok(ProjectItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<ProjectItem>, _>>()?;

        Ok(rows)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn mutate_project(
    state: State<'_, EnterpriseDbPool>,
    payload: SaveProjectPayload,
) -> Result<ProjectItem, AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut conn = db.lock();
        let tx = conn.transaction()?;

        let target_id = payload
            .id
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let now = chrono::Utc::now().timestamp();

        let mut existing_created_at: Option<i64> = None;
        {
            let mut check_stmt = tx.prepare_cached("SELECT created_at FROM projects WHERE id = ?1")?;
            let mut rows = check_stmt.query(params![target_id])?;
            if let Some(row) = rows.next()? {
                existing_created_at = Some(row.get(0)?);
            }
        }

        let created_at = existing_created_at.unwrap_or(now);

        tx.execute(
            "INSERT INTO projects (id, title, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                description = excluded.description,
                updated_at = excluded.updated_at",
            params![target_id, payload.title, payload.description, created_at, now],
        )?;

        tx.commit()?;

        Ok(ProjectItem {
            id: target_id,
            title: payload.title,
            description: payload.description,
            created_at,
            updated_at: now,
        })
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn delete_project(
    state: State<'_, EnterpriseDbPool>,
    id: String,
) -> Result<(), AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut conn = db.lock();
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        tx.commit()?;
        Ok(())
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn fetch_stories(
    state: State<'_, EnterpriseDbPool>,
    project_id: Option<String>,
) -> Result<Vec<EnterpriseStoryItem>, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let mut stories = Vec::new();

        if let Some(ref pid) = project_id {
            if !pid.is_empty() {
                let mut stmt = conn.prepare_cached(
                    "SELECT id, project_id, title, content, category, revision, content_hash, updated_at 
                     FROM stories 
                     WHERE project_id = ?1
                     ORDER BY updated_at DESC",
                )?;
                let rows = stmt.query_map(params![pid], |row| {
                    Ok(EnterpriseStoryItem {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        title: row.get(2)?,
                        content: row.get(3)?,
                        category: row.get(4)?,
                        revision: row.get(5)?,
                        content_hash: row.get(6)?,
                        updated_at: row.get(7)?,
                    })
                })?;
                for row in rows {
                    stories.push(row?);
                }
                return Ok(stories);
            }
        }

        let mut stmt = conn.prepare_cached(
            "SELECT id, project_id, title, content, category, revision, content_hash, updated_at 
             FROM stories 
             ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(EnterpriseStoryItem {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                category: row.get(4)?,
                revision: row.get(5)?,
                content_hash: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        for row in rows {
            stories.push(row?);
        }

        Ok(stories)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn get_stories(
    state: State<'_, EnterpriseDbPool>,
    project_id: Option<String>,
) -> Result<Vec<EnterpriseStoryItem>, AppError> {
    fetch_stories(state, project_id).await
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
        let target_project_id = if payload.project_id.trim().is_empty() {
            "default_project".to_string()
        } else {
            payload.project_id
        };

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
            "INSERT INTO stories (id, project_id, title, content, category, revision, content_hash, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                project_id = excluded.project_id,
                title = excluded.title,
                content = excluded.content,
                category = excluded.category,
                revision = excluded.revision,
                content_hash = excluded.content_hash,
                updated_at = excluded.updated_at",
            params![
                target_id,
                target_project_id,
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
            project_id: target_project_id,
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
        project_id: story.project_id,
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
    project_id: Option<String>,
) -> Result<String, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let target_pid = project_id.unwrap_or_else(|| "default_project".to_string());

        let mut proj_stmt = conn.prepare_cached("SELECT title FROM projects WHERE id = ?1")?;
        let project_title: String = proj_stmt
            .query_row(params![target_pid], |r| r.get(0))
            .unwrap_or_else(|_| "Wingnite Story Edition".to_string());

        let mut stmt = conn.prepare_cached(
            "SELECT id, project_id, title, content, category, revision, content_hash, updated_at 
             FROM stories 
             WHERE project_id = ?1
             ORDER BY updated_at ASC",
        )?;
        let stories = stmt
            .query_map(params![target_pid], |row| {
                Ok(EnterpriseStoryItem {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    category: row.get(4)?,
                    revision: row.get(5)?,
                    content_hash: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<EnterpriseStoryItem>, _>>()?;

        let mut html = format!(
            "<!DOCTYPE html><html lang=\"ru\"><head><meta charset=\"UTF-8\"><title>{}</title><style>body{{font-family:'Literata',Georgia,serif;max-width:760px;margin:40px auto;line-height:1.9;color:#1a1a1a;padding:20px;}}h1,h2{{font-family:'Playfair Display',serif;}}h1{{text-align:center;font-size:3em;margin-top:2em;}}h2{{page-break-before:always;font-size:2em;border-bottom:1px solid #ddd;padding-bottom:10px;margin-top:2em;}}.toc{{background:#f9f9f9;padding:25px;border-radius:10px;margin-bottom:50px;}}p{{text-indent:1.8em;margin-bottom:1em;text-align:justify;}}</style></head><body><h1>{}</h1><div class=\"toc\"><h2>Оглавление</h2><ul>",
            project_title, project_title
        );

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
    project_id: Option<String>,
) -> Result<String, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let target_pid = project_id.unwrap_or_else(|| "default_project".to_string());

        let mut proj_stmt = conn.prepare_cached("SELECT title FROM projects WHERE id = ?1")?;
        let project_title: String = proj_stmt
            .query_row(params![target_pid], |r| r.get(0))
            .unwrap_or_else(|_| "Wingnite Story".to_string());

        let mut stmt = conn.prepare_cached(
            "SELECT id, project_id, title, content, category, revision, content_hash, updated_at 
             FROM stories 
             WHERE project_id = ?1
             ORDER BY updated_at ASC",
        )?;
        let stories = stmt
            .query_map(params![target_pid], |row| {
                Ok(EnterpriseStoryItem {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    category: row.get(4)?,
                    revision: row.get(5)?,
                    content_hash: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<EnterpriseStoryItem>, _>>()?;

        let mut doc = format!("# {}\n*Экспорт произведения K. I. P. Studio*\n\n---\n\n## Оглавление\n", project_title);
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

#[tauri::command]
pub async fn export_project_wstory(
    state: State<'_, EnterpriseDbPool>,
    project_id: String,
) -> Result<String, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();

        let mut proj_stmt = conn.prepare_cached(
            "SELECT id, title, description, created_at, updated_at 
             FROM projects 
             WHERE id = ?1",
        )?;
        let project = proj_stmt
            .query_row(params![project_id], |row| {
                Ok(ProjectItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })
            .map_err(|_| AppError::NotFound(format!("Project '{}' not found", project_id)))?;

        let mut stories_stmt = conn.prepare_cached(
            "SELECT id, project_id, title, content, category, revision, content_hash, updated_at 
             FROM stories 
             WHERE project_id = ?1 
             ORDER BY updated_at ASC",
        )?;
        let stories = stories_stmt
            .query_map(params![project_id], |row| {
                Ok(EnterpriseStoryItem {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    category: row.get(4)?,
                    revision: row.get(5)?,
                    content_hash: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<EnterpriseStoryItem>, _>>()?;

        let story_ids: HashSet<String> = stories.iter().map(|s| s.id.clone()).collect();

        let mut timeline_stmt = conn.prepare_cached(
            "SELECT id, story_id, title, description, act, character_name, temporal_order, intensity 
             FROM timeline_events 
             ORDER BY temporal_order ASC",
        )?;
        let all_timeline = timeline_stmt
            .query_map([], |row| {
                Ok(TimelineEventItem {
                    id: row.get(0)?,
                    story_id: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    act: row.get(4)?,
                    character_name: row.get(5)?,
                    temporal_order: row.get(6)?,
                    intensity: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<TimelineEventItem>, _>>()?;

        let timeline_events: Vec<TimelineEventItem> = all_timeline
            .into_iter()
            .filter(|ev| {
                if let Some(ref sid) = ev.story_id {
                    story_ids.contains(sid)
                } else {
                    true
                }
            })
            .collect();

        let mut rel_stmt = conn.prepare_cached(
            "SELECT id, source_id, target_id, relation_type, description FROM entity_relations",
        )?;
        let all_relations = rel_stmt
            .query_map([], |row| {
                Ok(EntityRelationItem {
                    id: row.get(0)?,
                    source_id: row.get(1)?,
                    target_id: row.get(2)?,
                    relation_type: row.get(3)?,
                    description: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<EntityRelationItem>, _>>()?;

        let entity_relations: Vec<EntityRelationItem> = all_relations
            .into_iter()
            .filter(|r| story_ids.contains(&r.source_id) || story_ids.contains(&r.target_id))
            .collect();

        let mut hash_acc = String::new();
        hash_acc.push_str(&project.id);
        hash_acc.push_str(&project.title);
        for s in &stories {
            hash_acc.push_str(&s.content_hash);
        }
        let checksum = VaultService::calculate_hash(&hash_acc);

        let package = WstoryPackage {
            format: "wstory".to_string(),
            format_version: "1.2.0".to_string(),
            exported_at: chrono::Utc::now().timestamp(),
            generator: "Wingnite Story Enterprise Core 1.2.0".to_string(),
            project,
            stories,
            timeline_events,
            entity_relations,
            checksum,
        };

        let raw = serde_json::to_string_pretty(&package)
            .map_err(|e| AppError::Integrity(e.to_string()))?;
        Ok(raw)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn import_project_wstory(
    state: State<'_, EnterpriseDbPool>,
    package_raw: String,
    operator: String,
) -> Result<ProjectItem, AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let pkg: WstoryPackage = serde_json::from_str(&package_raw)
            .map_err(|e| AppError::Integrity(format!("Invalid .wstory package payload: {}", e)))?;

        if pkg.format != "wstory" {
            return Err(AppError::Integrity(
                "Incompatible package format. Expected 'wstory'".to_string(),
            ));
        }

        let mut conn = db.lock();
        let tx = conn.transaction()?;

        let now = chrono::Utc::now().timestamp();
        let mut target_project_id = pkg.project.id.clone();
        let mut target_title = pkg.project.title.clone();

        let project_exists: bool = {
            let mut check_stmt = tx.prepare_cached("SELECT 1 FROM projects WHERE id = ?1")?;
            let mut rows = check_stmt.query(params![target_project_id])?;
            rows.next()?.is_some()
        };

        let mut id_map: HashMap<String, String> = HashMap::new();

        if project_exists {
            target_project_id = uuid::Uuid::new_v4().to_string();
            target_title = format!("{} (Импорт)", target_title);
        }

        for s in &pkg.stories {
            id_map.insert(s.id.clone(), uuid::Uuid::new_v4().to_string());
        }

        tx.execute(
            "INSERT INTO projects (id, title, description, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                target_project_id,
                target_title,
                pkg.project.description,
                pkg.project.created_at,
                now
            ],
        )?;

        for s in &pkg.stories {
            let story_new_id = id_map.get(&s.id).cloned().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            let hash = VaultService::calculate_hash(&s.content);

            tx.execute(
                "INSERT INTO stories (id, project_id, title, content, category, revision, content_hash, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    story_new_id,
                    target_project_id,
                    s.title,
                    s.content,
                    s.category,
                    s.revision,
                    hash,
                    now
                ],
            )?;

            sync_story_embedding(&tx, &story_new_id, &s.content)?;
        }

        for ev in &pkg.timeline_events {
            let ev_id = uuid::Uuid::new_v4().to_string();
            let mapped_story_id = ev.story_id.as_ref().and_then(|sid| id_map.get(sid).cloned());

            tx.execute(
                "INSERT INTO timeline_events (id, story_id, title, description, act, character_name, temporal_order, intensity)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    ev_id,
                    mapped_story_id,
                    ev.title,
                    ev.description,
                    ev.act,
                    ev.character_name,
                    ev.temporal_order,
                    ev.intensity
                ],
            )?;
        }

        for rel in &pkg.entity_relations {
            let rel_id = uuid::Uuid::new_v4().to_string();
            let mapped_source = id_map.get(&rel.source_id).cloned().unwrap_or_else(|| rel.source_id.clone());
            let mapped_target = id_map.get(&rel.target_id).cloned().unwrap_or_else(|| rel.target_id.clone());

            tx.execute(
                "INSERT INTO entity_relations (id, source_id, target_id, relation_type, description)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    rel_id,
                    mapped_source,
                    mapped_target,
                    rel.relation_type,
                    rel.description
                ],
            )?;
        }

        let audit_id = uuid::Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO audit_ledger (id, entity_id, action, operator, revision, timestamp)
             VALUES (?1, ?2, 'IMPORT_WSTORY_PACKAGE', ?3, 1, ?4)",
            params![audit_id, target_project_id, operator, now],
        )?;

        tx.commit()?;

        Ok(ProjectItem {
            id: target_project_id,
            title: target_title,
            description: pkg.project.description,
            created_at: pkg.project.created_at,
            updated_at: now,
        })
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn check_application_update(
    app: tauri::AppHandle,
) -> Result<UpdateStatusPayload, AppError> {
    #[cfg(desktop)]
    {
        use tauri_plugin_updater::UpdaterExt;
        if let Ok(updater) = app.updater() {
            match updater.check().await {
                Ok(Some(update)) => {
                    return Ok(UpdateStatusPayload {
                        available: true,
                        current_version: update.current_version.clone(),
                        version: Some(update.version.clone()),
                        date: update.date.map(|d| d.to_string()),
                        body: update.body.clone(),
                    });
                }
                Ok(None) => {
                    return Ok(UpdateStatusPayload {
                        available: false,
                        current_version: app.package_info().version.to_string(),
                        version: None,
                        date: None,
                        body: None,
                    });
                }
                Err(e) => {
                    return Err(AppError::Streaming(format!("Update check failed: {}", e)));
                }
            }
        }
    }

    Ok(UpdateStatusPayload {
        available: false,
        current_version: app.package_info().version.to_string(),
        version: None,
        date: None,
        body: None,
    })
}

#[tauri::command]
pub async fn install_application_update(
    app: tauri::AppHandle,
) -> Result<bool, AppError> {
    #[cfg(desktop)]
    {
        use tauri_plugin_updater::UpdaterExt;
        if let Ok(updater) = app.updater() {
            if let Ok(Some(update)) = updater.check().await {
                update
                    .download_and_install(|_chunk_len, _content_len| {}, || {})
                    .await
                    .map_err(|e| AppError::Streaming(format!("Install error: {}", e)))?;

                app.restart();
            }
        }
    }
    Ok(true)
}