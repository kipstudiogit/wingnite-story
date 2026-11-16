// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use crate::models::{
    EnterpriseStoryItem, LspHoverInfo, LspQueryPayload, LspSuggestion, WikiLinkResolvePayload,
    WikiLinkResolveResult,
};
use crate::services::crypto_vault::VaultService;
use crate::services::lore_engine::sync_story_embedding;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub async fn lsp_complete_wikilink(
    state: State<'_, EnterpriseDbPool>,
    payload: LspQueryPayload,
) -> Result<Vec<LspSuggestion>, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let target_pid = if payload.project_id.trim().is_empty() {
            "default_project".to_string()
        } else {
            payload.project_id
        };
        let query_pattern = format!("%{}%", payload.raw_token.trim());
        let prefix_pattern = format!("{}%", payload.raw_token.trim());

        let mut stmt = conn.prepare_cached(
            "SELECT id, title, category, substr(content, 1, 120), revision
             FROM stories
             WHERE project_id = ?1 AND (title LIKE ?2 OR category LIKE ?2)
             ORDER BY CASE WHEN title LIKE ?3 THEN 0 ELSE 1 END, revision DESC
             LIMIT 8",
        )?;

        let rows = stmt
            .query_map(params![target_pid, query_pattern, prefix_pattern], |row| {
                let category: String = row.get(2)?;
                let lower = category.to_lowercase();
                let link_type = if lower.contains("artifact")
                    || lower.contains("артефакт")
                    || lower.contains("artefakt")
                    || lower.contains("artefacto")
                {
                    "artifact".to_string()
                } else if lower.contains("demiurge")
                    || lower.contains("character")
                    || lower.contains("демиург")
                    || lower.contains("персонаж")
                    || lower.contains("charakter")
                    || lower.contains("personaje")
                {
                    "character".to_string()
                } else if lower.contains("sanctuary")
                    || lower.contains("location")
                    || lower.contains("святилище")
                    || lower.contains("локация")
                {
                    "location".to_string()
                } else {
                    "scene".to_string()
                };

                let revision: i64 = row.get(4)?;
                Ok(LspSuggestion {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    category,
                    preview: row.get(3)?,
                    link_type,
                    match_score: revision,
                })
            })?
            .collect::<Result<Vec<LspSuggestion>, _>>()?;

        Ok(rows)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn lsp_hover_entity(
    state: State<'_, EnterpriseDbPool>,
    entity_title: String,
    project_id: Option<String>,
) -> Result<Option<LspHoverInfo>, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let target_pid = project_id
            .filter(|p| !p.trim().is_empty())
            .unwrap_or_else(|| "default_project".to_string());

        let mut stmt = conn.prepare_cached(
            "SELECT id, title, category, substr(content, 1, 240), revision
             FROM stories
             WHERE project_id = ?1 AND title = ?2 COLLATE NOCASE
             LIMIT 1",
        )?;

        let mut rows = stmt.query(params![target_pid, entity_title])?;
        if let Some(row) = rows.next()? {
            let entity_id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let category: String = row.get(2)?;
            let summary: String = row.get(3)?;
            let revision: i64 = row.get(4)?;

            let mut rel_stmt = conn.prepare_cached(
                "SELECT s.title
                 FROM entity_relations r
                 JOIN stories s ON (s.id = r.target_id OR s.id = r.source_id)
                 WHERE (r.source_id = ?1 OR r.target_id = ?1) AND s.id != ?1
                 LIMIT 5",
            )?;

            let connected_entities = rel_stmt
                .query_map(params![entity_id], |r| r.get(0))?
                .collect::<Result<Vec<String>, _>>()?;

            Ok(Some(LspHoverInfo {
                id: entity_id,
                title,
                category,
                summary,
                connected_entities,
                revision,
            }))
        } else {
            Ok(None)
        }
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn lsp_resolve_wikilink(
    state: State<'_, EnterpriseDbPool>,
    payload: WikiLinkResolvePayload,
) -> Result<WikiLinkResolveResult, AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut conn = db.lock();
        let tx = conn.transaction()?;

        let target_pid = if payload.project_id.trim().is_empty() {
            "default_project".to_string()
        } else {
            payload.project_id
        };

        let mut target_id = String::new();
        let mut is_new = false;
        let mut created_story: Option<EnterpriseStoryItem> = None;

        {
            let mut search_stmt = tx.prepare_cached(
                "SELECT id FROM stories WHERE project_id = ?1 AND title = ?2 COLLATE NOCASE LIMIT 1",
            )?;
            let mut rows = search_stmt.query(params![target_pid, payload.target_title])?;
            if let Some(row) = rows.next()? {
                target_id = row.get(0)?;
            }
        }

        let now = chrono::Utc::now().timestamp();

        if target_id.is_empty() {
            target_id = uuid::Uuid::new_v4().to_string();
            is_new = true;
            let placeholder_content = format!("# {}\n\n", payload.target_title);
            let hash = VaultService::calculate_hash(&placeholder_content);

            let max_order: i64 = tx
                .query_row(
                    "SELECT COALESCE(MAX(sort_order), -1) FROM stories WHERE project_id = ?1",
                    params![target_pid],
                    |r| r.get(0),
                )
                .unwrap_or(-1);
            let next_order = max_order + 1;
            let category = "Character / Entity".to_string();

            tx.execute(
                "INSERT INTO stories (id, project_id, title, content, category, revision, sort_order, content_hash, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, ?8)",
                params![
                    target_id,
                    target_pid,
                    payload.target_title,
                    placeholder_content,
                    category,
                    next_order,
                    hash,
                    now
                ],
            )?;

            sync_story_embedding(&tx, &target_id, &placeholder_content)?;

            created_story = Some(EnterpriseStoryItem {
                id: target_id.clone(),
                project_id: target_pid.clone(),
                title: payload.target_title.clone(),
                content: placeholder_content,
                category,
                revision: 1,
                sort_order: next_order,
                content_hash: hash,
                updated_at: now,
            });
        }

        let mut relation_id = String::new();
        if !payload.source_story_id.is_empty() && payload.source_story_id != target_id {
            let source_exists = {
                let mut source_exists_stmt =
                    tx.prepare_cached("SELECT 1 FROM stories WHERE id = ?1")?;
                let mut rows = source_exists_stmt.query(params![payload.source_story_id])?;
                rows.next()?.is_some()
            };

            if source_exists {
                let already_linked = {
                    let mut existing_stmt = tx.prepare_cached(
                        "SELECT id FROM entity_relations WHERE source_id = ?1 AND target_id = ?2 LIMIT 1",
                    )?;
                    let mut rows = existing_stmt.query(params![payload.source_story_id, target_id])?;
                    rows.next()?.map(|r| r.get::<_, String>(0)).transpose()?
                };

                if let Some(existing_id) = already_linked {
                    relation_id = existing_id;
                } else {
                    relation_id = uuid::Uuid::new_v4().to_string();
                    tx.execute(
                        "INSERT INTO entity_relations (id, source_id, target_id, relation_type, description)
                         VALUES (?1, ?2, ?3, 'Semantic Link', 'Auto-linked via [[wikilink]]')",
                        params![relation_id, payload.source_story_id, target_id],
                    )?;
                }
            }
        }

        tx.commit()?;

        Ok(WikiLinkResolveResult {
            target_id,
            target_title: payload.target_title,
            is_new_entity: is_new,
            relation_id,
            created_story,
        })
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}