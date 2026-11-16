<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { useI18n } from "../../i18n";

const { isSyncing = false } = defineProps<{
  isSyncing?: boolean;
}>();

const emit = defineEmits<{
  (e: "insert", prefix: string, suffix: string, placeholder?: string): void;
  (e: "save"): void;
}>();

const { t, messages } = useI18n();
</script>

<template>
  <div class="mobile-keyboard-accessory-bar" @mousedown.prevent @touchstart.prevent>
    <div class="accessory-scroll-track">
      <button class="acc-btn" @click="emit('insert', '\n# ', '\n', messages.studio.toolbar.h1Sample)">{{ messages.studio.toolbar.h1 }}</button>
      <button class="acc-btn" @click="emit('insert', '\n## ', '\n', messages.studio.toolbar.h2Sample)">{{ messages.studio.toolbar.h2 }}</button>
      <button class="acc-btn" @click="emit('insert', '\n### ', '\n', messages.studio.toolbar.h3Sample)">{{ messages.studio.toolbar.h3 }}</button>
      <button class="acc-btn font-bold" @click="emit('insert', '**', '**', messages.studio.toolbar.boldSample)">{{ messages.studio.toolbar.bold }}</button>
      <button class="acc-btn font-italic" @click="emit('insert', '*', '*', messages.studio.toolbar.italicSample)">{{ messages.studio.toolbar.italic }}</button>
      <button class="acc-btn" @click="emit('insert', '~~', '~~', messages.studio.toolbar.strikeSample)">{{ messages.studio.toolbar.strike }}</button>
      <button class="acc-btn accent-btn" @click="emit('insert', '[[', ']]', messages.studio.toolbar.wikiSample)">{{ messages.studio.toolbar.wiki }}</button>
      <button class="acc-btn accent-btn" @click="emit('insert', '@', ' ', messages.studio.toolbar.mentionSample)">{{ messages.studio.toolbar.mention }}</button>
      <button class="acc-btn highlight-btn" @click="emit('insert', '==', '==', messages.studio.toolbar.highlightSample)">{{ messages.studio.toolbar.highlight }}</button>
      <button class="acc-btn spoiler-btn" @click="emit('insert', '||', '||', messages.studio.toolbar.spoilerSample)">{{ messages.studio.toolbar.spoiler }}</button>
      <button class="acc-btn" @click="emit('insert', '\n> «', '»\n', messages.studio.toolbar.quoteSample)">« »</button>
      <button class="acc-btn" @click="emit('insert', '\n- [ ] ', '\n', messages.studio.toolbar.checklistSample)">{{ messages.studio.toolbar.checklist }}</button>
      <button class="acc-btn" @click="emit('insert', '\n- ', '\n', messages.studio.toolbar.listSample)">{{ messages.studio.toolbar.list }}</button>
      <button class="acc-btn" @click="emit('insert', '`', '`', messages.studio.toolbar.codeSample)">&lt;/&gt;</button>
      <button class="acc-btn" @click="emit('insert', '\n\n---\n\n', '', '')">{{ messages.studio.toolbar.asterism }}</button>
    </div>

    <button
      class="acc-save-btn"
      :disabled="isSyncing"
      @click="emit('save')"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
        <polyline points="17 21 17 13 7 13 7 21"/>
        <polyline points="7 3 7 8 15 8"/>
      </svg>
      <span>{{ isSyncing ? "..." : t("mobile.saveShort") }}</span>
    </button>
  </div>
</template>

<style scoped>
.mobile-keyboard-accessory-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-surface-solid);
  border-top: 1px solid var(--border);
  padding: 6px max(10px, var(--safe-right)) 6px max(10px, var(--safe-left));
  gap: 8px;
  width: 100%;
  flex-shrink: 0;
  z-index: 45;
  touch-action: pan-x;
}

.accessory-scroll-track {
  display: flex;
  align-items: center;
  gap: 6px;
  overflow-x: auto;
  flex: 1;
  padding-bottom: 2px;
  -webkit-overflow-scrolling: touch;
}

.acc-btn {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-main);
  border-radius: 9px;
  min-width: 38px;
  height: 38px;
  padding: 0 10px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  white-space: nowrap;
  display: flex;
  align-items: center;
  justify-content: center;
  user-select: none;
  touch-action: manipulation;
}

.acc-btn:active {
  background: var(--accent-indigo);
  color: #fff;
  transform: scale(0.95);
}

.font-bold {
  font-weight: 900;
}

.font-italic {
  font-style: italic;
}

.accent-btn {
  color: var(--accent-cyan);
  border-color: var(--accent-cyan-glow);
}

.highlight-btn {
  color: var(--accent-gold);
  border-color: var(--accent-gold-glow);
}

.spoiler-btn {
  color: var(--accent-purple);
  border-color: var(--accent-purple-glow);
}

.acc-save-btn {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  border-radius: 9px;
  height: 38px;
  padding: 0 12px;
  font-size: 11px;
  font-weight: 800;
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  cursor: pointer;
  user-select: none;
}

.acc-save-btn:disabled {
  opacity: 0.6;
}
</style>