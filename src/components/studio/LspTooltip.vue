<script setup lang="ts">
import type { LspHoverInfo } from "../../types";

const { hoverInfo, coordinates } = defineProps<{
  hoverInfo: LspHoverInfo;
  coordinates: { x: number; y: number };
}>();
</script>

<template>
  <div
    class="lsp-hover-tooltip"
    :style="{ top: `${coordinates.y}px`, left: `${coordinates.x}px` }"
  >
    <div class="hover-head">
      <span class="hover-category">{{ hoverInfo.category }}</span>
      <h4>{{ hoverInfo.title }}</h4>
    </div>
    <p class="hover-summary">{{ hoverInfo.summary }}</p>
    <div v-if="hoverInfo.connectedEntities.length > 0" class="hover-relations">
      <span class="rel-tag-label">Связи:</span>
      <span v-for="rel in hoverInfo.connectedEntities" :key="rel" class="rel-tag">{{ rel }}</span>
    </div>
  </div>
</template>

<style scoped>
.lsp-hover-tooltip {
  position: fixed;
  width: 280px;
  background: var(--bg-surface-solid);
  border: 1px solid var(--accent-cyan);
  border-radius: 12px;
  padding: 12px;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.9);
  z-index: 50;
  pointer-events: none;
}

.hover-category {
  font-size: 9px;
  font-weight: 800;
  color: var(--accent-cyan);
  text-transform: uppercase;
}

.hover-head h4 {
  font-size: 14px;
  font-weight: 800;
  margin-top: 2px;
}

.hover-summary {
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1.4;
  margin: 6px 0;
}

.hover-relations {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  align-items: center;
}

.rel-tag-label {
  font-size: 9px;
  font-weight: 700;
  color: var(--text-dim);
}

.rel-tag {
  font-size: 9px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  padding: 2px 5px;
  border-radius: 4px;
}
</style>