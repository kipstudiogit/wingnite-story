<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { computed } from "vue";
import type { TimelineEventItem } from "../../types";
import { useI18n } from "../../i18n";

const { events, selectedEventId } = defineProps<{
  events: TimelineEventItem[];
  selectedEventId: string | null;
}>();

const emit = defineEmits<{
  (e: "select", ev: TimelineEventItem): void;
}>();

const { t } = useI18n();

const tensionPathData = computed(() => {
  if (events.length === 0) return "";
  if (events.length === 1) {
    const y = 110 - (events[0].intensity / 100) * 90;
    return `M 20 ${y} L 980 ${y}`;
  }
  return events.reduce((acc, ev, idx) => {
    const x = (idx / (events.length - 1)) * 960 + 20;
    const y = 110 - (ev.intensity / 100) * 90;
    return `${acc} L ${x} ${y}`;
  }, `M 20 ${110 - (events[0].intensity / 100) * 90}`);
});

const tensionFillData = computed(() => {
  if (!tensionPathData.value || events.length === 0) return "";
  return `${tensionPathData.value} L 980 120 L 20 120 Z`;
});
</script>

<template>
  <div class="tension-graph-card">
    <div class="graph-legend">
      <span>{{ t("timeline.tensorTitle") }}</span>
      <span class="legend-scale">{{ t("timeline.tensorScale") }}</span>
    </div>

    <div class="tension-svg-wrap">
      <svg class="tension-svg" viewBox="0 0 1000 120" preserveAspectRatio="none">
        <defs>
          <linearGradient id="tensorGrad" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stop-color="#6366f1" stop-opacity="0.9" />
            <stop offset="50%" stop-color="#06b6d4" stop-opacity="0.95" />
            <stop offset="100%" stop-color="#f59e0b" stop-opacity="1" />
          </linearGradient>
          <linearGradient id="tensorFill" x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" stop-color="#6366f1" stop-opacity="0.35" />
            <stop offset="100%" stop-color="#6366f1" stop-opacity="0" />
          </linearGradient>
        </defs>

        <path v-if="tensionFillData" :d="tensionFillData" fill="url(#tensorFill)" />
        <path
          v-if="tensionPathData"
          :d="tensionPathData"
          fill="none"
          stroke="url(#tensorGrad)"
          stroke-width="3"
          stroke-linecap="round"
        />

        <circle
          v-for="(ev, idx) in events"
          :key="ev.id"
          :cx="(idx / Math.max(1, events.length - 1)) * 960 + 20"
          :cy="110 - (ev.intensity / 100) * 90"
          r="6"
          :class="['tension-pulse-dot', { active: selectedEventId === ev.id }]"
          @click="emit('select', ev)"
        />
      </svg>
    </div>
  </div>
</template>

<style scoped>
.tension-graph-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 18px 24px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.graph-legend {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--text-muted);
}

.legend-scale {
  color: var(--text-dim);
  font-family: var(--font-mono);
}

.tension-svg-wrap {
  width: 100%;
  height: 120px;
}

.tension-svg {
  width: 100%;
  height: 100%;
  overflow: visible;
}

.tension-pulse-dot {
  fill: #fff;
  stroke: var(--accent-indigo);
  stroke-width: 3px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tension-pulse-dot:hover,
.tension-pulse-dot.active {
  r: 8;
  stroke: var(--accent-cyan);
  fill: var(--accent-cyan);
}
</style>