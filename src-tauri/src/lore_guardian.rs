use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

pub const VECTOR_DIM: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticSearchResult {
    pub id: String,
    pub title: String,
    pub category: String,
    pub similarity: f32,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreAnomalyItem {
    pub id: String,
    pub entity_title: String,
    pub conflict_type: String,
    pub claim_in_text: String,
    pub original_lore_fact: String,
    pub severity: String,
    pub suggested_fix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreAnomalyReport {
    pub anomalies: Vec<LoreAnomalyItem>,
    pub verified_claims_count: usize,
    pub consistency_score: f32,
}

pub fn compute_embedding(text: &str) -> [f32; VECTOR_DIM] {
    let mut vec = [0.0f32; VECTOR_DIM];
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return vec;
    }

    for word in &words {
        let clean: String = word
            .chars()
            .filter(|c| c.is_alphanumeric())
            .flat_map(|c| c.to_lowercase())
            .collect();

        if clean.is_empty() {
            continue;
        }

        let mut h = 2166136261u32;
        for b in clean.bytes() {
            h ^= b as u32;
            h = h.wrapping_mul(16777619);
        }
        let idx = (h as usize) % VECTOR_DIM;
        vec[idx] += 1.0;

        let char_vec: Vec<char> = clean.chars().collect();
        if char_vec.len() >= 3 {
            for window in char_vec.windows(3) {
                let mut tri_h = 2166136261u32;
                for ch in window {
                    tri_h ^= *ch as u32;
                    tri_h = tri_h.wrapping_mul(16777619);
                }
                let tri_idx = (tri_h as usize) % VECTOR_DIM;
                vec[tri_idx] += 0.4;
            }
        }
    }

    let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in vec.iter_mut() {
            *v /= norm;
        }
    }
    vec
}

pub fn cosine_similarity(a: &[f32; VECTOR_DIM], b: &[f32; VECTOR_DIM]) -> f32 {
    let mut dot = 0.0f32;
    for i in 0..VECTOR_DIM {
        dot += a[i] * b[i];
    }
    dot
}

pub fn vector_to_bytes(v: &[f32; VECTOR_DIM]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(VECTOR_DIM * 4);
    for num in v {
        bytes.extend_from_slice(&num.to_le_bytes());
    }
    bytes
}

pub fn bytes_to_vector(b: &[u8]) -> Option<[f32; VECTOR_DIM]> {
    if b.len() != VECTOR_DIM * 4 {
        return None;
    }
    let mut v = [0.0f32; VECTOR_DIM];
    for (i, chunk) in b.chunks_exact(4).enumerate() {
        let arr = [chunk[0], chunk[1], chunk[2], chunk[3]];
        v[i] = f32::from_le_bytes(arr);
    }
    Some(v)
}

pub fn sync_story_embedding(
    conn: &rusqlite::Connection,
    story_id: &str,
    content: &str,
) -> Result<(), rusqlite::Error> {
    let embedding = compute_embedding(content);
    let bytes = vector_to_bytes(&embedding);
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO lore_embeddings (story_id, vector, updated_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(story_id) DO UPDATE SET
            vector = excluded.vector,
            updated_at = excluded.updated_at",
        params![story_id, bytes, now],
    )?;
    Ok(())
}

