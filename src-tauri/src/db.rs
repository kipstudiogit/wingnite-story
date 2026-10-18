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

         CREATE TABLE IF NOT EXISTS stories (
             id TEXT PRIMARY KEY,
             title TEXT NOT NULL,
             content TEXT NOT NULL,
             category TEXT NOT NULL,
             revision INTEGER NOT NULL DEFAULT 1,
             content_hash TEXT NOT NULL,
             updated_at INTEGER NOT NULL
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
         );

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

    let count: i64 = writer_conn
        .query_row("SELECT COUNT(*) FROM stories", [], |r| r.get(0))
        .unwrap_or(0);

    if count == 0 {
        let now = chrono::Utc::now().timestamp();
        let genesis_id = uuid::Uuid::new_v4().to_string();
        let artifact_id = uuid::Uuid::new_v4().to_string();
        let entity_id = uuid::Uuid::new_v4().to_string();

        let genesis_content = "# Глава 1: Тени Цитадели\n\nСеверный ветер трепал штандарты над каменными бастионами цитадели, когда караван беглецов перевалил через заснеженный перевал...\n\n---\n\n> «История никогда не начинается в тишине. Ей всегда предшествует раскат грома.»\n\nОн коснулся рукояти старинного клинка, чувствуя холод древней стали [[Око Бесконечности]]. В небе над цитаделью пробудился @Элион.";
        let artifact_content = "### Реликвия Высшего Порядка\n* **Класс:** Первородный Артефакт\n* **Владелец:** Павший Демиург Самаэль\n* **Истинное свойство:** Позволяет созерцать параллельные временные ветви ценой увядания воспоминаний\n* **Печать ограничения:** Не может быть активирован смертным без жертвы кровью рода\n* **История обретения:** Извлечен со дна Бездны на исходе Второй Эпохи...";
        let demiurge_content = "### Досье Божественной Сущности\n* **Истинный ранг:** Архитектор Времени\n* **Культ почитания:** Орден Золотого Веретена\n* **Облик:** Аморфное созвездие из вращающихся серебряных колец\n* **Уязвимость:** Искажение изначального ритма ткани мироздания\n* **Отношение к героям:** Бесстрастное наблюдение за фатальным выбором";

        let _ = writer_conn.execute(
            "INSERT INTO stories (id, title, content, category, revision, content_hash, updated_at) VALUES (?1, 'Глава 1: Тени Цитадели', ?2, 'Акт I', 1, 'genesis_1', ?3)",
            rusqlite::params![genesis_id, genesis_content, now],
        );
        let _ = writer_conn.execute(
            "INSERT INTO stories (id, title, content, category, revision, content_hash, updated_at) VALUES (?1, 'Око Бесконечности', ?2, 'Божественный Артефакт', 1, 'genesis_2', ?3 + 1)",
            rusqlite::params![artifact_id, artifact_content, now],
        );
        let _ = writer_conn.execute(
            "INSERT INTO stories (id, title, content, category, revision, content_hash, updated_at) VALUES (?1, 'Элион Хранитель', ?2, 'Демиург / Сущность', 1, 'genesis_3', ?3 + 2)",
            rusqlite::params![entity_id, demiurge_content, now],
        );

        let _ = writer_conn.execute(
            "INSERT INTO entity_relations (id, source_id, target_id, relation_type, description) VALUES (?1, ?2, ?3, 'Владение', 'Связь героя с реликвией')",
            rusqlite::params![uuid::Uuid::new_v4().to_string(), genesis_id, artifact_id],
        );
        let _ = writer_conn.execute(
            "INSERT INTO entity_relations (id, source_id, target_id, relation_type, description) VALUES (?1, ?2, ?3, 'Иерархия', 'Покровительство демиурга')",
            rusqlite::params![uuid::Uuid::new_v4().to_string(), genesis_id, entity_id],
        );

        let _ = writer_conn.execute(
            "INSERT INTO timeline_events (id, story_id, title, description, act, character_name, temporal_order, intensity) VALUES (?1, ?2, 'Переход через перевал', 'Начало пути и первое знамение бури', 'Акт I', 'Элион', 1, 65)",
            rusqlite::params![uuid::Uuid::new_v4().to_string(), genesis_id],
        );
    }

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