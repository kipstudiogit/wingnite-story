// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use crate::models::{
    SnapshotItem, VaultSealPayload, VaultStatus, VaultUnlockPayload, VaultVerificationResult,
};
use crate::services::crypto_vault::VaultService;
use rusqlite::params;
use tauri::State;

pub fn ensure_project_unlocked(
    conn: &rusqlite::Connection,
    project_id: &str,
) -> Result<(), AppError> {
    let mut stmt =
        conn.prepare_cached("SELECT is_locked FROM project_vaults WHERE project_id = ?1")?;
    let locked: i64 = stmt
        .query_row(params![project_id], |r| r.get(0))
        .unwrap_or(0);
    if locked != 0 {
        return Err(AppError::Vault(
            "Book is locked by Vault. Unlock before editing.".to_string(),
        ));
    }
    Ok(())
}

#[tauri::command]
pub async fn vault_get_status(
    state: State<'_, EnterpriseDbPool>,
    project_id: Option<String>,
) -> Result<VaultStatus, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let target_pid = project_id
            .filter(|p| !p.trim().is_empty())
            .unwrap_or_else(|| "default_project".to_string());

        let mut count_stmt = conn.prepare_cached(
            "SELECT COUNT(*), COALESCE(SUM(revision), 0) FROM stories WHERE project_id = ?1",
        )?;
        let (total_entities_num, total_revisions): (i64, i64) =
            count_stmt.query_row(params![target_pid], |r| Ok((r.get(0)?, r.get(1)?)))?;
        let total_entities = total_entities_num as usize;

        let mut meta_stmt = conn.prepare_cached(
            "SELECT merkle_root, sealed_at, is_locked FROM project_vaults WHERE project_id = ?1",
        )?;
        let meta = meta_stmt
            .query_row(params![target_pid], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })
            .ok();

        if let Some((root, sealed_at, is_locked)) = meta {
            Ok(VaultStatus {
                project_id: target_pid,
                is_sealed: true,
                is_locked: is_locked != 0,
                merkle_root: root,
                total_entities,
                total_revisions,
                last_sealed_at: sealed_at,
            })
        } else {
            Ok(VaultStatus {
                project_id: target_pid,
                is_sealed: false,
                is_locked: false,
                merkle_root: String::new(),
                total_entities,
                total_revisions,
                last_sealed_at: 0,
            })
        }
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn vault_seal_project(
    state: State<'_, EnterpriseDbPool>,
    payload: VaultSealPayload,
) -> Result<VaultStatus, AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let target_pid = if payload.project_id.trim().is_empty() {
            "default_project".to_string()
        } else {
            payload.project_id
        };

        if payload.lock_editing && payload.passphrase.trim().is_empty() {
            return Err(AppError::Vault(
                "Passphrase is required to lock editing".to_string(),
            ));
        }

        let mut conn = db.lock();
        let tx = conn.transaction()?;

        let (total_entities, merkle_root) = {
            let mut stmt = tx.prepare_cached(
                "SELECT content_hash FROM stories WHERE project_id = ?1 ORDER BY id ASC",
            )?;
            let hashes = stmt
                .query_map(params![target_pid], |r| r.get::<_, String>(0))?
                .collect::<Result<Vec<String>, _>>()?;
            let count = hashes.len();
            let root = VaultService::compute_merkle_root(hashes);
            (count, root)
        };

        let salt = uuid::Uuid::new_v4().to_string();
        let passphrase_hash = if payload.passphrase.trim().is_empty() {
            String::new()
        } else {
            VaultService::hash_passphrase(&salt, payload.passphrase.trim())
        };
        let lock_flag: i64 = if payload.lock_editing { 1 } else { 0 };
        let now = chrono::Utc::now().timestamp();

        tx.execute(
            "INSERT INTO project_vaults (project_id, merkle_root, sealed_at, operator, passphrase_salt, passphrase_hash, is_locked)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(project_id) DO UPDATE SET
                merkle_root = excluded.merkle_root,
                sealed_at = excluded.sealed_at,
                operator = excluded.operator,
                passphrase_salt = excluded.passphrase_salt,
                passphrase_hash = excluded.passphrase_hash,
                is_locked = excluded.is_locked",
            params![
                target_pid,
                merkle_root,
                now,
                payload.operator,
                salt,
                passphrase_hash,
                lock_flag
            ],
        )?;

        let total_revisions: i64 = {
            let mut sum_stmt = tx.prepare_cached(
                "SELECT COALESCE(SUM(revision), 0) FROM stories WHERE project_id = ?1",
            )?;
            sum_stmt.query_row(params![target_pid], |r| r.get(0))?
        };

        tx.commit()?;

        Ok(VaultStatus {
            project_id: target_pid,
            is_sealed: true,
            is_locked: payload.lock_editing,
            merkle_root,
            total_entities,
            total_revisions,
            last_sealed_at: now,
        })
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn vault_unlock_project(
    state: State<'_, EnterpriseDbPool>,
    payload: VaultUnlockPayload,
) -> Result<VaultStatus, AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let target_pid = if payload.project_id.trim().is_empty() {
            "default_project".to_string()
        } else {
            payload.project_id
        };

        let mut conn = db.lock();
        let tx = conn.transaction()?;

        let (salt, expected_hash, merkle_root, sealed_at) = {
            let mut stmt = tx.prepare_cached(
                "SELECT passphrase_salt, passphrase_hash, merkle_root, sealed_at
                 FROM project_vaults
                 WHERE project_id = ?1",
            )?;
            stmt.query_row(params![target_pid], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })
            .map_err(|_| AppError::Vault("Vault is not sealed for this book".to_string()))?
        };

        if !VaultService::verify_passphrase(&salt, payload.passphrase.trim(), &expected_hash) {
            return Err(AppError::Vault("Invalid passphrase".to_string()));
        }

        tx.execute(
            "UPDATE project_vaults SET is_locked = 0 WHERE project_id = ?1",
            params![target_pid],
        )?;

        let (total_entities, total_revisions) = {
            let mut count_stmt = tx.prepare_cached(
                "SELECT COUNT(*), COALESCE(SUM(revision), 0) FROM stories WHERE project_id = ?1",
            )?;
            let (cnt, rev): (i64, i64) =
                count_stmt.query_row(params![target_pid], |r| Ok((r.get(0)?, r.get(1)?)))?;
            (cnt as usize, rev)
        };

        tx.commit()?;

        Ok(VaultStatus {
            project_id: target_pid,
            is_sealed: true,
            is_locked: false,
            merkle_root,
            total_entities,
            total_revisions,
            last_sealed_at: sealed_at,
        })
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn vault_verify_merkle_tree(
    state: State<'_, EnterpriseDbPool>,
    project_id: Option<String>,
) -> Result<VaultVerificationResult, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let target_pid = project_id
            .filter(|p| !p.trim().is_empty())
            .unwrap_or_else(|| "default_project".to_string());

        let mut meta_stmt = conn.prepare_cached(
            "SELECT merkle_root FROM project_vaults WHERE project_id = ?1",
        )?;
        let recorded_root = meta_stmt
            .query_row(params![target_pid], |r| r.get::<_, String>(0))
            .unwrap_or_default();

        let mut stmt = conn.prepare_cached(
            "SELECT content_hash FROM stories WHERE project_id = ?1 ORDER BY id ASC",
        )?;
        let hashes = stmt
            .query_map(params![target_pid], |r| r.get::<_, String>(0))?
            .collect::<Result<Vec<String>, _>>()?;

        let verified_nodes = hashes.len();
        let calculated_root = VaultService::compute_merkle_root(hashes);
        let is_valid = !recorded_root.is_empty() && calculated_root == recorded_root;

        Ok(VaultVerificationResult {
            is_valid,
            calculated_root,
            recorded_root,
            verified_nodes,
        })
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn fetch_story_snapshots(
    state: State<'_, EnterpriseDbPool>,
    story_id: String,
) -> Result<Vec<SnapshotItem>, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        if story_id.trim().is_empty() {
            return Ok(Vec::new());
        }
        let conn = db.lock();
        let mut stmt = conn.prepare_cached(
            "SELECT id, story_id, project_id, title, content, word_count, created_at
             FROM story_snapshots
             WHERE story_id = ?1
             ORDER BY created_at DESC
             LIMIT 30",
        )?;

        let records = stmt
            .query_map(params![story_id], |row| {
                Ok(SnapshotItem {
                    id: row.get(0)?,
                    story_id: row.get(1)?,
                    project_id: row.get(2)?,
                    title: row.get(3)?,
                    content: row.get(4)?,
                    word_count: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<SnapshotItem>, _>>()?;

        Ok(records)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn create_story_snapshot(
    state: State<'_, EnterpriseDbPool>,
    story_id: String,
    project_id: String,
    title: String,
    content: String,
) -> Result<SnapshotItem, AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        if story_id.trim().is_empty() {
            return Err(AppError::NotFound("Save scene before creating a snapshot".to_string()));
        }
        let mut conn = db.lock();
        let tx = conn.transaction()?;

        let target_pid = if project_id.trim().is_empty() {
            "default_project".to_string()
        } else {
            project_id
        };

        let id = uuid::Uuid::new_v4().to_string();
        let word_count = content.split_whitespace().count() as i64;
        let now = chrono::Utc::now().timestamp();

        tx.execute(
            "INSERT INTO story_snapshots (id, story_id, project_id, title, content, word_count, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, story_id, target_pid, title, content, word_count, now],
        )?;

        tx.commit()?;

        Ok(SnapshotItem {
            id,
            story_id,
            project_id: target_pid,
            title,
            content,
            word_count,
            created_at: now,
        })
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn delete_story_snapshot(
    state: State<'_, EnterpriseDbPool>,
    id: String,
) -> Result<(), AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        conn.execute("DELETE FROM story_snapshots WHERE id = ?1", params![id])?;
        Ok(())
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}