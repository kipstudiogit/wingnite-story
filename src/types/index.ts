export interface StoryItem {
  id: string;
  title: string;
  content: string;
  category: string;
  updatedAt: number;
}

export interface SnapshotItem {
  id: string;
  storyId: string;
  title: string;
  content: string;
  wordCount: number;
  createdAt: number;
}

export interface FtsSearchResult {
  id: string;
  title: string;
  category: string;
  snippet: string;
  rank: number;
}

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

export type AppTab = "reader" | "studio" | "board" | "codex" | "graph" | "timeline";
export type StudioViewMode = "split" | "zen" | "edit";
export type ThemeMode = "obsidian" | "sepia" | "paper" | "oled" | "celestial";
export type DivineRank = "Демиург" | "Реликвия" | "Святилище" | "Персонаж" | "Акт";