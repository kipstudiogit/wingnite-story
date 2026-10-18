use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspSuggestion {
    pub id: String,
    pub title: String,
    pub category: String,
    pub preview: String,
    pub link_type: String,
    pub match_score: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspHoverInfo {
    pub id: String,
    pub title: String,
    pub category: String,
    pub summary: String,
    pub connected_entities: Vec<String>,
    pub revision: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspQueryPayload {
    pub story_id: String,
    pub raw_token: String,
    pub trigger_char: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiLinkResolvePayload {
    pub source_story_id: String,
    pub target_title: String,
    pub operator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiLinkResolveResult {
    pub target_id: String,
    pub target_title: String,
    pub is_new_entity: bool,
    pub relation_id: String,
}

#[tauri::command]
pub async fn lsp_complete_wikilink(
    state: State<'_, EnterpriseDbPool>,
    payload: LspQueryPayload,
) -> Result<Vec<LspSuggestion>, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let query_pattern = format!("%{}%", payload.raw_token.trim());

        let mut stmt = conn.prepare_cached(
            "SELECT id, title, category, substr(content, 1, 120), revision
             FROM stories
             WHERE title LIKE ?1 OR category LIKE ?1
             ORDER BY CASE WHEN title LIKE ?2 THEN 0 ELSE 1 END, revision DESC
             LIMIT 8",
        )?;

        let prefix_pattern = format!("{}%", payload.raw_token.trim());
        let rows = stmt
            .query_map(params![query_pattern, prefix_pattern], |row| {
                let category: String = row.get(2)?;
                let link_type = if category.contains("Артефакт") {
                    "artifact".to_string()
                } else if category.contains("Демиург") || category.contains("Персонаж") {
                    "character".to_string()
                } else if category.contains("Святилище") || category.contains("Локация") {
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
) -> Result<Option<LspHoverInfo>, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let mut stmt = conn.prepare_cached(
            "SELECT id, title, category, substr(content, 1, 240), revision
             FROM stories
             WHERE title = ?1 COLLATE NOCASE
             LIMIT 1",
        )?;

        let mut rows = stmt.query(params![entity_title])?;
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

        let mut target_id = String::new();
        let mut is_new = false;

        {
            let mut search_stmt = tx.prepare_cached(
                "SELECT id FROM stories WHERE title = ?1 COLLATE NOCASE LIMIT 1",
            )?;
            let mut rows = search_stmt.query(params![payload.target_title])?;
            if let Some(row) = rows.next()? {
                target_id = row.get(0)?;
            }
        }

        let now = chrono::Utc::now().timestamp();

        if target_id.is_empty() {
            target_id = uuid::Uuid::new_v4().to_string();
            is_new = true;
            let placeholder_content = format!(
                "# {}\n\nАвтоматически материализованная сущность из контекста сцены.",
                payload.target_title
            );

            tx.execute(
                "INSERT INTO stories (id, title, content, category, revision, content_hash, updated_at)
                 VALUES (?1, ?2, ?3, 'Материализованная сущность', 1, 'genesis', ?4)",
                params![target_id, payload.target_title, placeholder_content, now],
            )?;

            let audit_id = uuid::Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO audit_ledger (id, entity_id, action, operator, revision, timestamp)
                 VALUES (?1, ?2, 'LSP_AUTO_CREATE', ?3, 1, ?4)",
                params![audit_id, target_id, payload.operator, now],
            )?;
        }

        let mut relation_id = String::new();
        if !payload.source_story_id.is_empty() {
            let mut source_exists_stmt = tx.prepare_cached("SELECT 1 FROM stories WHERE id = ?1")?;
            let mut rows = source_exists_stmt.query(params![payload.source_story_id])?;
            if rows.next()?.is_some() {
                relation_id = uuid::Uuid::new_v4().to_string();
                tx.execute(
                    "INSERT INTO entity_relations (id, source_id, target_id, relation_type, description)
                     VALUES (?1, ?2, ?3, 'Семантическая ссылка', 'Автоматическое связывание через [[...]]')
                     ON CONFLICT(id) DO NOTHING",
                    params![relation_id, payload.source_story_id, target_id],
                )?;

                let rel_audit_id = uuid::Uuid::new_v4().to_string();
                tx.execute(
                    "INSERT INTO audit_ledger (id, entity_id, action, operator, revision, timestamp)
                     VALUES (?1, ?2, 'LSP_LINK_RELATION', ?3, 1, ?4)",
                    params![rel_audit_id, relation_id, payload.operator, now],
                )?;
            }
        }

        tx.commit()?;

        Ok(WikiLinkResolveResult {
            target_id,
            target_title: payload.target_title,
            is_new_entity: is_new,
            relation_id,
        })
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}