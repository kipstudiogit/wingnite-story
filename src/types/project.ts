// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

export interface ProjectItem {
  id: string;
  title: string;
  description: string;
  createdAt: number;
  updatedAt: number;
}

export interface SaveProjectPayload {
  id?: string;
  title: string;
  description: string;
}

export interface DbStatsPayload {
  totalStories: number;
  totalProjects: number;
  totalRelations: number;
  totalTimelineEvents: number;
  dbSizeBytes: number;
}