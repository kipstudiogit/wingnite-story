// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

fn default_project_id() -> String {
    "default_project".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEventItem {
    pub id: String,
    #[serde(default = "default_project_id")]
    pub project_id: String,
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
pub struct EntityRelationItem {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphNodePacked {
    pub id: String,
    pub index: u32,
    pub label: String,
    pub category: String,
    pub is_divine: bool,
    pub mass: f32,
    pub degree: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdgePacked {
    pub source_idx: u32,
    pub target_idx: u32,
    pub relation_type: String,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphTopologyPayload {
    pub nodes: Vec<GraphNodePacked>,
    pub edges: Vec<GraphEdgePacked>,
    pub revision: i64,
}