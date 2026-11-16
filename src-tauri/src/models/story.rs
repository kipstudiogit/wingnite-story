// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

fn default_project_id() -> String {
    "default_project".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnterpriseStoryItem {
    pub id: String,
    #[serde(default = "default_project_id")]
    pub project_id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    pub revision: i64,
    #[serde(default)]
    pub sort_order: i64,
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
    pub sort_order: Option<i64>,
    pub operator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryOrderUpdate {
    pub id: String,
    pub sort_order: i64,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotItem {
    pub id: String,
    pub story_id: String,
    pub project_id: String,
    pub title: String,
    pub content: String,
    pub word_count: i64,
    pub created_at: i64,
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
pub struct TextMetricsResult {
    pub word_count: usize,
    pub char_count: usize,
    pub paragraph_count: usize,
    pub reading_time_minutes: f64,
    pub dialogue_ratio: f64,
    pub sentence_cadence: f64,
    pub vocabulary_richness: f64,
    pub flesch_reading_ease: f64,
    pub sentiment_tone_key: String,
    pub repetition_warnings: Vec<String>,
    pub cliche_warnings: Vec<String>,
    pub pacing_timeline: Vec<f64>,
}