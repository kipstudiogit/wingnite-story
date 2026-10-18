<script setup lang="ts">
import type { TextMetricsResult } from "../../types";

const { metrics } = defineProps<{
  metrics: TextMetricsResult;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();
</script>

<template>
  <aside class="analytics-drawer">
    <header class="drawer-header-strip">
      <h4>Лингвистическая телеметрия</h4>
      <button class="btn-drawer-exit" @click="emit('close')">✕</button>
    </header>

    <div class="analytics-grid">
      <div class="analytic-stat-card">
        <span class="stat-caption">Диалоги</span>
        <span class="stat-value">{{ metrics.dialogueRatio.toFixed(1) }}%</span>
      </div>
      <div class="analytic-stat-card">
        <span class="stat-caption">Лексика</span>
        <span class="stat-value">{{ metrics.vocabularyRichness.toFixed(1) }}%</span>
      </div>
      <div class="analytic-stat-card">
        <span class="stat-caption">Ритм</span>
        <span class="stat-value">{{ metrics.sentenceCadence.toFixed(1) }}</span>
      </div>
      <div class="analytic-stat-card">
        <span class="stat-caption">Чтение</span>
        <span class="stat-value">~{{ metrics.readingTimeMinutes.toFixed(1) }} м</span>
      </div>
    </div>

    <div class="warnings-block">
      <span class="warnings-heading">Аномалии частотности</span>
      <div class="warning-tags">
        <span v-for="w in metrics.repetitionWarnings" :key="w" class="rep-tag">{{ w }}</span>
        <span v-if="metrics.repetitionWarnings.length === 0" class="rep-empty">Повторы в пределах допуска</span>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.analytics-drawer {
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

.analytics-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.analytic-stat-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  padding: 10px;
  border-radius: 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-caption {
  font-size: 10px;
  font-weight: 700;
  color: var(--text-dim);
  text-transform: uppercase;
}

.stat-value {
  font-family: var(--font-mono);
  font-size: 15px;
  font-weight: 800;
  color: var(--accent-cyan);
}

.warnings-block {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.warnings-heading {
  font-size: 11px;
  font-weight: 800;
  color: var(--text-muted);
  text-transform: uppercase;
}

.warning-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.rep-tag {
  background: rgba(244, 63, 94, 0.15);
  border: 1px solid rgba(244, 63, 94, 0.3);
  color: var(--accent-rose);
  font-size: 11px;
  font-family: var(--font-mono);
  padding: 3px 8px;
  border-radius: 6px;
}

.rep-empty {
  font-size: 11px;
  color: var(--text-dim);
  text-align: center;
  padding: 20px 0;
}
</style>