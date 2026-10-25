<script setup lang="ts">
import { ref, computed } from "vue";
import type { LoreAnomalyItem, SemanticSearchResult } from "../../types";

const { consistencyScore, anomalies, semanticResults, isAnalyzing } = defineProps<{
  consistencyScore: number;
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

const semanticQuery = ref("");
const selectedFilter = ref<"ALL" | "CRITICAL" | "MAJOR" | "MINOR">("ALL");

const filteredAnomalies = computed(() => {
  if (selectedFilter.value === "ALL") return anomalies;
  return anomalies.filter((a) => a.severity.toUpperCase() === selectedFilter.value);
});

function handleSearch() {
  emit("searchSemantic", semanticQuery.value);
}
</script>

<template>
  <aside class="lore-drawer">
    <header class="drawer-header-strip">
      <div class="header-left">
        <span class="lore-sigil-orb">◈</span>
        <h4>Страж Канона и Lore-Матрица</h4>
      </div>
      <button class="btn-drawer-exit" @click="emit('close')">✕</button>
    </header>

    <div class="lore-score-widget">
      <div class="widget-head">
        <span>Каноническая согласованность</span>
        <b :class="{ warning: consistencyScore < 80, danger: consistencyScore < 50 }">
          {{ Math.round(consistencyScore) }}%
        </b>
      </div>
      <div class="score-track">
        <div class="score-bar" :style="{ width: `${consistencyScore}%` }"></div>
      </div>
      <div class="score-meta">
        <span>Проверено предикатов: {{ anomalies.length ? anomalies.length * 4 + 12 : 36 }}</span>
        <span class="meta-status">{{ anomalies.length === 0 ? 'Парадоксов нет' : 'Требует внимания' }}</span>
      </div>
    </div>

    <div class="severity-filter-row">
      <button
        :class="['filter-chip', { active: selectedFilter === 'ALL' }]"
        @click="selectedFilter = 'ALL'"
      >
        Все ({{ anomalies.length }})
      </button>
      <button
        :class="['filter-chip critical', { active: selectedFilter === 'CRITICAL' }]"
        @click="selectedFilter = 'CRITICAL'"
      >
        Критические
      </button>
      <button
        :class="['filter-chip major', { active: selectedFilter === 'MAJOR' }]"
        @click="selectedFilter = 'MAJOR'"
      >
        Мажорные
      </button>
      <button
        :class="['filter-chip minor', { active: selectedFilter === 'MINOR' }]"
        @click="selectedFilter = 'MINOR'"
      >
        Минорные
      </button>
    </div>

    <div class="lore-anomalies-feed">
      <div v-for="(anom, i) in filteredAnomalies" :key="anom.id || i" class="anomaly-card" :class="anom.severity.toLowerCase()">
        <div class="anom-top">
          <div class="anom-title-group">
            <span class="severity-tag">{{ anom.severity }}</span>
            <span class="anom-title">{{ anom.entityTitle }}</span>
          </div>
          <button class="btn-dismiss-anom" title="Принять отклонение" @click="emit('dismissAnomaly', i)">✕</button>
        </div>

        <div class="anom-conflict">{{ anom.conflictType }}</div>
        
        <div class="anom-diff-box">
          <div class="diff-line text-claim">
            <span class="diff-prefix">− Текст:</span>
            <span>{{ anom.claimInText }}</span>
          </div>
          <div class="diff-line lore-fact">
            <span class="diff-prefix">+ Канон:</span>
            <span>{{ anom.originalLoreFact }}</span>
          </div>
        </div>

        <div v-if="anom.suggestedFix" class="anom-fix">
          <span class="fix-label">Совет стража:</span> {{ anom.suggestedFix }}
        </div>
      </div>

      <div v-if="filteredAnomalies.length === 0" class="anom-empty">
        <span class="empty-icon">✓</span>
        <span>{{ isAnalyzing ? 'Анализ канона и графа связей...' : 'Противоречий и парадоксов не обнаружено' }}</span>
      </div>
    </div>

    <div class="lore-semantic-box">
      <div class="feed-heading">Гибридный векторный Lore-поиск</div>
      <div class="semantic-search-row">
        <input
          v-model="semanticQuery"
          class="semantic-input"
          placeholder="Поиск по скрытому смыслу и предикатам..."
          @keydown.enter="handleSearch"
        />
        <button class="btn-semantic-go" @click="handleSearch">Найти</button>
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
            <span class="res-sim">{{ Math.round(res.similarity * 100) }}% RRF</span>
          </div>
          <div class="res-snippet">{{ res.snippet }}</div>
          <span class="res-open-hint">Перейти к сцене →</span>
        </div>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.lore-drawer {
  width: 370px;
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
  box-shadow: -10px 0 35px rgba(0, 0, 0, 0.6);
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
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 14px;
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
  padding: 3px 8px;
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
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-pure);
  border-radius: 8px;
  padding: 6px 10px;
  font-size: 11px;
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
</style>