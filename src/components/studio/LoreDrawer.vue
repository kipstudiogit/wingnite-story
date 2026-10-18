<script setup lang="ts">
import { ref } from "vue";
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

function handleSearch() {
  emit("searchSemantic", semanticQuery.value);
}
</script>

<template>
  <aside class="lore-drawer">
    <header class="drawer-header-strip">
      <h4>Страж канона и Lore-поиск</h4>
      <button class="btn-drawer-exit" @click="emit('close')">✕</button>
    </header>

    <div class="lore-score-widget">
      <div class="widget-head">
        <span>Согласованность мира</span>
        <b>{{ Math.round(consistencyScore) }}%</b>
      </div>
      <div class="score-track">
        <div class="score-bar" :style="{ width: `${consistencyScore}%` }"></div>
      </div>
    </div>

    <div class="lore-anomalies-feed">
      <div class="feed-heading">Обнаруженные аномалии ({{ anomalies.length }})</div>
      <div v-for="(anom, i) in anomalies" :key="i" class="anomaly-card">
        <div class="anom-top">
          <span class="anom-title">{{ anom.entityTitle }}</span>
          <button class="btn-dismiss-anom" @click="emit('dismissAnomaly', i)">✕</button>
        </div>
        <div class="anom-conflict">{{ anom.conflictType }}</div>
        <div class="anom-claim">{{ anom.claimInText }}</div>
        <div class="anom-fact">{{ anom.originalLoreFact }}</div>
      </div>
      <div v-if="anomalies.length === 0" class="anom-empty">
        {{ isAnalyzing ? 'Анализ канона...' : 'Противоречий не обнаружено' }}
      </div>
    </div>

    <div class="lore-semantic-box">
      <div class="feed-heading">Семантический векторный поиск</div>
      <div class="semantic-search-row">
        <input
          v-model="semanticQuery"
          class="semantic-input"
          placeholder="Поиск по скрытому смыслу..."
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
            <span class="res-sim">{{ Math.round(res.similarity * 100) }}% совп.</span>
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
  width: 340px;
  background: var(--bg-surface-solid);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  padding: 16px;
  border-radius: 0 16px 16px 0;
  gap: 14px;
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  z-index: 25;
  overflow-y: auto;
}

.drawer-header-strip {
  display: flex;
  justify-content: space-between;
  align-items: center;
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
  font-size: 12px;
  font-weight: 700;
}

.score-track {
  height: 6px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 999px;
  overflow: hidden;
}

.score-bar {
  height: 100%;
  background: linear-gradient(90deg, var(--accent-indigo) 0%, var(--accent-cyan) 100%);
  border-radius: 999px;
}

.lore-anomalies-feed {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.feed-heading {
  font-size: 11px;
  font-weight: 800;
  color: var(--text-muted);
  text-transform: uppercase;
}

.anomaly-card {
  background: rgba(244, 63, 94, 0.08);
  border: 1px solid rgba(244, 63, 94, 0.3);
  border-radius: 10px;
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.anom-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.anom-title {
  font-size: 12px;
  font-weight: 800;
  color: var(--accent-rose);
}

.btn-dismiss-anom {
  background: transparent;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
}

.anom-conflict {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-pure);
}

.anom-claim,
.anom-fact {
  font-size: 10px;
  color: var(--text-muted);
}

.anom-empty {
  font-size: 11px;
  color: var(--text-dim);
  text-align: center;
  padding: 20px 0;
}

.lore-semantic-box {
  display: flex;
  flex-direction: column;
  gap: 8px;
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
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  transition: all 0.2s ease;
}

.semantic-res-card.clickable {
  cursor: pointer;
}

.semantic-res-card.clickable:hover {
  border-color: var(--accent-cyan);
  background: var(--bg-card-hover);
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
}

.res-snippet {
  font-size: 10px;
  color: var(--text-muted);
}

.res-open-hint {
  font-size: 9px;
  font-weight: 700;
  color: var(--accent-cyan);
  margin-top: 2px;
}
</style>