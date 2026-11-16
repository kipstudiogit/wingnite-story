// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

import type { ProjectItem } from "./project";
import type { EntityRelationItem, TimelineEventItem } from "./narrative";

export interface EnterpriseStoryItem {
  id: string;
  projectId: string;
  title: string;
  content: string;
  category: string;
  revision: number;
  sortOrder: number;
  contentHash: string;
  updatedAt: number;
}

export type StoryItem = EnterpriseStoryItem;

export interface SaveStoryPayload {
  id: string;
  projectId: string;
  title: string;
  content: string;
  category: string;
  expectedRevision: number;
  sortOrder?: number;
  operator: string;
}

export interface StoryOrderUpdate {
  id: string;
  sortOrder: number;
  category?: string;
}

export interface SnapshotItem {
  id: string;
  storyId: string;
  projectId: string;
  title: string;
  content: string;
  wordCount: number;
  createdAt: number;
}

export interface FtsResultItem {
  id: string;
  title: string;
  category: string;
  snippet: string;
  rank: number;
}

export type FtsSearchResult = FtsResultItem;

export interface TextMetricsResult {
  wordCount: number;
  charCount: number;
  paragraphCount: number;
  readingTimeMinutes: number;
  dialogueRatio: number;
  sentenceCadence: number;
  vocabularyRichness: number;
  fleschReadingEase: number;
  sentimentToneKey: "dialogue" | "action" | "prose" | "epic" | "neutral";
  repetitionWarnings: string[];
  clicheWarnings: string[];
  pacingTimeline: number[];
}

export interface ViewportSlice {
  lines: string[];
  startLine: number;
  totalLines: number;
  totalChars: number;
  totalWords: number;
  revisionHash: string;
}

export interface RopeEditPayload {
  sessionKey: string;
  charIdx: number;
  insertText?: string;
  deleteLen?: number;
}

export interface WstoryPackage {
  format: string;
  formatVersion: string;
  exportedAt: number;
  generator: string;
  project: ProjectItem;
  stories: EnterpriseStoryItem[];
  timelineEvents: TimelineEventItem[];
  entityRelations: EntityRelationItem[];
  checksum: string;
}