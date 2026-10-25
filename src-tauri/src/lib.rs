pub mod analytics_commands;
pub mod db;
pub mod db_commands;
pub mod error;
pub mod export_commands;
pub mod graph_engine;
pub mod lore_guardian;
pub mod lsp;
pub mod models;
pub mod project_commands;
pub mod rope_engine;
pub mod story_commands;
pub mod timeline_commands;
pub mod updater_commands;
pub mod vault;
pub mod wstory_commands;

pub use db::EnterpriseDbPool;
pub use error::AppError;
pub use models::*;
pub use rope_engine::RopeStateManager;

use std::collections::HashMap;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
            vault::vault_get_status,
            vault::vault_seal_project,
            vault::vault_verify_merkle_tree,
            rope_engine::rope_init_session,
            rope_engine::rope_get_viewport,
            rope_engine::rope_apply_delta,
            rope_engine::rope_flush_to_storage,
            project_commands::fetch_projects,
            project_commands::mutate_project,
            project_commands::delete_project,
            db_commands::app_checkpoint_wal,
            db_commands::app_vacuum_db,
            db_commands::app_get_db_stats,
            export_commands::compile_book_edition,
            export_commands::export_manuscript,
            export_commands::export_manuscript_txt,
            export_commands::import_plain_text_chapters,
            analytics_commands::analyze_text_metrics,
            analytics_commands::execute_fts_search,
            analytics_commands::stream_audit_trail,
            analytics_commands::generate_twist,
            story_commands::fetch_stories,
            story_commands::get_stories,
            story_commands::mutate_story,
            story_commands::save_story,
            story_commands::delete_story,
            story_commands::reorder_stories,
            wstory_commands::save_export_file,
            wstory_commands::export_project_wstory,
            wstory_commands::import_project_wstory,
            updater_commands::check_application_update,
            updater_commands::install_application_update,
            timeline_commands::fetch_timeline_events,
            timeline_commands::mutate_timeline_event,
            timeline_commands::delete_timeline_event,
            timeline_commands::fetch_entity_relations,
            timeline_commands::mutate_entity_relation,
            timeline_commands::delete_entity_relation,
            lsp::lsp_complete_wikilink,
            lsp::lsp_hover_entity,
            lsp::lsp_resolve_wikilink,
            graph_engine::fetch_packed_topology,
            lore_guardian::lore_clear_embeddings_cache,
            lore_guardian::lore_index_story,
            lore_guardian::lore_semantic_search,
            lore_guardian::lore_verify_consistency
        ])
        .run(tauri::generate_context!())
        .expect("Fatal error during Enterprise Runtime execution");
}