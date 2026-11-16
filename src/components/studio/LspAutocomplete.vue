<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import type { LspSuggestion } from "../../types";
import { useI18n } from "../../i18n";

const { suggestions, selectedIndex } = defineProps<{
  suggestions: LspSuggestion[];
  selectedIndex: number;
}>();

const emit = defineEmits<{
  (e: "select", suggestion: LspSuggestion): void;
  (e: "hoverIndex", index: number): void;
}>();

const { t } = useI18n();
</script>

<template>
  <div class="lsp-autocomplete-popup" @mousedown.prevent>
    <div class="lsp-popup-header">{{ t("lsp.popupHeader") }}</div>
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
  top: 56px;
  left: 24px;
  width: min(calc(100% - 48px), 380px);
  background: var(--bg-surface-solid);
  border: 1px solid var(--accent-indigo);
  border-radius: 12px;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.75);
  z-index: 40;
  overflow: hidden;
}

.lsp-popup-header {
  padding: 8px 12px;
  background: var(--bg-card);
  border-bottom: 1px solid var(--border);
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--accent-cyan);
}

.lsp-items-list {
  max-height: 220px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.lsp-item-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
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
  background: var(--bg-input);
  color: var(--accent-cyan);
  padding: 2px 6px;
  border-radius: 4px;
  flex-shrink: 0;
}

.lsp-item-row.active .sug-badge,
.lsp-item-row:hover .sug-badge {
  background: rgba(0, 0, 0, 0.25);
  color: #fff;
}

.sug-title {
  font-size: 13px;
  font-weight: 700;
  flex-shrink: 0;
}

.sug-preview {
  font-size: 11px;
  color: var(--text-dim);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.lsp-item-row.active .sug-preview,
.lsp-item-row:hover .sug-preview {
  color: rgba(255, 255, 255, 0.85);
}

@media (max-width: 900px) {
  .lsp-autocomplete-popup {
    top: 10px;
    left: 10px;
    width: calc(100% - 20px);
  }
}
</style>