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

export interface EnterpriseStoryItem {
  id: string;
  projectId: string;
  title: string;
  content: string;
  category: string;
  revision: number;
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
  operator: string;
}

export interface AuditLogEntry {
  id: string;
  entityId: string;
  action: string;
  operator: string;
  revision: number;
  timestamp: number;
}

export interface SnapshotItem {
  id: string;
  storyId: string;
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
  readingTimeMinutes: number;
  dialogueRatio: number;
  sentenceCadence: number;
  vocabularyRichness: number;
  repetitionWarnings: string[];
}

export interface TimelineEventItem {
  id: string;
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

export interface ViewportSlice {
  lines: string[];
  startLine: number;
  totalLines: number;
  totalChars: number;
  totalWords: number;
  revisionHash: string;
}

export interface RopeEditPayload {
  storyId: string;
  charIdx: number;
  insertText?: string;
  deleteLen?: number;
}

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
  storyId: string;
  rawToken: string;
  triggerChar: string;
}

export interface WikiLinkResolvePayload {
  sourceStoryId: string;
  targetTitle: string;
  operator: string;
}

export interface WikiLinkResolveResult {
  targetId: string;
  targetTitle: string;
  isNewEntity: boolean;
  relationId: string;
}

export interface GraphNodePacked {
  id: string;
  index: number;
  label: string;
  category: string;
  isDivine: boolean;
  mass: number;
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

export interface SemanticSearchResult {
  id: string;
  title: string;
  category: string;
  similarity: number;
  snippet: string;
}

export interface LoreAnomalyItem {
  entityTitle: string;
  conflictType: string;
  claimInText: string;
  originalLoreFact: string;
  severity: string;
}

export interface LoreAnomalyReport {
  anomalies: LoreAnomalyItem[];
  verifiedClaimsCount: number;
  consistencyScore: number;
}

export interface VaultStatus {
  isSealed: boolean;
  merkleRoot: string;
  totalEntities: number;
  totalRevisions: number;
  lastSealedAt: number;
}

export interface VaultSealPayload {
  passphraseSignature: string;
  operator: string;
}

export interface VaultVerificationResult {
  isValid: boolean;
  calculatedRoot: string;
  recordedRoot: string;
  verifiedNodes: number;
}

export interface UpdateStatusPayload {
  available: boolean;
  currentVersion: string;
  version?: string;
  date?: string;
  body?: string;
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

export type AppTab = "reader" | "studio" | "board" | "codex" | "graph" | "timeline";
export type StudioViewMode = "split" | "zen" | "edit" | "preview";
export type ThemeMode = "obsidian" | "sepia" | "paper" | "oled" | "celestial";
export type DivineRank = "Демиург" | "Реликвия" | "Святилище" | "Персонаж" | "Акт";