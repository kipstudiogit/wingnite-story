use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use crate::models::DbStatsPayload;
use tauri::State;

#[tauri::command]
pub async fn app_checkpoint_wal(
    state: State<'_, EnterpriseDbPool>,
) -> Result<(), AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        Ok(())
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn app_vacuum_db(
    state: State<'_, EnterpriseDbPool>,
) -> Result<(), AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        conn.execute_batch("VACUUM; PRAGMA optimize;")?;
        Ok(())
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn app_get_db_stats(
    state: State<'_, EnterpriseDbPool>,
    app: tauri::AppHandle,
) -> Result<DbStatsPayload, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let total_stories = conn.query_row("SELECT COUNT(*) FROM stories", [], |r| r.get::<_, i64>(0))? as usize;
        let total_projects = conn.query_row("SELECT COUNT(*) FROM projects", [], |r| r.get::<_, i64>(0))? as usize;
        let total_relations = conn.query_row("SELECT COUNT(*) FROM entity_relations", [], |r| r.get::<_, i64>(0))? as usize;
        let total_timeline_events = conn.query_row("SELECT COUNT(*) FROM timeline_events", [], |r| r.get::<_, i64>(0))? as usize;

        let db_path = crate::db::resolve_secure_storage(&app)?;
        let metadata = std::fs::metadata(&db_path).ok();
        let db_size_bytes = metadata.map(|m| m.len()).unwrap_or(0);

        Ok(DbStatsPayload {
            total_stories,
            total_projects,
            total_relations,
            total_timeline_events,
            db_size_bytes,
        })
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}