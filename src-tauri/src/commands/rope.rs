// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

use crate::commands::story::mutate_story;
use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use crate::models::{EnterpriseStoryItem, RopeEditPayload, SaveStoryPayload, ViewportSlice};
use crate::services::crypto_vault::VaultService;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

pub struct RopeStateManager {
    pub sessions: Arc<parking_lot::RwLock<HashMap<String, ropey::Rope>>>,
}

fn build_slice(rope: &ropey::Rope, center_char_idx: Option<usize>) -> ViewportSlice {
    let total_lines = rope.len_lines();
    let total_chars = rope.len_chars();
    let start_line = match center_char_idx {
        Some(idx) => {
            let clamped = idx.min(total_chars);
            rope.char_to_line(clamped).saturating_sub(15)
        }
        None => 0,
    };
    let end_line = (start_line + 60).min(total_lines);

    let mut lines = Vec::with_capacity(end_line.saturating_sub(start_line));
    for i in start_line..end_line {
        lines.push(rope.line(i).to_string());
    }

    let full_text = rope.to_string();
    let hash = VaultService::calculate_hash(&full_text);
    let total_words = full_text.split_whitespace().count();

    ViewportSlice {
        lines,
        start_line,
        total_lines,
        total_chars,
        total_words,
        revision_hash: hash,
    }
}

#[tauri::command]
pub async fn rope_init_session(
    state: State<'_, RopeStateManager>,
    session_key: String,
    initial_content: String,
) -> Result<ViewportSlice, AppError> {
    let rope = ropey::Rope::from_str(&initial_content);
    let slice = build_slice(&rope, None);
    state.sessions.write().insert(session_key, rope);
    Ok(slice)
}

#[tauri::command]
pub async fn rope_get_viewport(
    state: State<'_, RopeStateManager>,
    session_key: String,
    start_line: usize,
    line_count: usize,
) -> Result<ViewportSlice, AppError> {
    let sessions = state.sessions.read();
    let rope = sessions
        .get(&session_key)
        .ok_or_else(|| AppError::Rope("Rope session does not exist".to_string()))?;

    let total_lines = rope.len_lines();
    let total_chars = rope.len_chars();
    let valid_start = start_line.min(total_lines);
    let valid_end = (valid_start + line_count).min(total_lines);

    let mut lines = Vec::with_capacity(valid_end.saturating_sub(valid_start));
    for i in valid_start..valid_end {
        lines.push(rope.line(i).to_string());
    }

    let full_text = rope.to_string();
    let hash = VaultService::calculate_hash(&full_text);
    let total_words = full_text.split_whitespace().count();

    Ok(ViewportSlice {
        lines,
        start_line: valid_start,
        total_lines,
        total_chars,
        total_words,
        revision_hash: hash,
    })
}

#[tauri::command]
pub async fn rope_apply_delta(
    state: State<'_, RopeStateManager>,
    payload: RopeEditPayload,
) -> Result<ViewportSlice, AppError> {
    let mut sessions = state.sessions.write();
    let rope = sessions
        .get_mut(&payload.session_key)
        .ok_or_else(|| AppError::Rope("Rope session does not exist".to_string()))?;

    let len = rope.len_chars();
    let idx = payload.char_idx.min(len);

    if let Some(del_len) = payload.delete_len {
        let end = (idx + del_len).min(len);
        if idx < end {
            rope.remove(idx..end);
        }
    }

    if let Some(ins_text) = payload.insert_text {
        if !ins_text.is_empty() {
            rope.insert(idx, &ins_text);
        }
    }

    Ok(build_slice(rope, Some(idx)))
}

#[tauri::command]
pub async fn rope_close_session(
    state: State<'_, RopeStateManager>,
    session_key: String,
) -> Result<(), AppError> {
    state.sessions.write().remove(&session_key);
    Ok(())
}

#[tauri::command]
pub async fn rope_flush_to_storage(
    db_state: State<'_, EnterpriseDbPool>,
    rope_state: State<'_, RopeStateManager>,
    session_key: String,
    story_id: String,
    project_id: String,
    title: String,
    category: String,
    operator: String,
) -> Result<EnterpriseStoryItem, AppError> {
    let content = {
        let sessions = rope_state.sessions.read();
        let rope = sessions
            .get(&session_key)
            .ok_or_else(|| AppError::Rope("Rope session does not exist".to_string()))?;
        rope.to_string()
    };

    let payload = SaveStoryPayload {
        id: story_id,
        project_id,
        title,
        content,
        category,
        expected_revision: 0,
        sort_order: None,
        operator,
    };

    mutate_story(db_state, payload).await
}