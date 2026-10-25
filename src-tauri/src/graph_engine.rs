use crate::error::AppError;
use crate::models::{GraphEdgePacked, GraphNodePacked, GraphTopologyPayload};
use crate::EnterpriseDbPool;
use rusqlite::params;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub async fn fetch_packed_topology(
    state: State<'_, EnterpriseDbPool>,
    project_id: Option<String>,
) -> Result<GraphTopologyPayload, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let target_pid = project_id.unwrap_or_else(|| "default_project".to_string());

        let mut node_stmt = conn.prepare_cached(
            "SELECT id, title, category, revision 
             FROM stories 
             WHERE project_id = ?1 
             ORDER BY sort_order ASC, updated_at DESC",
        )?;

        let mut nodes = Vec::new();
        let mut id_to_index = HashMap::new();
        let mut current_idx = 0u32;

        let node_rows = node_stmt.query_map(params![target_pid], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let category: String = row.get(2)?;
            let rev: i64 = row.get(3)?;
            Ok((id, title, category, rev))
        })?;

        for item in node_rows {
            let (id, title, category, rev) = item?;
            let is_divine = category.contains("Артефакт")
                || category.contains("Демиург")
                || category.contains("Реликвия");

            let mass = 1.0 + (rev as f32 * 0.1).min(3.0);

            id_to_index.insert(id.clone(), current_idx);
            nodes.push(GraphNodePacked {
                id,
                index: current_idx,
                label: title,
                category,
                is_divine,
                mass,
                degree: 0,
            });
            current_idx += 1;
        }

        let mut edge_stmt = conn.prepare_cached(
            "SELECT source_id, target_id, relation_type FROM entity_relations",
        )?;

        let mut edges = Vec::new();
        let edge_rows = edge_stmt.query_map([], |row| {
            let s_id: String = row.get(0)?;
            let t_id: String = row.get(1)?;
            let r_type: String = row.get(2)?;
            Ok((s_id, t_id, r_type))
        })?;

        for item in edge_rows {
            let (s_id, t_id, r_type) = item?;
            if let (Some(&src_idx), Some(&tgt_idx)) = (id_to_index.get(&s_id), id_to_index.get(&t_id)) {
                let weight = if r_type.contains("Вражда") {
                    2.0
                } else if r_type.contains("Владение") {
                    1.6
                } else {
                    1.0
                };

                edges.push(GraphEdgePacked {
                    source_idx: src_idx,
                    target_idx: tgt_idx,
                    relation_type: r_type,
                    weight,
                });

                if let Some(src_node) = nodes.get_mut(src_idx as usize) {
                    src_node.degree += 1;
                }
                if let Some(tgt_node) = nodes.get_mut(tgt_idx as usize) {
                    tgt_node.degree += 1;
                }
            }
        }

        let revision = chrono::Utc::now().timestamp();

        Ok(GraphTopologyPayload {
            nodes,
            edges,
            revision,
        })
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}