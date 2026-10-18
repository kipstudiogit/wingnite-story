use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use crate::models::{VaultSealPayload, VaultStatus, VaultVerificationResult};
use rusqlite::params;
use sha2::{Digest, Sha256};
use tauri::State;

pub struct VaultService;

impl VaultService {
    pub fn calculate_hash(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn compute_merkle_root(mut hashes: Vec<String>) -> String {
        if hashes.is_empty() {
            return Self::calculate_hash("genesis_empty_vault");
        }
        hashes.sort();
        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in hashes.chunks(2) {
                if chunk.len() == 2 {
                    let combined = format!("{}{}", chunk[0], chunk[1]);
                    next_level.push(Self::calculate_hash(&combined));
                } else {
                    let combined = format!("{}{}", chunk[0], chunk[0]);
                    next_level.push(Self::calculate_hash(&combined));
                }
            }
            hashes = next_level;
        }
        hashes.remove(0)
    }
}

#[tauri::command]
pub async fn vault_get_status(state: State<'_, EnterpriseDbPool>) -> Result<VaultStatus, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let mut count_stmt = conn.prepare_cached("SELECT COUNT(*), COALESCE(SUM(revision), 0) FROM stories")?;
        let (total_entities_num, total_revisions): (i64, i64) =
            count_stmt.query_row([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        let total_entities = total_entities_num as usize;

        let mut meta_stmt = conn.prepare_cached("SELECT merkle_root, sealed_at FROM vault_metadata WHERE id = 1")?;
        let meta = meta_stmt.query_row([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))).ok();

        if let Some((root, sealed_at)) = meta {
            Ok(VaultStatus {
                is_sealed: true,
                merkle_root: root,
                total_entities,
                total_revisions,
                last_sealed_at: sealed_at,
            })
        } else {
            Ok(VaultStatus {
                is_sealed: false,
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
        let mut conn = db.lock();
        let tx = conn.transaction()?;

        let (total_entities, merkle_root) = {
            let mut stmt = tx.prepare_cached("SELECT content_hash FROM stories ORDER BY id ASC")?;
            let hashes = stmt
                .query_map([], |r| r.get::<_, String>(0))?
                .collect::<Result<Vec<String>, _>>()?;
            let count = hashes.len();
            let root = VaultService::compute_merkle_root(hashes);
            (count, root)
        };

        let now = chrono::Utc::now().timestamp();

        tx.execute(
            "INSERT INTO vault_metadata (id, merkle_root, sealed_at, operator, signature)
             VALUES (1, ?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
                merkle_root = excluded.merkle_root,
                sealed_at = excluded.sealed_at,
                operator = excluded.operator,
                signature = excluded.signature",
            params![merkle_root, now, payload.operator, payload.passphrase_signature],
        )?;

        let audit_id = uuid::Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO audit_ledger (id, entity_id, action, operator, revision, timestamp)
             VALUES (?1, 'PROJECT_VAULT', 'SEAL_PROJECT', ?2, 1, ?3)",
            params![audit_id, payload.operator, now],
        )?;

        let total_revisions: i64 = {
            let mut sum_stmt = tx.prepare_cached("SELECT COALESCE(SUM(revision), 0) FROM stories")?;
            sum_stmt.query_row([], |r| r.get(0))?
        };

        tx.commit()?;

        Ok(VaultStatus {
            is_sealed: true,
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
pub async fn vault_verify_merkle_tree(
    state: State<'_, EnterpriseDbPool>,
) -> Result<VaultVerificationResult, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();

        let mut meta_stmt = conn.prepare_cached("SELECT merkle_root FROM vault_metadata WHERE id = 1")?;
        let recorded_root = meta_stmt
            .query_row([], |r| r.get::<_, String>(0))
            .unwrap_or_default();

        let mut stmt = conn.prepare_cached("SELECT content_hash FROM stories ORDER BY id ASC")?;
        let hashes = stmt
            .query_map([], |r| r.get::<_, String>(0))?
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