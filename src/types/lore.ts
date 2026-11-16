// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

import type { EnterpriseStoryItem } from "./story";

export interface LspSuggestion {
  id: string;
  title: string;
  category: string;
  preview: string;
  linkType: string;
  matchScore: number;
}

export interface LspHoverInfo {
  id: string;
  title: string;
  category: string;
  summary: string;
  connectedEntities: string[];
  revision: number;
}

export interface LspQueryPayload {
  projectId: string;
  storyId: string;
  rawToken: string;
  triggerChar: string;
}

export interface WikiLinkResolvePayload {
  projectId: string;
  sourceStoryId: string;
  targetTitle: string;
  operator: string;
}

export interface WikiLinkResolveResult {
  targetId: string;
  targetTitle: string;
  isNewEntity: boolean;
  relationId: string;
  createdStory?: EnterpriseStoryItem | null;
}

export interface SemanticSearchResult {
  id: string;
  title: string;
  category: string;
  similarity: number;
  snippet: string;
}

export interface LoreAnomalyItem {
  id: string;
  entityTitle: string;
  conflictType: string;
  claimInText: string;
  originalLoreFact: string;
  severity: "CRITICAL" | "MAJOR" | "MINOR" | "SUGGESTION" | string;
  suggestedFix: string;
}

export interface LoreAnomalyReport {
  anomalies: LoreAnomalyItem[];
  verifiedClaimsCount: number;
  consistencyScore: number;
}

export interface VaultStatus {
  projectId: string;
  isSealed: boolean;
  isLocked: boolean;
  merkleRoot: string;
  totalEntities: number;
  totalRevisions: number;
  lastSealedAt: number;
}

export interface VaultSealPayload {
  projectId: string;
  passphrase: string;
  lockEditing: boolean;
  operator: string;
}

export interface VaultUnlockPayload {
  projectId: string;
  passphrase: string;
  operator: string;
}

export interface VaultVerificationResult {
  isValid: boolean;
  calculatedRoot: string;
  recordedRoot: string;
  verifiedNodes: number;
}