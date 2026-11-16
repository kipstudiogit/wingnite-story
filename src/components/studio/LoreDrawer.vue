<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { ref, computed } from "vue";
import type { LoreAnomalyItem, SemanticSearchResult } from "../../types";
import { useI18n } from "../../i18n";

const { consistencyScore, verifiedClaimsCount, anomalies, semanticResults, isAnalyzing } = defineProps<{
  consistencyScore: number;
  verifiedClaimsCount: number;
  anomalies: LoreAnomalyItem[];
  semanticResults: SemanticSearchResult[];
  isAnalyzing: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "dismissAnomaly", index: number): void;
  (e: "searchSemantic", query: string): void;
  (e: "selectStoryById", storyId: string): void;
}>();

const { t } = useI18n();

const semanticQuery = ref("");
const selectedFilter = ref<"ALL" | "CRITICAL" | "MAJOR" | "MINOR">("ALL");

const filteredAnomalies = computed(() => {
  if (selectedFilter.value === "ALL") return anomalies;
  return anomalies.filter((a) => a.severity.toUpperCase() === selectedFilter.value);
});

function handleDismiss(anom: LoreAnomalyItem) {
  const realIndex = anomalies.indexOf(anom);
  if (realIndex !== -1) {
    emit("dismissAnomaly", realIndex);
  }
}

function handleSearch() {
  emit("searchSemantic", semanticQuery.value);
}
</script>

<template>
  <aside class="lore-drawer">
    <header class="drawer-header-strip">
      <div class="header-left">
        <span class="lore-sigil-orb">◈</span>
        <h4>{{ t("lore.title") }}</h4>
      </div>
      <button class="btn-drawer-exit" @click="emit('close')">✕</button>
    </header>

    <div class="lore-score-widget">
      <div class="widget-head">
        <span>{{ t("lore.scoreLabel") }}</span>
        <b :class="{ warning: consistencyScore < 80, danger: consistencyScore < 50 }">
          {{ Math.round(consistencyScore) }}%
        </b>
      </div>
      <div class="score-track">
        <div class="score-bar" :style="{ width: `${consistencyScore}%` }"></div>
      </div>
      <div class="score-meta">
        <span>{{ t("lore.checkedPredicates", { count: verifiedClaimsCount }) }}</span>
        <span class="meta-status">{{ anomalies.length === 0 ? t("lore.noParadoxes") : t("lore.needsAttention") }}</span>
      </div>
    </div>

    <div class="severity-filter-row">
      <button
        :class="['filter-chip', { active: selectedFilter === 'ALL' }]"
        @click="selectedFilter = 'ALL'"
      >
        {{ t("lore.filters.all", { count: anomalies.length }) }}
      </button>
      <button
        :class="['filter-chip critical', { active: selectedFilter === 'CRITICAL' }]"
        @click="selectedFilter = 'CRITICAL'"
      >
        {{ t("lore.filters.critical") }}
      </button>
      <button
        :class="['filter-chip major', { active: selectedFilter === 'MAJOR' }]"
        @click="selectedFilter = 'MAJOR'"
      >
        {{ t("lore.filters.major") }}
      </button>
      <button
        :class="['filter-chip minor', { active: selectedFilter === 'MINOR' }]"
        @click="selectedFilter = 'MINOR'"
      >
        {{ t("lore.filters.minor") }}
      </button>
    </div>

    <div class="lore-anomalies-feed">
      <div v-for="(anom, i) in filteredAnomalies" :key="anom.id || i" class="anomaly-card" :class="anom.severity.toLowerCase()">
        <div class="anom-top">
          <div class="anom-title-group">
            <span class="severity-tag">{{ anom.severity }}</span>
            <span class="anom-title">{{ anom.entityTitle }}</span>
          </div>
          <button class="btn-dismiss-anom" :title="t('common.accept')" @click="handleDismiss(anom)">✕</button>
        </div>

        <div class="anom-conflict">{{ anom.conflictType }}</div>

        <div class="anom-diff-box">
          <div class="diff-line text-claim">
            <span class="diff-prefix">{{ t("lore.textPrefix") }}</span>
            <span>{{ anom.claimInText }}</span>
          </div>
          <div class="diff-line lore-fact">
            <span class="diff-prefix">{{ t("lore.canonPrefix") }}</span>
            <span>{{ anom.originalLoreFact }}</span>
          </div>
        </div>

        <div v-if="anom.suggestedFix" class="anom-fix">
          <span class="fix-label">{{ t("lore.suggestionLabel") }}</span> {{ anom.suggestedFix }}
        </div>
      </div>

      <div v-if="filteredAnomalies.length === 0" class="anom-empty">
        <span class="empty-icon">✓</span>
        <span>{{ isAnalyzing ? t("lore.analyzing") : t("lore.cleanState") }}</span>
      </div>
    </div>

    <div class="lore-semantic-box">
      <div class="feed-heading">{{ t("lore.semanticTitle") }}</div>
      <div class="semantic-search-row">
        <input
          v-model="semanticQuery"
          class="semantic-input"
          :placeholder="t('lore.semanticPlaceholder')"
          @keydown.enter="handleSearch"
        />
        <button class="btn-semantic-go" @click="handleSearch">{{ t("lore.searchBtn") }}</button>
      </div>

      <div class="semantic-results-feed">
        <div
          v-for="res in semanticResults"
          :key="res.id"
          class="semantic-res-card clickable"
          @click="emit('selectStoryById', res.id)"
        >
          <div class="res-top">
            <span class="res-title">{{ res.title }}</span>
            <span class="res-sim">{{ Math.round(res.similarity * 100) }}%</span>
          </div>
          <div class="res-snippet">{{ res.snippet }}</div>
          <span class="res-open-hint">{{ t("lore.jumpToScene") }}</span>
        </div>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.lore-drawer {
  width: min(100%, 370px);
  background: var(--bg-surface-solid);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  padding: 16px;
  gap: 12px;
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  z-index: 35;
  overflow-y: auto;
  overscroll-behavior-y: contain;
  box-shadow: -10px 0 35px rgba(0, 0, 0, 0.45);
}

