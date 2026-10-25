use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use crate::models::{AuditLogEntry, FtsResultItem, TextMetricsResult};
use rusqlite::params;
use std::collections::{HashMap, HashSet};
use tauri::ipc::Channel;
use tauri::State;

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
        let mut syllable_count = 0usize;
        let mut word_freq: HashMap<String, usize> = HashMap::new();
        let mut unique_words = HashSet::new();

        let russian_vowels: HashSet<char> = "аеёиоуыэюяaeiouy".chars().collect();
        let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
        let paragraph_count = lines.len().max(1);

        for word in text.split_whitespace() {
            word_count += 1;
            let normalized: String = word
                .chars()
                .filter(|c| c.is_alphanumeric())
                .flat_map(|c| c.to_lowercase())
                .collect();

            if normalized.chars().count() > 2 {
                unique_words.insert(normalized.clone());
                *word_freq.entry(normalized.clone()).or_insert(0) += 1;
            }

            for ch in normalized.chars() {
                if russian_vowels.contains(&ch) {
                    syllable_count += 1;
                }
            }
        }

        let reading_time_minutes = (word_count as f64 / 190.0).max(0.1);

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

        if sentence_count == 0 {
            sentence_count = 1;
        }

        let sentence_cadence = if !sentence_lengths.is_empty() {
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

        let asl = word_count as f64 / sentence_count as f64;
        let asw = if word_count > 0 {
            syllable_count as f64 / word_count as f64
        } else {
            1.0
        };
        let flesch_reading_ease = (206.835 - (1.3 * asl) - (60.1 * asw)).clamp(0.0, 100.0);

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

        let cliche_dictionary = [
            "в конечном итоге", "имеет место быть", "ни для кого не секрет",
            "на сегодняшний день", "ледяной ужас", "мурашки по спине",
            "холодный пот", "волосы встали дыбом", "кровь застыла в жилах",
            "в глубине души", "луч света в темном", "на вес золота",
        ];

        let lower_text = text.to_lowercase();
        let mut cliche_warnings = Vec::new();
        for cliche in cliche_dictionary {
            if lower_text.contains(cliche) {
                cliche_warnings.push(format!("Штамп: «{}»", cliche));
            }
        }

        let segments = 8;
        let segment_size = (lines.len() / segments).max(1);
        let mut pacing_timeline = Vec::with_capacity(segments);
        for chunk in lines.chunks(segment_size).take(segments) {
            let chunk_text = chunk.join(" ");
            let chunk_words = chunk_text.split_whitespace().count();
            let quotes_in_chunk = chunk_text.chars().filter(|&c| c == '«' || c == '"').count();
            let pace = ((chunk_words as f64 * 0.4) + (quotes_in_chunk as f64 * 8.0)).min(100.0);
            pacing_timeline.push(pace);
        }
        while pacing_timeline.len() < segments {
            pacing_timeline.push(35.0);
        }

        let sentiment_tone = if dialogue_ratio > 45.0 {
            "Динамичный диалог / Саспенс".to_string()
        } else if sentence_cadence > 22.0 {
            "Ритмичная экспрессия / Экшен".to_string()
        } else if flesch_reading_ease > 65.0 {
            "Легкая повествовательная проза".to_string()
        } else {
            "Плотный философский эпос".to_string()
        };

        Ok(TextMetricsResult {
            word_count,
            char_count,
            paragraph_count,
            reading_time_minutes,
            dialogue_ratio,
            sentence_cadence,
            vocabulary_richness,
            flesch_reading_ease,
            sentiment_tone,
            repetition_warnings,
            cliche_warnings,
            pacing_timeline,
        })
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn generate_twist() -> Result<String, AppError> {
    use rand::seq::IndexedRandom;
    let mut rng = rand::rng();

    let catalysts = [
        "Тайный советник ордена", "Преданный оруженосец", "Беглый картограф империи",
        "Торговец забытыми реликвиями", "Странник из зеркального измерения", "Истинный наследник престола",
        "Архивариус запретной крипты", "Опальный алхимик",
    ];

    let twists = [
        "невольно взламывает защитный барьер цитадели",
        "находит доказательства того, что война окончилась век назад",
        "теряет способность лгать перед священным пламенем",
        "обнаруживает в древнем фолианте портрет собственного предка",
        "похищает ключ от крипты забвения ради спасения врага",
        "снимает маску, обнажая метку павшего демиурга",
        "осознает, что хроно-артефакт питается воспоминаниями его близких",
    ];

    let climaxes = [
        "и теперь обратный отсчет мироздания необратим.",
        "что ставит под удар всю исследовательскую экспедицию.",
        "пока на северной границе собираются тени прошлого.",
        "хотя цена этой правды уже оплачена чужой кровью.",
        "вынуждая стороны заключить шаткое перемирие.",
    ];

    let c = catalysts.choose(&mut rng).unwrap_or(&"Герой");
    let t = twists.choose(&mut rng).unwrap_or(&"совершает роковой шаг");
    let cl = climaxes.choose(&mut rng).unwrap_or(&"навсегда.");

    Ok(format!("💡 ПОВОРОТ: {} {}, {}", c, t, cl))
}