// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

use crate::error::AppError;
use rusqlite::Connection;

fn column_exists(conn: &Connection, table: &str, column_name: &str) -> Result<bool, AppError> {
    let pragma_sql = format!("PRAGMA table_info({})", table);
    let mut stmt = conn.prepare(&pragma_sql)?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let col_name: String = row.get(1)?;
        if col_name == column_name {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn apply_schema_and_migrations(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
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

         CREATE TABLE IF NOT EXISTS story_snapshots (
             id TEXT PRIMARY KEY,
             story_id TEXT NOT NULL,
             project_id TEXT NOT NULL,
             title TEXT NOT NULL,
             content TEXT NOT NULL,
             word_count INTEGER NOT NULL DEFAULT 0,
             created_at INTEGER NOT NULL,
             FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE,
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
             project_id TEXT NOT NULL DEFAULT 'default_project',
             story_id TEXT,
             title TEXT NOT NULL,
             description TEXT NOT NULL,
             act TEXT NOT NULL,
             character_name TEXT NOT NULL,
             temporal_order INTEGER NOT NULL,
             intensity INTEGER NOT NULL,
             FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
             FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE SET NULL
         );

         CREATE TABLE IF NOT EXISTS lore_embeddings (
             story_id TEXT PRIMARY KEY,
             vector BLOB NOT NULL,
             updated_at INTEGER NOT NULL,
             FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE
         );

         CREATE TABLE IF NOT EXISTS project_vaults (
             project_id TEXT PRIMARY KEY,
             merkle_root TEXT NOT NULL,
             sealed_at INTEGER NOT NULL,
             operator TEXT NOT NULL,
             passphrase_salt TEXT NOT NULL,
             passphrase_hash TEXT NOT NULL,
             is_locked INTEGER NOT NULL DEFAULT 0,
             FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
         );",
    )?;

    if !column_exists(conn, "stories", "project_id")? {
        conn.execute(
            "ALTER TABLE stories ADD COLUMN project_id TEXT NOT NULL DEFAULT 'default_project'",
            [],
        )?;
    }

    if !column_exists(conn, "stories", "sort_order")? {
        conn.execute(
            "ALTER TABLE stories ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }

    if !column_exists(conn, "timeline_events", "project_id")? {
        conn.execute(
            "ALTER TABLE timeline_events ADD COLUMN project_id TEXT NOT NULL DEFAULT 'default_project'",
            [],
        )?;
        let _ = conn.execute(
            "UPDATE timeline_events
             SET project_id = COALESCE(
                 (SELECT s.project_id FROM stories s WHERE s.id = timeline_events.story_id),
                 'default_project'
             )",
            [],
        );
    }

    conn.execute_batch(
        "DROP TABLE IF EXISTS vault_metadata;

         CREATE INDEX IF NOT EXISTS idx_stories_project ON stories(project_id);
         CREATE INDEX IF NOT EXISTS idx_stories_order ON stories(project_id, sort_order ASC);
         CREATE INDEX IF NOT EXISTS idx_stories_revision ON stories(id, revision);
         CREATE INDEX IF NOT EXISTS idx_snapshots_story ON story_snapshots(story_id, created_at DESC);
         CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_ledger(entity_id);
         CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_ledger(timestamp DESC);
         CREATE INDEX IF NOT EXISTS idx_rel_source ON entity_relations(source_id);
         CREATE INDEX IF NOT EXISTS idx_rel_target ON entity_relations(target_id);
         CREATE INDEX IF NOT EXISTS idx_timeline_project ON timeline_events(project_id);
         CREATE INDEX IF NOT EXISTS idx_timeline_order ON timeline_events(project_id, temporal_order ASC);

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
    conn.execute(
        "INSERT OR IGNORE INTO projects (id, title, description, created_at, updated_at)
         VALUES ('default_project', 'My Book', 'Primary manuscript', ?1, ?1)",
        rusqlite::params![now],
    )?;

    let _ = conn.execute(
        "UPDATE stories SET project_id = 'default_project' WHERE project_id IS NULL OR project_id = ''",
        [],
    );

    let _ = conn.execute(
        "DELETE FROM stories WHERE id IN ('genesis_1', 'genesis_2', 'genesis_3')",
        [],
    );

    Ok(())
}