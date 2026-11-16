// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

export interface AppSettings {
  editorFontFamily: string;
  editorLineHeight: number;
  canvasMaxWidth: string;
  smoothCaret: boolean;
  autoSaveIntervalMs: number;
  loreAuditOnInput: boolean;
  loreSensitivity: "STRICT" | "BALANCED" | "PERMISSIVE";
  dailyWordGoal: number;
  bookWordGoal: number;
  sprintDurationMinutes: number;
  hapticsEnabled: boolean;
}

export interface UpdateStatusPayload {
  available: boolean;
  currentVersion: string;
  version?: string;
  date?: string;
  body?: string;
}

export type AppTab = "reader" | "studio" | "board" | "codex" | "graph" | "timeline" | "settings";
export type StudioViewMode = "split" | "zen" | "edit" | "preview";
export type ThemeMode = "obsidian" | "sepia" | "paper" | "oled" | "celestial";
export type EntityKind = "character" | "artifact" | "pantheon" | "sanctuary";
export type StudioDrawerType = "analytics" | "lore" | "vault" | null;

export interface ConfirmDialogOptions {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
}