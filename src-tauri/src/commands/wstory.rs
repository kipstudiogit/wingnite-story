// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use crate::models::{
    EnterpriseStoryItem, EntityRelationItem, ProjectItem, TimelineEventItem, WstoryPackage,
};
use crate::services::crypto_vault::VaultService;
use crate::services::lore_engine::sync_story_embedding;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rusqlite::params;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::path::Path;
use tauri::State;
use tauri_plugin_dialog::{DialogExt, FilePath};
use tauri_plugin_fs::{FsExt, OpenOptions};

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
            "SELECT id, project_id, story_id, title, description, act, character_name, temporal_order, intensity
             FROM timeline_events
             WHERE project_id = ?1
             ORDER BY temporal_order ASC",
        )?;
        let timeline_events = timeline_stmt
            .query_map(params![project_id], |row| {
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
            .filter(|r| story_ids.contains(&r.source_id) && story_ids.contains(&r.target_id))
            .collect();

        let checksum = VaultService::compute_wstory_checksum(&project, &stories);

        let package = WstoryPackage {
            format: "wstory".to_string(),
            format_version: "1.4.0001".to_string(),
            exported_at: chrono::Utc::now().timestamp(),
            generator: "Wingnite Story Core 1.4.0001".to_string(),
            project,
            stories,
            timeline_events,
            entity_relations,
            checksum,
        };

        let json_raw =
            serde_json::to_string(&package).map_err(|e| AppError::Integrity(e.to_string()))?;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(json_raw.as_bytes())?;
        let compressed = encoder.finish()?;

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

        let json_str = if let Some(b64) = trimmed.strip_prefix("WSTORY2:") {
            let compressed = BASE64_STANDARD
                .decode(b64.trim())
                .map_err(|e| AppError::Integrity(format!("Invalid Base64 in .wstory v2: {}", e)))?;

            let mut decoder = GzDecoder::new(&compressed[..]);
            let mut decompressed = String::new();
            decoder.read_to_string(&mut decompressed).map_err(|e| {
                AppError::Integrity(format!("Decompression error in .wstory v2: {}", e))
            })?;
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

        let expected_checksum = VaultService::compute_wstory_checksum(&pkg.project, &pkg.stories);
        if !pkg.checksum.is_empty() && pkg.checksum != expected_checksum {
            return Err(AppError::Integrity(
                "Container checksum verification failed".to_string(),
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

        if project_exists {
            target_project_id = uuid::Uuid::new_v4().to_string();
            target_title = format!("{} (Import)", target_title);
        }

        let mut id_map: HashMap<String, String> = HashMap::new();
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
            let story_new_id = id_map
                .get(&s.id)
                .cloned()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
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
                "INSERT INTO timeline_events (id, project_id, story_id, title, description, act, character_name, temporal_order, intensity)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    ev_id,
                    target_project_id,
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
            if let (Some(mapped_source), Some(mapped_target)) =
                (id_map.get(&rel.source_id), id_map.get(&rel.target_id))
            {
                let rel_id = uuid::Uuid::new_v4().to_string();
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
) -> Result<Option<String>, AppError> {
    let safe_name = Path::new(&filename)
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|n| !n.is_empty())
        .map(|n| n.to_string())
        .ok_or_else(|| AppError::Integrity("Invalid export filename".to_string()))?;

    tauri::async_runtime::spawn_blocking(move || {
        #[cfg(desktop)]
        let dialog = {
            use tauri::Manager;
            let mut d = app.dialog().file().set_file_name(&safe_name);
            if let Some(dir) = app
                .path()
                .document_dir()
                .ok()
                .or_else(|| app.path().download_dir().ok())
            {
                d = d.set_directory(dir);
            }
            if let Some(ext) = Path::new(&safe_name).extension().and_then(|e| e.to_str()) {
                d = d.add_filter(ext.to_uppercase(), &[ext]);
            }
            d
        };

        #[cfg(not(desktop))]
        let dialog = app.dialog().file().set_file_name(&safe_name);

        let Some(selected_path) = dialog.blocking_save_file() else {
            return Ok(None);
        };

        match selected_path {
            FilePath::Path(ref p) => {
                std::fs::write(p, content.as_bytes())?;
                #[cfg(desktop)]
                {
                    let _ = tauri_plugin_opener::reveal_item_in_dir(p);
                }
                Ok(Some(p.to_string_lossy().to_string()))
            }
            FilePath::Url(ref url) => {
                let mut opts = OpenOptions::new();
                opts.write(true);
                let mut file = app.fs().open(selected_path.clone(), opts).map_err(|e| {
                    AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                })?;
                file.write_all(content.as_bytes())?;
                file.sync_all()?;
                Ok(Some(url.to_string()))
            }
        }
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}