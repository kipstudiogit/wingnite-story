use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use crate::lore_guardian::sync_story_embedding;
use crate::models::{
    EnterpriseStoryItem, EntityRelationItem, ProjectItem, TimelineEventItem, WstoryPackage,
};
use crate::vault::VaultService;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rusqlite::params;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use tauri::Manager;
use tauri::State;

#[tauri::command]
pub async fn export_project_wstory(
    state: State<'_, EnterpriseDbPool>,
    project_id: String,
) -> Result<String, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();

        let mut proj_stmt = conn.prepare_cached(
            "SELECT id, title, description, created_at, updated_at 
             FROM projects 
             WHERE id = ?1",
        )?;
        let project = proj_stmt
            .query_row(params![project_id], |row| {
                Ok(ProjectItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })
            .map_err(|_| AppError::NotFound(format!("Project '{}' not found", project_id)))?;

        let mut stories_stmt = conn.prepare_cached(
            "SELECT id, project_id, title, content, category, revision, sort_order, content_hash, updated_at 
             FROM stories 
             WHERE project_id = ?1 
             ORDER BY sort_order ASC, updated_at ASC",
        )?;
        let stories = stories_stmt
            .query_map(params![project_id], |row| {
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

        let story_ids: HashSet<String> = stories.iter().map(|s| s.id.clone()).collect();

        let mut timeline_stmt = conn.prepare_cached(
            "SELECT id, story_id, title, description, act, character_name, temporal_order, intensity 
             FROM timeline_events 
             ORDER BY temporal_order ASC",
        )?;
        let all_timeline = timeline_stmt
            .query_map([], |row| {
                Ok(TimelineEventItem {
                    id: row.get(0)?,
                    story_id: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    act: row.get(4)?,
                    character_name: row.get(5)?,
                    temporal_order: row.get(6)?,
                    intensity: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<TimelineEventItem>, _>>()?;

        let timeline_events: Vec<TimelineEventItem> = all_timeline
            .into_iter()
            .filter(|ev| {
                if let Some(ref sid) = ev.story_id {
                    story_ids.contains(sid)
                } else {
                    true
                }
            })
            .collect();

        let mut rel_stmt = conn.prepare_cached(
            "SELECT id, source_id, target_id, relation_type, description FROM entity_relations",
        )?;
        let all_relations = rel_stmt
            .query_map([], |row| {
                Ok(EntityRelationItem {
                    id: row.get(0)?,
                    source_id: row.get(1)?,
                    target_id: row.get(2)?,
                    relation_type: row.get(3)?,
                    description: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<EntityRelationItem>, _>>()?;

        let entity_relations: Vec<EntityRelationItem> = all_relations
            .into_iter()
            .filter(|r| story_ids.contains(&r.source_id) || story_ids.contains(&r.target_id))
            .collect();

        let mut hash_acc = String::new();
        hash_acc.push_str(&project.id);
        hash_acc.push_str(&project.title);
        for s in &stories {
            hash_acc.push_str(&s.content_hash);
        }
        let checksum = VaultService::calculate_hash(&hash_acc);

        let package = WstoryPackage {
            format: "wstory".to_string(),
            format_version: "1.3.0".to_string(),
            exported_at: chrono::Utc::now().timestamp(),
            generator: "Wingnite Story Enterprise Core 1.3.0".to_string(),
            project,
            stories,
            timeline_events,
            entity_relations,
            checksum,
        };

        let json_raw = serde_json::to_string(&package)
            .map_err(|e| AppError::Integrity(e.to_string()))?;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(json_raw.as_bytes())
            .map_err(|e| AppError::Io(e))?;
        let compressed = encoder.finish().map_err(|e| AppError::Io(e))?;

        let b64 = BASE64_STANDARD.encode(&compressed);
        Ok(format!("WSTORY2:{}", b64))
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn import_project_wstory(
    state: State<'_, EnterpriseDbPool>,
    package_raw: String,
    operator: String,
) -> Result<ProjectItem, AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let trimmed = package_raw.trim();

        let json_str = if trimmed.starts_with("WSTORY2:") {
            let b64 = &trimmed["WSTORY2:".len()..];
            let compressed = BASE64_STANDARD
                .decode(b64.trim())
                .map_err(|e| AppError::Integrity(format!("Invalid Base64 in .wstory v2: {}", e)))?;

            let mut decoder = GzDecoder::new(&compressed[..]);
            let mut decompressed = String::new();
            decoder
                .read_to_string(&mut decompressed)
                .map_err(|e| AppError::Integrity(format!("Decompression error in .wstory v2: {}", e)))?;
            decompressed
        } else if trimmed.starts_with('{') {
            trimmed.to_string()
        } else {
            return Err(AppError::Integrity(
                "Unrecognized container format. Expected .wstory v2 or valid JSON".to_string(),
            ));
        };

        let pkg: WstoryPackage = serde_json::from_str(&json_str)
            .map_err(|e| AppError::Integrity(format!("Invalid package payload: {}", e)))?;

        if pkg.format != "wstory" {
            return Err(AppError::Integrity(
                "Incompatible package format. Expected 'wstory'".to_string(),
            ));
        }

        let mut conn = db.lock();
        let tx = conn.transaction()?;

        let now = chrono::Utc::now().timestamp();
        let mut target_project_id = pkg.project.id.clone();
        let mut target_title = pkg.project.title.clone();

        let project_exists: bool = {
            let mut check_stmt = tx.prepare_cached("SELECT 1 FROM projects WHERE id = ?1")?;
            let mut rows = check_stmt.query(params![target_project_id])?;
            rows.next()?.is_some()
        };

        let mut id_map: HashMap<String, String> = HashMap::new();

        if project_exists {
            target_project_id = uuid::Uuid::new_v4().to_string();
            target_title = format!("{} (Импорт)", target_title);
        }

        for s in &pkg.stories {
            id_map.insert(s.id.clone(), uuid::Uuid::new_v4().to_string());
        }

        tx.execute(
            "INSERT INTO projects (id, title, description, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                target_project_id,
                target_title,
                pkg.project.description,
                pkg.project.created_at,
                now
            ],
        )?;

        for s in &pkg.stories {
            let story_new_id = id_map.get(&s.id).cloned().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            let hash = VaultService::calculate_hash(&s.content);

            tx.execute(
                "INSERT INTO stories (id, project_id, title, content, category, revision, sort_order, content_hash, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    story_new_id,
                    target_project_id,
                    s.title,
                    s.content,
                    s.category,
                    s.revision,
                    s.sort_order,
                    hash,
                    now
                ],
            )?;

            sync_story_embedding(&tx, &story_new_id, &s.content)?;
        }

        for ev in &pkg.timeline_events {
            let ev_id = uuid::Uuid::new_v4().to_string();
            let mapped_story_id = ev.story_id.as_ref().and_then(|sid| id_map.get(sid).cloned());

            tx.execute(
                "INSERT INTO timeline_events (id, story_id, title, description, act, character_name, temporal_order, intensity)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    ev_id,
                    mapped_story_id,
                    ev.title,
                    ev.description,
                    ev.act,
                    ev.character_name,
                    ev.temporal_order,
                    ev.intensity
                ],
            )?;
        }

        for rel in &pkg.entity_relations {
            let rel_id = uuid::Uuid::new_v4().to_string();
            let mapped_source = id_map.get(&rel.source_id).cloned().unwrap_or_else(|| rel.source_id.clone());
            let mapped_target = id_map.get(&rel.target_id).cloned().unwrap_or_else(|| rel.target_id.clone());

            tx.execute(
                "INSERT INTO entity_relations (id, source_id, target_id, relation_type, description)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    rel_id,
                    mapped_source,
                    mapped_target,
                    rel.relation_type,
                    rel.description
                ],
            )?;
        }

        let audit_id = uuid::Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO audit_ledger (id, entity_id, action, operator, revision, timestamp)
             VALUES (?1, ?2, 'IMPORT_WSTORY_PACKAGE', ?3, 1, ?4)",
            params![audit_id, target_project_id, operator, now],
        )?;

        tx.commit()?;

        Ok(ProjectItem {
            id: target_project_id,
            title: target_title,
            description: pkg.project.description,
            created_at: pkg.project.created_at,
            updated_at: now,
        })
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn save_export_file(
    app: tauri::AppHandle,
    filename: String,
    content: String,
) -> Result<String, AppError> {
    let target_dir = app
        .path()
        .document_dir()
        .or_else(|_| app.path().download_dir())
        .or_else(|_| app.path().app_data_dir())
        .or_else(|_| app.path().temp_dir())
        .map_err(|e| AppError::Path(e.to_string()))?;

    let target_path = target_dir.join(&filename);
    std::fs::write(&target_path, content.as_bytes())?;

    #[cfg(desktop)]
    {
        let _ = tauri_plugin_opener::reveal_item_in_dir(&target_path);
    }

    Ok(target_path.to_string_lossy().to_string())
}