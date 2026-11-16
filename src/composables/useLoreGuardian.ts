// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

import { ref, shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { LoreAnomalyItem, LoreAnomalyReport, SemanticSearchResult } from "../types";
import { useAppSettings } from "./useAppSettings";
import { useI18n } from "../i18n";

const IGNORED_STORAGE_KEY = "wingnite_ignored_lore_v140";

function loadIgnoredKeys(): Set<string> {
  try {
    const raw = localStorage.getItem(IGNORED_STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed)) {
        return new Set<string>(parsed);
      }
    }
  } catch {
  }
  return new Set<string>();
}

function saveIgnoredKeys(keys: Set<string>) {
  try {
    localStorage.setItem(IGNORED_STORAGE_KEY, JSON.stringify(Array.from(keys)));
  } catch {
  }
}

export function useLoreGuardian(projectIdGetter: () => string) {
  const { settings } = useAppSettings();
  const { locale } = useI18n();

  const isAnalyzing = ref<boolean>(false);
  const consistencyScore = ref<number>(100);
  const verifiedClaimsCount = ref<number>(0);
  const anomalies = shallowRef<LoreAnomalyItem[]>([]);
  const semanticResults = shallowRef<SemanticSearchResult[]>([]);
  const guardianError = ref<string | null>(null);
  const ignoredKeys = ref<Set<string>>(loadIgnoredKeys());

  let verifyDebounce: number | null = null;

  function buildScopedAnomalyKey(anom: LoreAnomalyItem): string {
    return `${projectIdGetter()}::${anom.id}`;
  }

  function applyFilteredReport(report: LoreAnomalyReport) {
    const activeAnomalies = report.anomalies.filter(
      (a) => !ignoredKeys.value.has(buildScopedAnomalyKey(a))
    );
    anomalies.value = activeAnomalies;
    verifiedClaimsCount.value = report.verifiedClaimsCount;
    const penalty = Math.min(100, activeAnomalies.length * 14);
    consistencyScore.value = Math.max(0, 100 - penalty);
  }

  function triggerLoreAuditDebounced(content: string, delayMs = 380) {
    if (verifyDebounce) clearTimeout(verifyDebounce);
    if (!content.trim()) {
      anomalies.value = [];
      consistencyScore.value = 100;
      verifiedClaimsCount.value = 0;
      return;
    }

    verifyDebounce = window.setTimeout(async () => {
      isAnalyzing.value = true;
      guardianError.value = null;

      try {
        const report = await invoke<LoreAnomalyReport>("lore_verify_consistency", {
          activeContent: content,
          projectId: projectIdGetter(),
          sensitivity: settings.value.loreSensitivity,
          locale: locale.value,
        });
        applyFilteredReport(report);
      } catch (err) {
        guardianError.value = String(err);
      } finally {
        isAnalyzing.value = false;
      }
    }, delayMs);
  }

  async function executeSemanticSearch(query: string, limit = 8): Promise<void> {
    if (!query.trim()) {
      semanticResults.value = [];
      return;
    }

    isAnalyzing.value = true;
    guardianError.value = null;

    try {
      const results = await invoke<SemanticSearchResult[]>("lore_semantic_search", {
        query,
        projectId: projectIdGetter(),
        limit,
      });
      semanticResults.value = results;
    } catch (err) {
      guardianError.value = String(err);
      semanticResults.value = [];
    } finally {
      isAnalyzing.value = false;
    }
  }

  function dismissAnomaly(index: number) {
    const next = [...anomalies.value];
    const [removed] = next.splice(index, 1);
    if (removed) {
      ignoredKeys.value.add(buildScopedAnomalyKey(removed));
      saveIgnoredKeys(ignoredKeys.value);
    }
    anomalies.value = next;
    const penalty = Math.min(100, next.length * 14);
    consistencyScore.value = Math.max(0, 100 - penalty);
  }

  return {
    isAnalyzing,
    consistencyScore,
    verifiedClaimsCount,
    anomalies,
    semanticResults,
    guardianError,
    triggerLoreAuditDebounced,
    executeSemanticSearch,
    dismissAnomaly,
  };
}