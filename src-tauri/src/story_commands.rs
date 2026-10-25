use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use crate::lore_guardian::sync_story_embedding;
use crate::models::{EnterpriseStoryItem, SaveStoryPayload, StoryOrderUpdate};
use crate::vault::VaultService;
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
        let mut stories = Vec::new();

        if let Some(ref pid) = project_id {
            if !pid.is_empty() {
                let mut stmt = conn.prepare_cached(
                    "SELECT id, project_id, title, content, category, revision, sort_order, content_hash, updated_at 
                     FROM stories 
                     WHERE project_id = ?1
                     ORDER BY sort_order ASC, updated_at DESC",
                )?;
                let rows = stmt.query_map(params![pid], |row| {
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
                })?;
                for row in rows {
                    stories.push(row?);
                }
                return Ok(stories);
            }
        }

        let mut stmt = conn.prepare_cached(
            "SELECT id, project_id, title, content, category, revision, sort_order, content_hash, updated_at 
             FROM stories 
             ORDER BY sort_order ASC, updated_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
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
        let mut existing_order = 0i64;
        let mut target_id = payload.id.clone();
        let target_project_id = if payload.project_id.trim().is_empty() {
            "default_project".to_string()
        } else {
            payload.project_id
        };

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
            let mut check_stmt = tx.prepare_cached("SELECT revision, sort_order FROM stories WHERE id = ?1")?;
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
            sort_order: final_order,
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
        sort_order: Some(story.sort_order),
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
pub async fn reorder_stories(
    state: State<'_, EnterpriseDbPool>,
    updates: Vec<StoryOrderUpdate>,
) -> Result<(), AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut conn = db.lock();
        let tx = conn.transaction()?;

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