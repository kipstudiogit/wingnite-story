<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { computed } from "vue";
import type { TextMetricsResult } from "../../types";
import { useI18n } from "../../i18n";

const { metrics } = defineProps<{
  metrics: TextMetricsResult;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const { t, messages } = useI18n();

const fleschLabel = computed(() => {
  const f = metrics.fleschReadingEase;
  if (f >= 80) return t("analytics.fleschEasy");
  if (f >= 60) return t("analytics.fleschOptimal");
  if (f >= 45) return t("analytics.fleschLiterary");
  return t("analytics.fleschDense");
});

const toneLabel = computed(() => {
  return messages.value.analytics.tones[metrics.sentimentToneKey] ?? messages.value.analytics.tones.neutral;
});
</script>

<template>
  <aside class="analytics-drawer">
    <header class="drawer-header-strip">
      <h4>{{ t("analytics.title") }}</h4>
      <button class="btn-drawer-exit" @click="emit('close')">✕</button>
    </header>

    <div class="analytics-grid">
      <div class="analytic-stat-card">
        <span class="stat-caption">{{ t("analytics.wordsAndChars") }}</span>
        <span class="stat-value">{{ metrics.wordCount }} / {{ metrics.charCount }}</span>
      </div>
      <div class="analytic-stat-card">
        <span class="stat-caption">{{ t("analytics.paragraphs") }}</span>
        <span class="stat-value">{{ metrics.paragraphCount }}</span>
      </div>
      <div class="analytic-stat-card">
        <span class="stat-caption">{{ t("analytics.dialogues") }}</span>
        <span class="stat-value">{{ metrics.dialogueRatio.toFixed(1) }}%</span>
      </div>
      <div class="analytic-stat-card">
        <span class="stat-caption">{{ t("analytics.vocabulary") }}</span>
        <span class="stat-value">{{ metrics.vocabularyRichness.toFixed(1) }}%</span>
      </div>
      <div class="analytic-stat-card">
        <span class="stat-caption">{{ t("analytics.cadence") }}</span>
        <span class="stat-value">{{ metrics.sentenceCadence.toFixed(1) }}</span>
      </div>
      <div class="analytic-stat-card">
        <span class="stat-caption">{{ t("analytics.readingTime") }}</span>
        <span class="stat-value">~{{ metrics.readingTimeMinutes.toFixed(1) }} {{ t("analytics.minutesUnit") }}</span>
      </div>
    </div>

    <div class="readability-index-box">
      <div class="index-head">
        <span>{{ t("analytics.fleschTitle") }}</span>
        <b>{{ Math.round(metrics.fleschReadingEase) }} / 100</b>
      </div>
      <div class="index-track">
        <div class="index-fill" :style="{ width: `${metrics.fleschReadingEase}%` }"></div>
      </div>
      <span class="index-desc">{{ fleschLabel }}</span>
    </div>

    <div class="pacing-sparkline-box">
      <div class="sparkline-head">
        <span>{{ t("analytics.pacingTitle") }}</span>
        <span class="sparkline-val">{{ toneLabel }}</span>
      </div>
      <div class="pacing-bars">
        <div
          v-for="(val, idx) in metrics.pacingTimeline"
          :key="idx"
          class="pacing-bar-col"
        >
          <div class="pacing-bar-fill" :style="{ height: `${Math.max(15, val)}%` }"></div>
        </div>
      </div>
    </div>

    <div v-if="metrics.clicheWarnings.length > 0" class="cliche-block">
      <span class="warnings-heading">{{ t("analytics.clichesTitle", { count: metrics.clicheWarnings.length }) }}</span>
      <div class="warning-tags">
        <span v-for="c in metrics.clicheWarnings" :key="c" class="cliche-tag">{{ c }}</span>
      </div>
    </div>

    <div class="warnings-block">
      <span class="warnings-heading">{{ t("analytics.repetitionsTitle") }}</span>
      <div class="warning-tags">
        <span v-for="w in metrics.repetitionWarnings" :key="w" class="rep-tag">{{ w }}</span>
        <span v-if="metrics.repetitionWarnings.length === 0" class="rep-empty">{{ t("analytics.repetitionsEmpty") }}</span>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.analytics-drawer {
  width: min(100%, 360px);
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

.analytics-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.analytic-stat-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  padding: 8px 10px;
  border-radius: 10px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.stat-caption {
  font-size: 9px;
  font-weight: 700;
  color: var(--text-dim);
  text-transform: uppercase;
}

.stat-value {
  font-family: var(--font-mono);
  font-size: 14px;
  font-weight: 800;
  color: var(--accent-cyan);
}

.readability-index-box {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.index-head {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  font-weight: 700;
  color: var(--text-pure);
}

.index-track {
  height: 5px;
  background: var(--border-subtle);
  border-radius: 999px;
  overflow: hidden;
}

.index-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--accent-indigo) 0%, var(--accent-cyan) 100%);
  border-radius: 999px;
}

.index-desc {
  font-size: 10px;
  color: var(--text-muted);
}

.pacing-sparkline-box {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.sparkline-head {
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: 10px;
  font-weight: 700;
  color: var(--text-muted);
}

.sparkline-val {
  color: var(--accent-gold);
  font-family: var(--font-mono);
}

.pacing-bars {
  display: flex;
  align-items: flex-end;
  height: 48px;
  gap: 4px;
  background: var(--bg-input);
  border-radius: 6px;
  padding: 4px 6px;
}

.pacing-bar-col {
  flex: 1;
  height: 100%;
  display: flex;
  align-items: flex-end;
}

.pacing-bar-fill {
  width: 100%;
  background: linear-gradient(180deg, var(--accent-cyan) 0%, var(--accent-indigo) 100%);
  border-radius: 3px 3px 0 0;
  transition: height 0.3s ease;
}

.cliche-block,
.warnings-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.warnings-heading {
  font-size: 10px;
  font-weight: 800;
  color: var(--text-muted);
  text-transform: uppercase;
}

.warning-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.cliche-tag {
  background: rgba(245, 158, 11, 0.12);
  border: 1px solid rgba(245, 158, 11, 0.3);
  color: var(--accent-gold);
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 6px;
}

.rep-tag {
  background: rgba(244, 63, 94, 0.12);
  border: 1px solid rgba(244, 63, 94, 0.25);
  color: var(--accent-rose);
  font-size: 10px;
  font-family: var(--font-mono);
  padding: 2px 6px;
  border-radius: 6px;
}

.rep-empty {
  font-size: 11px;
  color: var(--text-dim);
  padding: 12px 0;
}

@media (max-width: 900px) {
  .analytics-drawer {
    width: 100%;
    border-left: none;
    border-radius: 14px;
  }
}
</style>