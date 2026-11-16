// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

use super::narrative::{EntityRelationItem, TimelineEventItem};
use super::project::ProjectItem;
use super::story::EnterpriseStoryItem;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewportSlice {
    pub lines: Vec<String>,
    pub start_line: usize,
    pub total_lines: usize,
    pub total_chars: usize,
    pub total_words: usize,
    pub revision_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RopeEditPayload {
    pub session_key: String,
    pub char_idx: usize,
    pub insert_text: Option<String>,
    pub delete_len: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultStatus {
    pub project_id: String,
    pub is_sealed: bool,
    pub is_locked: bool,
    pub merkle_root: String,
    pub total_entities: usize,
    pub total_revisions: i64,
    pub last_sealed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultSealPayload {
    pub project_id: String,
    pub passphrase: String,
    pub lock_editing: bool,
    pub operator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultUnlockPayload {
    pub project_id: String,
    pub passphrase: String,
    pub operator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultVerificationResult {
    pub is_valid: bool,
    pub calculated_root: String,
    pub recorded_root: String,
    pub verified_nodes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusPayload {
    pub available: bool,
    pub current_version: String,
    pub version: Option<String>,
    pub date: Option<String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WstoryPackage {
    pub format: String,
    pub format_version: String,
    pub exported_at: i64,
    pub generator: String,
    pub project: ProjectItem,
    pub stories: Vec<EnterpriseStoryItem>,
    pub timeline_events: Vec<TimelineEventItem>,
    pub entity_relations: Vec<EntityRelationItem>,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspSuggestion {
    pub id: String,
    pub title: String,
    pub category: String,
    pub preview: String,
    pub link_type: String,
    pub match_score: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspHoverInfo {
    pub id: String,
    pub title: String,
    pub category: String,
    pub summary: String,
    pub connected_entities: Vec<String>,
    pub revision: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspQueryPayload {
    pub project_id: String,
    pub story_id: String,
    pub raw_token: String,
    pub trigger_char: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiLinkResolvePayload {
    pub project_id: String,
    pub source_story_id: String,
    pub target_title: String,
    pub operator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiLinkResolveResult {
    pub target_id: String,
    pub target_title: String,
    pub is_new_entity: bool,
    pub relation_id: String,
    pub created_story: Option<EnterpriseStoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticSearchResult {
    pub id: String,
    pub title: String,
    pub category: String,
    pub similarity: f32,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreAnomalyItem {
    pub id: String,
    pub entity_title: String,
    pub conflict_type: String,
    pub claim_in_text: String,
    pub original_lore_fact: String,
    pub severity: String,
    pub suggested_fix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreAnomalyReport {
    pub anomalies: Vec<LoreAnomalyItem>,
    pub verified_claims_count: usize,
    pub consistency_score: f32,
}