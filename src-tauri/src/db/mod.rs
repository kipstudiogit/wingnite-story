// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

pub mod migrations;

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
    migrations::apply_schema_and_migrations(&writer_conn)?;

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