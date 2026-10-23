use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProjectPayload {
    pub id: Option<String>,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnterpriseStoryItem {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    pub revision: i64,
    pub content_hash: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveStoryPayload {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    pub expected_revision: i64,
    pub operator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogEntry {
    pub id: String,
    pub entity_id: String,
    pub action: String,
    pub operator: String,
    pub revision: i64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextMetricsResult {
    pub word_count: usize,
    pub char_count: usize,
    pub reading_time_minutes: f64,
    pub dialogue_ratio: f64,
    pub sentence_cadence: f64,
    pub vocabulary_richness: f64,
    pub repetition_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FtsResultItem {
    pub id: String,
    pub title: String,
    pub category: String,
    pub snippet: String,
    pub rank: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityRelationItem {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEventItem {
    pub id: String,
    pub story_id: Option<String>,
    pub title: String,
    pub description: String,
    pub act: String,
    pub character_name: String,
    pub temporal_order: i64,
    pub intensity: i64,
}

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
    pub story_id: String,
    pub char_idx: usize,
    pub insert_text: Option<String>,
    pub delete_len: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultStatus {
    pub is_sealed: bool,
    pub merkle_root: String,
    pub total_entities: usize,
    pub total_revisions: i64,
    pub last_sealed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultSealPayload {
    pub passphrase_signature: String,
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