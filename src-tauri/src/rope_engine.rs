use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use crate::models::{EnterpriseStoryItem, RopeEditPayload, SaveStoryPayload, ViewportSlice};
use crate::story_commands::mutate_story;
use crate::vault::VaultService;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

pub struct RopeStateManager {
    pub sessions: Arc<parking_lot::RwLock<HashMap<String, ropey::Rope>>>,
}

#[tauri::command]
pub async fn rope_init_session(
    state: State<'_, RopeStateManager>,
    story_id: String,
    initial_content: String,
) -> Result<ViewportSlice, AppError> {
    let rope = ropey::Rope::from_str(&initial_content);
    let total_lines = rope.len_lines();
    let total_chars = rope.len_chars();
    let preview_end = 60.min(total_lines);

    let mut lines = Vec::with_capacity(preview_end);
    for i in 0..preview_end {
        lines.push(rope.line(i).to_string());
    }

    let hash = VaultService::calculate_hash(&initial_content);
    let total_words = initial_content.split_whitespace().count();

    state.sessions.write().insert(story_id, rope);

    Ok(ViewportSlice {
        lines,
        start_line: 0,
        total_lines,
        total_chars,
        total_words,
        revision_hash: hash,
    })
}

#[tauri::command]
pub async fn rope_get_viewport(
    state: State<'_, RopeStateManager>,
    story_id: String,
    start_line: usize,
    line_count: usize,
) -> Result<ViewportSlice, AppError> {
    let sessions = state.sessions.read();
    let rope = sessions
        .get(&story_id)
        .ok_or_else(|| AppError::Rope("Rope session does not exist".to_string()))?;

    let total_lines = rope.len_lines();
    let total_chars = rope.len_chars();
    let valid_start = start_line.min(total_lines);
    let valid_end = (valid_start + line_count).min(total_lines);

    let mut lines = Vec::with_capacity(valid_end - valid_start);
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
        .get_mut(&payload.story_id)
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
        rope.insert(idx, &ins_text);
    }

    let total_lines = rope.len_lines();
    let total_chars = rope.len_chars();
    let line_at_idx = rope.char_to_line(idx.min(total_chars));
    let start_line = line_at_idx.saturating_sub(15);
    let end_line = (start_line + 60).min(total_lines);

    let mut lines = Vec::with_capacity(end_line - start_line);
    for i in start_line..end_line {
        lines.push(rope.line(i).to_string());
    }

    let full_text = rope.to_string();
    let hash = VaultService::calculate_hash(&full_text);
    let total_words = full_text.split_whitespace().count();

    Ok(ViewportSlice {
        lines,
        start_line,
        total_lines,
        total_chars,
        total_words,
        revision_hash: hash,
    })
}

#[tauri::command]
pub async fn rope_flush_to_storage(
    db_state: State<'_, EnterpriseDbPool>,
    rope_state: State<'_, RopeStateManager>,
    story_id: String,
    title: String,
    category: String,
    operator: String,
) -> Result<EnterpriseStoryItem, AppError> {
    let content = {
        let sessions = rope_state.sessions.read();
        let rope = sessions
            .get(&story_id)
            .ok_or_else(|| AppError::Rope("Rope session does not exist".to_string()))?;
        rope.to_string()
    };

    let payload = SaveStoryPayload {
        id: story_id,
        title,
        content,
        category,
        expected_revision: 0,
        operator,
    };

    mutate_story(db_state, payload).await
}