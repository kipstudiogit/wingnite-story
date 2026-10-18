import { ref, shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";

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

export function useLoreGuardian() {
  const isAnalyzing = ref<boolean>(false);
  const consistencyScore = ref<number>(100);
  const verifiedClaimsCount = ref<number>(0);
  const anomalies = shallowRef<LoreAnomalyItem[]>([]);
  const semanticResults = shallowRef<SemanticSearchResult[]>([]);
  const guardianError = ref<string | null>(null);

  let verifyDebounce: number | null = null;

  function triggerLoreAuditDebounced(content: string, delayMs = 400) {
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
        });

        anomalies.value = report.anomalies;
        consistencyScore.value = report.consistencyScore;
        verifiedClaimsCount.value = report.verifiedClaimsCount;
      } catch (err) {
        guardianError.value = String(err);
      } finally {
        isAnalyzing.value = false;
      }
    }, delayMs);
  }

  async function indexSceneEmbedding(storyId: string, content: string): Promise<void> {
    if (!storyId || !content.trim()) return;
    try {
      await invoke("lore_index_story", { storyId, content });
    } catch (err) {
      guardianError.value = String(err);
    }
  }

  async function executeSemanticSearch(query: string, limit = 6): Promise<void> {
    if (!query.trim()) {
      semanticResults.value = [];
      return;
    }

    isAnalyzing.value = true;
    guardianError.value = null;

    try {
      const results = await invoke<SemanticSearchResult[]>("lore_semantic_search", {
        query,
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
    next.splice(index, 1);
    anomalies.value = next;
    const penalty = Math.min(100, next.length * 15);
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
    indexSceneEmbedding,
    executeSemanticSearch,
    dismissAnomaly,
  };
}