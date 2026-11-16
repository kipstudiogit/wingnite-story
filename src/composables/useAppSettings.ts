// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

import { ref, watch } from "vue";
import { DEFAULT_BOOK_WORD_GOAL } from "../constants/app";
import type { AppSettings } from "../types";

const STORAGE_KEY = "wingnite_enterprise_settings_v140";

const defaultSettings: AppSettings = {
  editorFontFamily: "var(--font-sans)",
  editorLineHeight: 1.85,
  canvasMaxWidth: "100%",
  smoothCaret: true,
  autoSaveIntervalMs: 2500,
  loreAuditOnInput: true,
  loreSensitivity: "BALANCED",
  dailyWordGoal: 1000,
  bookWordGoal: DEFAULT_BOOK_WORD_GOAL,
  sprintDurationMinutes: 15,
  hapticsEnabled: true,
};

function loadStoredSettings(): AppSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY) || localStorage.getItem("wingnite_enterprise_settings_v130");
    if (raw) {
      return { ...defaultSettings, ...JSON.parse(raw) };
    }
  } catch {
  }
  return { ...defaultSettings };
}

export const settings = ref<AppSettings>(loadStoredSettings());

watch(
  settings,
  (next) => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    } catch {
    }
  },
  { deep: true }
);

export function useAppSettings() {
  function resetToDefaults() {
    settings.value = { ...defaultSettings };
  }

  return {
    settings,
    resetToDefaults,
  };
}