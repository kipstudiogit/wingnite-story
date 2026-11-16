// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

export interface TimelineEventItem {
  id: string;
  projectId: string;
  storyId?: string | null;
  title: string;
  description: string;
  act: string;
  characterName: string;
  temporalOrder: number;
  intensity: number;
}

export interface EntityRelationItem {
  id: string;
  sourceId: string;
  targetId: string;
  relationType: string;
  description: string;
}

export interface GraphNodePacked {
  id: string;
  index: number;
  label: string;
  category: string;
  isDivine: boolean;
  mass: number;
  degree: number;
}

export interface GraphEdgePacked {
  sourceIdx: number;
  targetIdx: number;
  relationType: string;
  weight: number;
}

export interface GraphTopologyPayload {
  nodes: GraphNodePacked[];
  edges: GraphEdgePacked[];
  revision: number;
}