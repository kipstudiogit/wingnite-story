// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

use crate::commands::vault::ensure_project_unlocked;
use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use crate::models::{EnterpriseStoryItem, FtsResultItem, SaveStoryPayload, StoryOrderUpdate};
use crate::services::crypto_vault::VaultService;
use crate::services::lore_engine::sync_story_embedding;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub async fn fetch_stories(
    state: State<'_, EnterpriseDbPool>,
    project_id: Option<String>,
) -> Result<Vec<EnterpriseStoryItem>, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let target_pid = project_id
            .filter(|p| !p.trim().is_empty())
            .unwrap_or_else(|| "default_project".to_string());

        let mut stmt = conn.prepare_cached(
            "SELECT id, project_id, title, content, category, revision, sort_order, content_hash, updated_at
             FROM stories
             WHERE project_id = ?1
             ORDER BY sort_order ASC, updated_at DESC",
        )?;

        let records = stmt
            .query_map(params![target_pid], |row| {
                Ok(EnterpriseStoryItem {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    category: row.get(4)?,
                    revision: row.get(5)?,
                    sort_order: row.get(6)?,
                    content_hash: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<EnterpriseStoryItem>, _>>()?;

        Ok(records)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
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

        let target_project_id = if payload.project_id.trim().is_empty() {
            "default_project".to_string()
        } else {
            payload.project_id
        };

        ensure_project_unlocked(&tx, &target_project_id)?;

        let mut existing_revision = 0i64;
        let mut existing_order = 0i64;
        let mut target_id = payload.id.clone();

        if target_id.is_empty() {
            target_id = uuid::Uuid::new_v4().to_string();
            let max_order: i64 = tx
                .query_row(
                    "SELECT COALESCE(MAX(sort_order), -1) FROM stories WHERE project_id = ?1",
                    params![target_project_id],
                    |r| r.get(0),
                )
                .unwrap_or(-1);
            existing_order = max_order + 1;
        } else {
            let mut check_stmt =
                tx.prepare_cached("SELECT revision, sort_order FROM stories WHERE id = ?1")?;
            let mut rows = check_stmt.query(params![target_id])?;
            if let Some(row) = rows.next()? {
                existing_revision = row.get(0)?;
                existing_order = row.get(1)?;
            }
        }

        let next_revision = existing_revision + 1;
        let final_order = payload.sort_order.unwrap_or(existing_order);
        let now = chrono::Utc::now().timestamp();
        let hash = VaultService::calculate_hash(&payload.content);

        tx.execute(
            "INSERT INTO stories (id, project_id, title, content, category, revision, sort_order, content_hash, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                project_id = excluded.project_id,
                title = excluded.title,
                content = excluded.content,
                category = excluded.category,
                revision = excluded.revision,
                sort_order = excluded.sort_order,
                content_hash = excluded.content_hash,
                updated_at = excluded.updated_at",
            params![
                target_id,
                target_project_id,
                payload.title,
                payload.content,
                payload.category,
                next_revision,
                final_order,
                hash,
                now
            ],
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
            sort_order: final_order,
            content_hash: hash,
            updated_at: now,
        })
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
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

        let project_id: Option<String> = {
            let mut stmt = tx.prepare_cached("SELECT project_id FROM stories WHERE id = ?1")?;
            stmt.query_row(params![id], |r| r.get(0)).ok()
        };
        if let Some(ref pid) = project_id {
            ensure_project_unlocked(&tx, pid)?;
        }

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
pub async fn reorder_stories(
    state: State<'_, EnterpriseDbPool>,
    updates: Vec<StoryOrderUpdate>,
) -> Result<(), AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut conn = db.lock();
        let tx = conn.transaction()?;

        if let Some(first) = updates.first() {
            let project_id: Option<String> = {
                let mut stmt =
                    tx.prepare_cached("SELECT project_id FROM stories WHERE id = ?1")?;
                stmt.query_row(params![first.id], |r| r.get(0)).ok()
            };
            if let Some(ref pid) = project_id {
                ensure_project_unlocked(&tx, pid)?;
            }
        }

        for item in updates {
            if let Some(cat) = item.category {
                tx.execute(
                    "UPDATE stories SET sort_order = ?1, category = ?2 WHERE id = ?3",
                    params![item.sort_order, cat, item.id],
                )?;
            } else {
                tx.execute(
                    "UPDATE stories SET sort_order = ?1 WHERE id = ?2",
                    params![item.sort_order, item.id],
                )?;
            }
        }

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
    project_id: Option<String>,
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
        let target_pid = project_id
            .filter(|p| !p.trim().is_empty())
            .unwrap_or_else(|| "default_project".to_string());

        let mut stmt = conn.prepare_cached(
            "SELECT stories_fts.id, stories_fts.title, stories_fts.category, snippet(stories_fts, 2, '<b>', '</b>', '...', 15), bm25(stories_fts)
             FROM stories_fts
             JOIN stories ON stories.id = stories_fts.id
             WHERE stories_fts MATCH ?1 AND stories.project_id = ?2
             ORDER BY bm25(stories_fts)
             LIMIT 50",
        )?;

        let records = stmt
            .query_map(params![fts_expr, target_pid], |row| {
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