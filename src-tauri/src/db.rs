use crate::error::AppError;
use rusqlite::{Connection, OpenFlags};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

pub struct EnterpriseDbPool {
    pub reader: Arc<parking_lot::Mutex<Connection>>,
    pub writer: Arc<parking_lot::Mutex<Connection>>,
}

pub fn resolve_secure_storage(app: &AppHandle) -> Result<PathBuf, AppError> {
    let base_dir = app
        .path()
        .app_data_dir()
        .or_else(|_| app.path().document_dir())
        .or_else(|_| app.path().temp_dir())
        .map_err(|e| AppError::Path(e.to_string()))?;

    fs::create_dir_all(&base_dir)?;
    Ok(base_dir.join("enterprise_datastore.db"))
}

pub fn init_database(app: &AppHandle) -> Result<EnterpriseDbPool, AppError> {
    let db_path = resolve_secure_storage(app)?;

    let writer_conn = Connection::open(&db_path)?;
    writer_conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 8000;

         CREATE TABLE IF NOT EXISTS projects (
             id TEXT PRIMARY KEY,
             title TEXT NOT NULL,
             description TEXT NOT NULL,
             created_at INTEGER NOT NULL,
             updated_at INTEGER NOT NULL
         );

         CREATE TABLE IF NOT EXISTS stories (
             id TEXT PRIMARY KEY,
             project_id TEXT NOT NULL DEFAULT 'default_project',
             title TEXT NOT NULL,
             content TEXT NOT NULL,
             category TEXT NOT NULL,
             revision INTEGER NOT NULL DEFAULT 1,
             sort_order INTEGER NOT NULL DEFAULT 0,
             content_hash TEXT NOT NULL,
             updated_at INTEGER NOT NULL,
             FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
         );

         CREATE TABLE IF NOT EXISTS audit_ledger (
             id TEXT PRIMARY KEY,
             entity_id TEXT NOT NULL,
             action TEXT NOT NULL,
             operator TEXT NOT NULL,
             revision INTEGER NOT NULL,
             timestamp INTEGER NOT NULL
         );

         CREATE TABLE IF NOT EXISTS entity_relations (
             id TEXT PRIMARY KEY,
             source_id TEXT NOT NULL,
             target_id TEXT NOT NULL,
             relation_type TEXT NOT NULL,
             description TEXT NOT NULL,
             FOREIGN KEY (source_id) REFERENCES stories(id) ON DELETE CASCADE,
             FOREIGN KEY (target_id) REFERENCES stories(id) ON DELETE CASCADE
         );

         CREATE TABLE IF NOT EXISTS timeline_events (
             id TEXT PRIMARY KEY,
             story_id TEXT,
             title TEXT NOT NULL,
             description TEXT NOT NULL,
             act TEXT NOT NULL,
             character_name TEXT NOT NULL,
             temporal_order INTEGER NOT NULL,
             intensity INTEGER NOT NULL,
             FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE SET NULL
         );

         CREATE TABLE IF NOT EXISTS lore_embeddings (
             story_id TEXT PRIMARY KEY,
             vector BLOB NOT NULL,
             updated_at INTEGER NOT NULL,
             FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE
         );

         CREATE TABLE IF NOT EXISTS vault_metadata (
             id INTEGER PRIMARY KEY CHECK (id = 1),
             merkle_root TEXT NOT NULL,
             sealed_at INTEGER NOT NULL,
             operator TEXT NOT NULL,
             signature TEXT NOT NULL
         );",
    )?;

    let mut has_project_column = false;
    let mut has_sort_order_column = false;
    {
        let mut pragma_stmt = writer_conn.prepare("PRAGMA table_info(stories)")?;
        let mut pragma_rows = pragma_stmt.query([])?;
        while let Some(row) = pragma_rows.next()? {
            let col_name: String = row.get(1)?;
            if col_name == "project_id" {
                has_project_column = true;
            } else if col_name == "sort_order" {
                has_sort_order_column = true;
            }
        }
    }

    if !has_project_column {
        writer_conn.execute(
            "ALTER TABLE stories ADD COLUMN project_id TEXT NOT NULL DEFAULT 'default_project'",
            [],
        )?;
    }

    if !has_sort_order_column {
        writer_conn.execute(
            "ALTER TABLE stories ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }

    writer_conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_stories_project ON stories(project_id);
         CREATE INDEX IF NOT EXISTS idx_stories_order ON stories(project_id, sort_order ASC);
         CREATE INDEX IF NOT EXISTS idx_stories_revision ON stories(id, revision);
         CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_ledger(entity_id);
         CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_ledger(timestamp DESC);
         CREATE INDEX IF NOT EXISTS idx_rel_source ON entity_relations(source_id);
         CREATE INDEX IF NOT EXISTS idx_rel_target ON entity_relations(target_id);
         CREATE INDEX IF NOT EXISTS idx_timeline_order ON timeline_events(temporal_order ASC);

         CREATE VIRTUAL TABLE IF NOT EXISTS stories_fts USING fts5(
             id UNINDEXED,
             title,
             content,
             category,
             tokenize='unicode61'
         );

         CREATE TRIGGER IF NOT EXISTS trg_stories_ai AFTER INSERT ON stories BEGIN
             INSERT INTO stories_fts(id, title, content, category)
             VALUES (new.id, new.title, new.content, new.category);
         END;

         CREATE TRIGGER IF NOT EXISTS trg_stories_ad AFTER DELETE ON stories BEGIN
             DELETE FROM stories_fts WHERE id = old.id;
         END;

         CREATE TRIGGER IF NOT EXISTS trg_stories_au AFTER UPDATE ON stories BEGIN
             DELETE FROM stories_fts WHERE id = old.id;
             INSERT INTO stories_fts(id, title, content, category)
             VALUES (new.id, new.title, new.content, new.category);
         END;",
    )?;

    let now = chrono::Utc::now().timestamp();

    writer_conn.execute(
        "INSERT OR IGNORE INTO projects (id, title, description, created_at, updated_at) 
         VALUES ('default_project', 'Моя книга', 'Основная рукопись', ?1, ?1)",
        rusqlite::params![now],
    )?;

    let _ = writer_conn.execute(
        "UPDATE stories SET project_id = 'default_project' WHERE project_id IS NULL OR project_id = ''",
        [],
    );

    let _ = writer_conn.execute(
        "DELETE FROM stories WHERE id IN ('genesis_1', 'genesis_2', 'genesis_3') 
         OR content_hash IN ('genesis_1', 'genesis_2', 'genesis_3')
         OR title IN ('Глава 1: Тени Цитадели', 'Око Бесконечности', 'Элион Хранитель')",
        [],
    );
    let _ = writer_conn.execute(
        "DELETE FROM timeline_events WHERE title = 'Переход через перевал'",
        [],
    );
    let _ = writer_conn.execute(
        "DELETE FROM entity_relations WHERE description IN ('Связь героя с реликвией', 'Покровительство демиурга')",
        [],
    );

    let reader_conn = Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    reader_conn.execute_batch(
        "PRAGMA busy_timeout = 8000;
         PRAGMA foreign_keys = ON;",
    )?;

    Ok(EnterpriseDbPool {
        reader: Arc::new(parking_lot::Mutex::new(reader_conn)),
        writer: Arc::new(parking_lot::Mutex::new(writer_conn)),
    })
}