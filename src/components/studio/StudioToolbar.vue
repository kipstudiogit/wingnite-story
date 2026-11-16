<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { useI18n } from "../../i18n";

const emit = defineEmits<{
  (e: "insert", prefix: string, suffix: string, placeholder?: string): void;
}>();

const { messages } = useI18n();
</script>

<template>
  <div class="quantum-tools-strip">
    <div class="tools-horizontal-lane">
      <button class="tool-chip-btn" @mousedown.prevent="emit('insert', '\n# ', '\n', messages.studio.toolbar.h1Sample)">{{ messages.studio.toolbar.h1 }}</button>
      <button class="tool-chip-btn" @mousedown.prevent="emit('insert', '\n## ', '\n', messages.studio.toolbar.h2Sample)">{{ messages.studio.toolbar.h2 }}</button>
      <button class="tool-chip-btn" @mousedown.prevent="emit('insert', '\n### ', '\n', messages.studio.toolbar.h3Sample)">{{ messages.studio.toolbar.h3 }}</button>
      <button class="tool-chip-btn" @mousedown.prevent="emit('insert', '**', '**', messages.studio.toolbar.boldSample)"><b>{{ messages.studio.toolbar.bold }}</b></button>
      <button class="tool-chip-btn" @mousedown.prevent="emit('insert', '*', '*', messages.studio.toolbar.italicSample)"><i>{{ messages.studio.toolbar.italic }}</i></button>
      <button class="tool-chip-btn" @mousedown.prevent="emit('insert', '~~', '~~', messages.studio.toolbar.strikeSample)"><s>{{ messages.studio.toolbar.strike }}</s></button>
      <button class="tool-chip-btn" @mousedown.prevent="emit('insert', '==', '==', messages.studio.toolbar.highlightSample)">{{ messages.studio.toolbar.highlight }}</button>
      <button class="tool-chip-btn" @mousedown.prevent="emit('insert', '||', '||', messages.studio.toolbar.spoilerSample)">{{ messages.studio.toolbar.spoiler }}</button>
      <button class="tool-chip-btn" @mousedown.prevent="emit('insert', '[[', ']]', messages.studio.toolbar.wikiSample)">{{ messages.studio.toolbar.wiki }}</button>
      <button class="tool-chip-btn" @mousedown.prevent="emit('insert', '@', ' ', messages.studio.toolbar.mentionSample)">{{ messages.studio.toolbar.mention }}</button>
      <button class="tool-chip-btn" @mousedown.prevent="emit('insert', '\n> «', '»\n', messages.studio.toolbar.quoteSample)">{{ messages.studio.toolbar.quote }}</button>
      <button class="tool-chip-btn" @mousedown.prevent="emit('insert', '\n- [ ] ', '\n', messages.studio.toolbar.checklistSample)">{{ messages.studio.toolbar.checklist }}</button>
      <button class="tool-chip-btn" @mousedown.prevent="emit('insert', '\n- ', '\n', messages.studio.toolbar.listSample)">{{ messages.studio.toolbar.list }}</button>
      <button class="tool-chip-btn" @mousedown.prevent="emit('insert', '`', '`', messages.studio.toolbar.codeSample)">{{ messages.studio.toolbar.code }}</button>
      <button class="tool-chip-btn" @mousedown.prevent="emit('insert', '\n\n***\n\n', '', '')">{{ messages.studio.toolbar.asterism }}</button>
    </div>
  </div>
</template>

<style scoped>
.quantum-tools-strip {
  width: 100%;
  overflow-x: auto;
}

.tools-horizontal-lane {
  display: flex;
  gap: 6px;
  width: max-content;
}

.tool-chip-btn {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-muted);
  padding: 0 10px;
  height: 32px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 700;
}

.tool-chip-btn:hover {
  border-color: var(--border-accent);
  color: var(--text-pure);
}

@media (max-width: 820px) {
  .quantum-tools-strip {
    display: none !important;
  }
}
</style>