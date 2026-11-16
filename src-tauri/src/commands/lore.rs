// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use crate::models::{LoreAnomalyReport, SemanticSearchResult};
use crate::services::lore_engine::{
    bytes_to_vector, compute_embedding, cosine_similarity, evaluate_lore_consistency,
    is_codex_category,
};
use rusqlite::params;
use std::collections::HashMap;
use tauri::State;

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
        let target_pid = project_id
            .filter(|p| !p.trim().is_empty())
            .unwrap_or_else(|| "default_project".to_string());

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

        let mut results = Vec::new();
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

        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
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
    locale: Option<String>,
) -> Result<LoreAnomalyReport, AppError> {
    let db = state.reader.clone();
    let mode = sensitivity.unwrap_or_else(|| "BALANCED".to_string());
    let lang = locale.unwrap_or_else(|| "en".to_string());

    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let target_pid = project_id
            .filter(|p| !p.trim().is_empty())
            .unwrap_or_else(|| "default_project".to_string());

        let mut stmt = conn.prepare_cached(
            "SELECT id, title, category, content
             FROM stories
             WHERE project_id = ?1",
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
            .filter_map(|res| match res {
                Ok(item) if is_codex_category(&item.2) => Some(Ok(item)),
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<Result<Vec<(String, String, String, String)>, _>>()?;

        let mut relations_map: HashMap<String, Vec<(String, String)>> = HashMap::new();
        {
            let mut rel_stmt = conn.prepare_cached(
                "SELECT r.source_id, s.title, r.relation_type
                 FROM entity_relations r
                 JOIN stories src ON src.id = r.source_id
                 JOIN stories s ON s.id = r.target_id
                 WHERE src.project_id = ?1",
            )?;
            let rel_rows = rel_stmt.query_map(params![target_pid], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?;
            for item in rel_rows {
                let (src, tgt_title, rel_type) = item?;
                relations_map.entry(src).or_default().push((tgt_title, rel_type));
            }
        }

        Ok(evaluate_lore_consistency(
            &lore_entities,
            &relations_map,
            &active_content,
            &mode,
            &lang,
        ))
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}