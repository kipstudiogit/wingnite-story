<script setup lang="ts">
import type { LspSuggestion } from "../../types";

const { suggestions, selectedIndex } = defineProps<{
  suggestions: LspSuggestion[];
  selectedIndex: number;
}>();

const emit = defineEmits<{
  (e: "select", suggestion: LspSuggestion): void;
  (e: "hoverIndex", index: number): void;
}>();
</script>

<template>
  <div class="lsp-autocomplete-popup" @mousedown.prevent>
    <div class="lsp-popup-header">LSP Сущности и Реликвии</div>
    <div class="lsp-items-list">
      <div
        v-for="(sug, idx) in suggestions"
        :key="sug.id"
        :class="['lsp-item-row', { active: idx === selectedIndex }]"
        @mouseenter="emit('hoverIndex', idx)"
        @click="emit('select', sug)"
      >
        <span class="sug-badge">{{ sug.category }}</span>
        <span class="sug-title">{{ sug.title }}</span>
        <span class="sug-preview">{{ sug.preview }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.lsp-autocomplete-popup {
  position: absolute;
  top: 70px;
  left: 60px;
  width: 380px;
  background: var(--bg-surface-solid);
  border: 1px solid var(--accent-indigo);
  border-radius: 12px;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.8);
  z-index: 40;
  overflow: hidden;
}

.lsp-popup-header {
  padding: 8px 12px;
  background: rgba(99, 102, 241, 0.15);
  border-bottom: 1px solid var(--border);
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--accent-cyan);
}

.lsp-items-list {
  max-height: 240px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.lsp-item-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  border-bottom: 1px solid var(--border-subtle);
}

.lsp-item-row.active,
.lsp-item-row:hover {
  background: var(--accent-indigo);
  color: #fff;
}

.sug-badge {
  font-size: 9px;
  font-weight: 800;
  background: rgba(0, 0, 0, 0.3);
  padding: 2px 6px;
  border-radius: 4px;
}

.sug-title {
  font-size: 13px;
  font-weight: 700;
}

.sug-preview {
  font-size: 11px;
  color: var(--text-dim);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>