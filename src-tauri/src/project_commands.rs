use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use crate::models::{ProjectItem, SaveProjectPayload};
use rusqlite::params;
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