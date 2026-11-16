// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod services;

pub use commands::rope::RopeStateManager;
pub use db::EnterpriseDbPool;
pub use error::AppError;
pub use models::*;

use std::collections::HashMap;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                let _ = app.handle().plugin(tauri_plugin_updater::Builder::new().build());
            }

            let pool = db::init_database(app.handle())?;
            app.manage(pool);

            app.manage(RopeStateManager {
                sessions: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::vault::vault_get_status,
            commands::vault::vault_seal_project,
            commands::vault::vault_unlock_project,
            commands::vault::vault_verify_merkle_tree,
            commands::vault::fetch_story_snapshots,
            commands::vault::create_story_snapshot,
            commands::vault::delete_story_snapshot,
            commands::rope::rope_init_session,
            commands::rope::rope_get_viewport,
            commands::rope::rope_apply_delta,
            commands::rope::rope_close_session,
            commands::rope::rope_flush_to_storage,
            commands::project::fetch_projects,
            commands::project::mutate_project,
            commands::project::delete_project,
            commands::db::app_checkpoint_wal,
            commands::db::app_vacuum_db,
            commands::db::app_get_db_stats,
            commands::export::compile_book_edition,
            commands::export::export_manuscript,
            commands::export::export_manuscript_txt,
            commands::export::import_plain_text_chapters,
            commands::analytics::analyze_text_metrics,
            commands::analytics::generate_twist,
            commands::story::fetch_stories,
            commands::story::mutate_story,
            commands::story::delete_story,
            commands::story::reorder_stories,
            commands::story::execute_fts_search,
            commands::wstory::save_export_file,
            commands::wstory::export_project_wstory,
            commands::wstory::import_project_wstory,
            commands::updater::check_application_update,
            commands::updater::install_application_update,
            commands::timeline::fetch_timeline_events,
            commands::timeline::mutate_timeline_event,
            commands::timeline::delete_timeline_event,
            commands::timeline::fetch_entity_relations,
            commands::timeline::mutate_entity_relation,
            commands::timeline::delete_entity_relation,
            commands::lsp::lsp_complete_wikilink,
            commands::lsp::lsp_hover_entity,
            commands::lsp::lsp_resolve_wikilink,
            commands::graph::fetch_packed_topology,
            commands::lore::lore_semantic_search,
            commands::lore::lore_verify_consistency
        ])
        .run(tauri::generate_context!())
        .expect("Fatal error during Wingnite runtime execution");
}