#[tauri::command]
pub async fn lore_clear_embeddings_cache(
    state: State<'_, EnterpriseDbPool>,
) -> Result<(), AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        conn.execute("DELETE FROM lore_embeddings", [])?;
        Ok(())
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn lore_index_story(
    state: State<'_, EnterpriseDbPool>,
    story_id: String,
    content: String,
) -> Result<(), AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut conn = db.lock();
        let tx = conn.transaction()?;
        sync_story_embedding(&tx, &story_id, &content)?;
        tx.commit()?;
        Ok(())
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn lore_semantic_search(
    state: State<'_, EnterpriseDbPool>,
    query: String,
    project_id: Option<String>,
    limit: usize,
) -> Result<Vec<SemanticSearchResult>, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let query_vec = compute_embedding(&query);

        let mut results = Vec::new();
        let target_pid = project_id.unwrap_or_else(|| "default_project".to_string());

        let mut stmt = conn.prepare_cached(
            "SELECT s.id, s.title, s.category, substr(s.content, 1, 160), e.vector
             FROM stories s
             JOIN lore_embeddings e ON s.id = e.story_id
             WHERE s.project_id = ?1",
        )?;

        let rows = stmt.query_map(params![target_pid], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let category: String = row.get(2)?;
            let snippet: String = row.get(3)?;
            let raw_bytes: Vec<u8> = row.get(4)?;
            Ok((id, title, category, snippet, raw_bytes))
        })?;

        for item in rows {
            let (id, title, category, snippet, raw_bytes) = item?;
            if let Some(target_vec) = bytes_to_vector(&raw_bytes) {
                let sim = cosine_similarity(&query_vec, &target_vec);
                if sim > 0.12 {
                    results.push(SemanticSearchResult {
                        id,
                        title,
                        category,
                        similarity: sim,
                        snippet,
                    });
                }
            }
        }

        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit.max(1));

        Ok(results)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn lore_verify_consistency(
    state: State<'_, EnterpriseDbPool>,
    active_content: String,
    project_id: Option<String>,
    sensitivity: Option<String>,
) -> Result<LoreAnomalyReport, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let target_pid = project_id.unwrap_or_else(|| "default_project".to_string());
        let mode = sensitivity.unwrap_or_else(|| "BALANCED".to_string());

        let mut stmt = conn.prepare_cached(
            "SELECT id, title, category, content
             FROM stories
             WHERE project_id = ?1 AND category IN ('Персонаж', 'Божественный Артефакт', 'Демиург / Сущность', 'Святилище Сил')",
        )?;

        let lore_entities = stmt
            .query_map(params![target_pid], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?
            .collect::<Result<Vec<(String, String, String, String)>, _>>()?;

        let mut anomalies = Vec::new();
        let mut verified_count = 0usize;
        let lower_text = active_content.to_lowercase();

        let mut relations_map: HashMap<String, Vec<(String, String)>> = HashMap::new();
        {
            let mut rel_stmt = conn.prepare_cached(
                "SELECT r.source_id, s.title, r.relation_type 
                 FROM entity_relations r
                 JOIN stories s ON s.id = r.target_id",
            )?;
            let rel_rows = rel_stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
            })?;
            for item in rel_rows {
                let (src, tgt_title, rel_type) = item?;
                relations_map.entry(src).or_default().push((tgt_title, rel_type));
            }
        }

        for (id, title, category, lore_text) in lore_entities {
            let title_lower = title.to_lowercase();
            if !lower_text.contains(&title_lower) {
                continue;
            }

            verified_count += 1;
            let lore_lower = lore_text.to_lowercase();

            let ontological_rules = [
                ("смертен", "бессмерт", "Онтологический парадокс бессмертия", "CRITICAL", "Устранить упоминание смертности или зафиксировать утрату божественного статуса"),
                ("погиб", "жив", "Хронологическое нарушение гибели", "CRITICAL", "Персонаж зафиксирован как живой в каноне. Проверьте хронологию сцены"),
                ("уничтожен", "неуязвим", "Нарушение целостности реликвии", "CRITICAL", "Артефакт зафиксирован как неуязвимый"),
                ("предатель", "верный", "Искажение мотивов альянса", "MAJOR", "В каноне зафиксирована верность фракции"),
                ("человек", "демиург", "Искажение божественного ранга", "MAJOR", "Сущность обладает рангом Демиурга"),
                ("бессилен", "всемогущ", "Конфликт пределов могущества", "MINOR", "Уточните источник подавления сил"),
            ];

            for (text_needle, lore_needle, conflict_desc, severity, fix) in ontological_rules {
                if mode == "PERMISSIVE" && severity != "CRITICAL" {
                    continue;
                }
                if mode == "BALANCED" && severity == "MINOR" {
                    continue;
                }

                let pos = lower_text.find(&title_lower);
                if let Some(p) = pos {
                    let window_start = p.saturating_sub(120);
                    let window_end = (p + title_lower.len() + 120).min(lower_text.len());
                    let context_window = &lower_text[window_start..window_end];

                    let text_has_claim = context_window.contains(text_needle);
                    let lore_has_fact = lore_lower.contains(lore_needle);

                    if text_has_claim && lore_has_fact {
                        anomalies.push(LoreAnomalyItem {
                            id: uuid::Uuid::new_v4().to_string(),
                            entity_title: title.clone(),
                            conflict_type: conflict_desc.to_string(),
                            claim_in_text: format!("В сцене: «{}»", text_needle),
                            original_lore_fact: format!("В каноне [{}] «{}»", category, lore_needle),
                            severity: severity.to_string(),
                            suggested_fix: fix.to_string(),
                        });
                    }
                }
            }

            if mode != "PERMISSIVE" {
                if let Some(rels) = relations_map.get(&id) {
                    for (target_title, rel_type) in rels {
                        let tgt_lower = target_title.to_lowercase();
                        if lower_text.contains(&tgt_lower) {
                            let pos_a = lower_text.find(&title_lower).unwrap_or(0);
                            let pos_b = lower_text.find(&tgt_lower).unwrap_or(0);
                            let dist = (pos_a as isize - pos_b as isize).abs();

                            if dist < 300 && rel_type.contains("Вражда") && (lower_text.contains("союзник") || lower_text.contains("друг")) {
                                anomalies.push(LoreAnomalyItem {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    entity_title: format!("{} ↔ {}", title, target_title),
                                    conflict_type: "Нарушение статуса вражды".to_string(),
                                    claim_in_text: "В тексте персонажи выступают в мирном альянсе".to_string(),
                                    original_lore_fact: "В графе зафиксирована «Вражда»".to_string(),
                                    severity: "MAJOR".to_string(),
                                    suggested_fix: "Обоснуйте перемирие или измените связь в Кодексе".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        let penalty = (anomalies.len() as f32 * 14.0).min(100.0);
        let consistency_score = (100.0 - penalty).max(0.0);

        Ok(LoreAnomalyReport {
            anomalies,
            verified_claims_count: verified_count,
            consistency_score,
        })
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}