.drawer-header-strip {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.lore-sigil-orb {
  color: var(--accent-cyan);
  font-size: 14px;
}

.drawer-header-strip h4 {
  font-size: 13px;
  font-weight: 800;
  color: var(--text-pure);
}

.btn-drawer-exit {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  width: 30px;
  height: 30px;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 13px;
}

.lore-score-widget {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.widget-head {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  font-weight: 700;
  color: var(--text-pure);
}

.widget-head b {
  color: var(--accent-emerald);
  font-family: var(--font-mono);
}

.widget-head b.warning { color: var(--accent-gold); }
.widget-head b.danger { color: var(--accent-rose); }

.score-track {
  height: 6px;
  background: var(--border-subtle);
  border-radius: 999px;
  overflow: hidden;
}

.score-bar {
  height: 100%;
  background: linear-gradient(90deg, var(--accent-indigo) 0%, var(--accent-cyan) 100%);
  border-radius: 999px;
  transition: width 0.3s ease;
}

.score-meta {
  display: flex;
  justify-content: space-between;
  font-size: 10px;
  color: var(--text-muted);
  font-family: var(--font-mono);
}

.severity-filter-row {
  display: flex;
  gap: 4px;
  overflow-x: auto;
  padding-bottom: 2px;
}

.filter-chip {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-muted);
  padding: 5px 8px;
  border-radius: 6px;
  font-size: 10px;
  font-weight: 700;
  cursor: pointer;
  white-space: nowrap;
}

.filter-chip.active {
  background: var(--accent-indigo);
  color: #fff;
  border-color: var(--accent-indigo);
}

.filter-chip.critical.active {
  background: var(--accent-rose);
  border-color: var(--accent-rose);
}

.lore-anomalies-feed {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.anomaly-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-left: 3px solid var(--accent-indigo);
  border-radius: 10px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.anomaly-card.critical {
  border-left-color: var(--accent-rose);
  background: rgba(244, 63, 94, 0.05);
}

.anomaly-card.major {
  border-left-color: var(--accent-gold);
}

.anom-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.anom-title-group {
  display: flex;
  align-items: center;
  gap: 6px;
}

.severity-tag {
  font-family: var(--font-mono);
  font-size: 9px;
  font-weight: 800;
  padding: 1px 4px;
  border-radius: 4px;
  background: var(--border);
  color: var(--text-pure);
}

.anom-title {
  font-size: 12px;
  font-weight: 800;
  color: var(--text-pure);
}

.btn-dismiss-anom {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
}

.anom-conflict {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-main);
}

.anom-diff-box {
  background: var(--bg-input);
  border: 1px solid var(--border-subtle);
  border-radius: 6px;
  padding: 6px 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 10px;
  font-family: var(--font-mono);
}

.text-claim { color: var(--accent-rose); }
.lore-fact { color: var(--accent-emerald); }
.diff-prefix { font-weight: 700; margin-right: 4px; }

.anom-fix {
  font-size: 10px;
  color: var(--text-muted);
}

.fix-label {
  font-weight: 700;
  color: var(--accent-cyan);
}

.anom-empty {
  padding: 24px 10px;
  text-align: center;
  color: var(--text-dim);
  font-size: 11px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}

.empty-icon {
  color: var(--accent-emerald);
  font-size: 18px;
}

.lore-semantic-box {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 4px;
  border-top: 1px solid var(--border);
  padding-top: 10px;
}

.feed-heading {
  font-size: 11px;
  font-weight: 800;
  color: var(--text-muted);
  text-transform: uppercase;
}

.semantic-search-row {
  display: flex;
  gap: 6px;
}

.semantic-input {
  flex: 1;
  min-width: 0;
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-pure);
  border-radius: 8px;
  padding: 8px 10px;
  font-size: 12px;
  outline: none;
}

.btn-semantic-go {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  border-radius: 8px;
  padding: 0 12px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}

.semantic-results-feed {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.semantic-res-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 3px;
  cursor: pointer;
}

.semantic-res-card:hover {
  border-color: var(--accent-cyan);
}

.res-top {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  font-weight: 700;
}

.res-sim {
  color: var(--accent-cyan);
  font-family: var(--font-mono);
  font-size: 10px;
}

.res-snippet {
  font-size: 10px;
  color: var(--text-muted);
}

.res-open-hint {
  font-size: 9px;
  color: var(--accent-cyan);
  font-weight: 700;
}

@media (max-width: 900px) {
  .lore-drawer {
    width: 100%;
    border-left: none;
    border-radius: 14px;
  }
}
</style>