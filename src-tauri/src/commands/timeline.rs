// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use crate::models::{EntityRelationItem, TimelineEventItem};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub async fn fetch_timeline_events(
    state: State<'_, EnterpriseDbPool>,
    project_id: Option<String>,
) -> Result<Vec<TimelineEventItem>, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let target_pid = project_id
            .filter(|p| !p.trim().is_empty())
            .unwrap_or_else(|| "default_project".to_string());

        let mut stmt = conn.prepare_cached(
            "SELECT id, project_id, story_id, title, description, act, character_name, temporal_order, intensity
             FROM timeline_events
             WHERE project_id = ?1
             ORDER BY temporal_order ASC",
        )?;

        let records = stmt
            .query_map(params![target_pid], |row| {
                Ok(TimelineEventItem {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    story_id: row.get(2)?,
                    title: row.get(3)?,
                    description: row.get(4)?,
                    act: row.get(5)?,
                    character_name: row.get(6)?,
                    temporal_order: row.get(7)?,
                    intensity: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<TimelineEventItem>, _>>()?;

        Ok(records)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn mutate_timeline_event(
    state: State<'_, EnterpriseDbPool>,
    mut event: TimelineEventItem,
    operator: String,
) -> Result<TimelineEventItem, AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut conn = db.lock();
        let tx = conn.transaction()?;

        if event.id.is_empty() {
            event.id = uuid::Uuid::new_v4().to_string();
        }
        if event.project_id.trim().is_empty() {
            event.project_id = "default_project".to_string();
        }

        tx.execute(
            "INSERT INTO timeline_events (id, project_id, story_id, title, description, act, character_name, temporal_order, intensity)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                project_id = excluded.project_id,
                story_id = excluded.story_id,
                title = excluded.title,
                description = excluded.description,
                act = excluded.act,
                character_name = excluded.character_name,
                temporal_order = excluded.temporal_order,
                intensity = excluded.intensity",
            params![
                event.id,
                event.project_id,
                event.story_id,
                event.title,
                event.description,
                event.act,
                event.character_name,
                event.temporal_order,
                event.intensity
            ],
        )?;

        let audit_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        tx.execute(
            "INSERT INTO audit_ledger (id, entity_id, action, operator, revision, timestamp)
             VALUES (?1, ?2, 'MUTATE_TIMELINE_EVENT', ?3, 1, ?4)",
            params![audit_id, event.id, operator, now],
        )?;

        tx.commit()?;
        Ok(event)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn delete_timeline_event(
    state: State<'_, EnterpriseDbPool>,
    id: String,
    operator: String,
) -> Result<(), AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut conn = db.lock();
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM timeline_events WHERE id = ?1", params![id])?;

        let audit_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        tx.execute(
            "INSERT INTO audit_ledger (id, entity_id, action, operator, revision, timestamp)
             VALUES (?1, ?2, 'DELETE_TIMELINE_EVENT', ?3, 0, ?4)",
            params![audit_id, id, operator, now],
        )?;

        tx.commit()?;
        Ok(())
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn fetch_entity_relations(
    state: State<'_, EnterpriseDbPool>,
    project_id: Option<String>,
) -> Result<Vec<EntityRelationItem>, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let target_pid = project_id
            .filter(|p| !p.trim().is_empty())
            .unwrap_or_else(|| "default_project".to_string());

        let mut stmt = conn.prepare_cached(
            "SELECT r.id, r.source_id, r.target_id, r.relation_type, r.description
             FROM entity_relations r
             JOIN stories s1 ON s1.id = r.source_id
             JOIN stories s2 ON s2.id = r.target_id
             WHERE s1.project_id = ?1 AND s2.project_id = ?1",
        )?;

        let records = stmt
            .query_map(params![target_pid], |row| {
                Ok(EntityRelationItem {
                    id: row.get(0)?,
                    source_id: row.get(1)?,
                    target_id: row.get(2)?,
                    relation_type: row.get(3)?,
                    description: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<EntityRelationItem>, _>>()?;

        Ok(records)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn mutate_entity_relation(
    state: State<'_, EnterpriseDbPool>,
    mut relation: EntityRelationItem,
    operator: String,
) -> Result<EntityRelationItem, AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut conn = db.lock();
        let tx = conn.transaction()?;

        if relation.id.is_empty() {
            relation.id = uuid::Uuid::new_v4().to_string();
        }

        tx.execute(
            "INSERT INTO entity_relations (id, source_id, target_id, relation_type, description)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                source_id = excluded.source_id,
                target_id = excluded.target_id,
                relation_type = excluded.relation_type,
                description = excluded.description",
            params![
                relation.id,
                relation.source_id,
                relation.target_id,
                relation.relation_type,
                relation.description
            ],
        )?;

        let audit_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        tx.execute(
            "INSERT INTO audit_ledger (id, entity_id, action, operator, revision, timestamp)
             VALUES (?1, ?2, 'MUTATE_RELATION', ?3, 1, ?4)",
            params![audit_id, relation.id, operator, now],
        )?;

        tx.commit()?;
        Ok(relation)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn delete_entity_relation(
    state: State<'_, EnterpriseDbPool>,
    id: String,
    operator: String,
) -> Result<(), AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut conn = db.lock();
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM entity_relations WHERE id = ?1", params![id])?;

        let audit_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        tx.execute(
            "INSERT INTO audit_ledger (id, entity_id, action, operator, revision, timestamp)
             VALUES (?1, ?2, 'DELETE_RELATION', ?3, 0, ?4)",
            params![audit_id, id, operator, now],
        )?;

        tx.commit()?;
        Ok(())
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}