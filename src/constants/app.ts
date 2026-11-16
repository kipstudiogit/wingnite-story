// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

import type { EntityKind, ThemeMode } from "../types";

export const APP_DISPLAY_VERSION = "1.4.0001";
export const APP_SEMVER = "1.4.1";
export const APP_LICENSE = "GPL-3.0-only";
export const APP_PUBLISHER = "Kettle Ignition Power Studio (K. I. P. Studio)";
export const DEFAULT_PROJECT_ID = "default_project";
export const DEFAULT_BOOK_WORD_GOAL = 30000;

export const THEME_META_COLORS: Record<ThemeMode, string> = {
  obsidian: "#0e121c",
  celestial: "#13100c",
  sepia: "#f4ede0",
  paper: "#ffffff",
  oled: "#000000",
};

const CATEGORY_MATCHERS: Record<EntityKind, string[]> = {
  character: ["character", "персонаж", "charakter", "personaje"],
  artifact: ["artifact", "relic", "артефакт", "реликвия", "artefakt", "relikt", "artefacto", "reliquia"],
  pantheon: ["demiurge", "entity", "демиург", "сущность", "entität", "entidad"],
  sanctuary: ["sanctuary", "location", "святилище", "локация", "heiligtum", "ort", "santuario", "ubicación"],
};

export function classifyEntityCategory(category: string): EntityKind | null {
  const lower = category.toLowerCase();
  for (const kind of Object.keys(CATEGORY_MATCHERS) as EntityKind[]) {
    if (CATEGORY_MATCHERS[kind].some((kw) => lower.includes(kw))) {
      return kind;
    }
  }
  return null;
}

export function isCodexEntityCategory(category: string): boolean {
  return classifyEntityCategory(category) !== null;
}

export function isEnemyRelationType(relationType: string): boolean {
  const lower = relationType.toLowerCase();
  return (
    lower.includes("enemy") ||
    lower.includes("rivalry") ||
    lower.includes("conflict") ||
    lower.includes("вражд") ||
    lower.includes("конфликт") ||
    lower.includes("rival") ||
    lower.includes("konflikt")
  );
}