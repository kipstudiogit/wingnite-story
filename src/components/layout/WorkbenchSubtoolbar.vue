<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import type { AppTab, StudioViewMode, ThemeMode } from "../../types";
import { useI18n, type LocaleCode } from "../../i18n";

const studioMode = defineModel<StudioViewMode>("studioMode", { required: true });
const currentTheme = defineModel<ThemeMode>("currentTheme", { required: true });
const fontSize = defineModel<number>("fontSize", { required: true });

const { currentTab } = defineProps<{
  currentTab: AppTab;
}>();

const emit = defineEmits<{
  (e: "exportWstory"): void;
  (e: "exportHtml"): void;
  (e: "exportMd"): void;
  (e: "exportTxt"): void;
  (e: "getTwist"): void;
}>();

const { t, locale, availableLocales, setLocale } = useI18n();
</script>

<template>
  <div class="workbench-subtoolbar">
    <div class="subtoolbar-scroller">
      <div class="subtool-left">
        <div class="zoom-stepper">
          <button class="btn-stepper" :title="t('subtoolbar.fontSizeDecrease')" @click="fontSize = Math.max(13, fontSize - 1)">-</button>
          <span class="stepper-label">{{ fontSize }}px</span>
          <button class="btn-stepper" :title="t('subtoolbar.fontSizeIncrease')" @click="fontSize = Math.min(28, fontSize + 1)">+</button>
        </div>

        <div class="theme-picker-segmented">
          <button :class="['theme-dot-btn', { active: currentTheme === 'obsidian' }]" @click="currentTheme = 'obsidian'">{{ t("subtoolbar.themes.obsidian") }}</button>
          <button :class="['theme-dot-btn', { active: currentTheme === 'celestial' }]" @click="currentTheme = 'celestial'">{{ t("subtoolbar.themes.celestial") }}</button>
          <button :class="['theme-dot-btn', { active: currentTheme === 'sepia' }]" @click="currentTheme = 'sepia'">{{ t("subtoolbar.themes.sepia") }}</button>
          <button :class="['theme-dot-btn', { active: currentTheme === 'paper' }]" @click="currentTheme = 'paper'">{{ t("subtoolbar.themes.paper") }}</button>
          <button :class="['theme-dot-btn', { active: currentTheme === 'oled' }]" @click="currentTheme = 'oled'">{{ t("subtoolbar.themes.oled") }}</button>
        </div>
      </div>

      <div v-if="currentTab === 'studio'" class="layout-segmented">
        <button :class="['btn-layout', { active: studioMode === 'split' }]" @click="studioMode = 'split'">{{ t("subtoolbar.modes.split") }}</button>
        <button :class="['btn-layout', { active: studioMode === 'edit' }]" @click="studioMode = 'edit'">{{ t("subtoolbar.modes.edit") }}</button>
        <button :class="['btn-layout', { active: studioMode === 'preview' }]" @click="studioMode = 'preview'">{{ t("subtoolbar.modes.preview") }}</button>
        <button :class="['btn-layout', { active: studioMode === 'zen' }]" @click="studioMode = 'zen'">{{ t("subtoolbar.modes.zen") }}</button>
      </div>

      <div class="subtool-right-actions">
        <select
          class="locale-quick-select"
          :value="locale"
          :title="t('settings.localization.languageLabel')"
          @change="setLocale(($event.target as HTMLSelectElement).value as LocaleCode)"
        >
          <option v-for="loc in availableLocales" :key="loc.code" :value="loc.code">
            {{ loc.code.toUpperCase() }} — {{ loc.nativeName }}
          </option>
        </select>

        <button class="btn-quick-export" @click="emit('exportTxt')">{{ t("subtoolbar.quickTxt") }}</button>
        <button class="btn-quick-export" @click="emit('exportMd')">{{ t("subtoolbar.quickMd") }}</button>
        <button class="btn-quick-export" @click="emit('exportHtml')">{{ t("subtoolbar.quickHtml") }}</button>
        <button class="btn-quick-export" @click="emit('exportWstory')">{{ t("subtoolbar.quickWstory") }}</button>
        <button class="btn-generator-spark" @click="emit('getTwist')">{{ t("nav.twist") }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.workbench-subtoolbar {
  display: flex;
  align-items: center;
  background: var(--bg-surface-solid);
  border-bottom: 1px solid var(--border);
  height: 38px;
  flex-shrink: 0;
}

.subtoolbar-scroller {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 0 14px;
  gap: 10px;
}

.subtool-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.zoom-stepper {
  display: flex;
  align-items: center;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-card);
}

.btn-stepper {
  background: transparent;
  border: none;
  width: 26px;
  height: 26px;
  color: var(--text-muted);
  cursor: pointer;
}

.stepper-label {
  font-size: 10px;
  font-family: var(--font-mono);
  padding: 0 4px;
  color: var(--text-main);
}

.theme-picker-segmented {
  display: flex;
  gap: 2px;
  background: var(--bg-card);
  padding: 2px;
  border-radius: 8px;
  border: 1px solid var(--border);
}

.theme-dot-btn {
  background: transparent;
  border: none;
  font-size: 10px;
  font-weight: 700;
  padding: 3px 6px;
  border-radius: 6px;
  color: var(--text-muted);
  cursor: pointer;
}

.theme-dot-btn.active {
  background: var(--accent-indigo);
  color: #fff;
}

.layout-segmented {
  display: flex;
  gap: 2px;
  background: var(--bg-card);
  padding: 2px;
  border-radius: 8px;
  border: 1px solid var(--border);
}

.btn-layout {
  background: transparent;
  border: none;
  font-size: 11px;
  padding: 3px 8px;
  border-radius: 6px;
  color: var(--text-muted);
  cursor: pointer;
}

.btn-layout.active {
  background: var(--accent-indigo);
  color: #fff;
}

.subtool-right-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.locale-quick-select {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-main);
  border-radius: 6px;
  padding: 3px 6px;
  font-size: 10px;
  font-weight: 700;
  outline: none;
  cursor: pointer;
}

.btn-quick-export {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-main);
  border-radius: 6px;
  padding: 3px 8px;
  font-size: 10px;
  font-weight: 800;
  font-family: var(--font-mono);
  cursor: pointer;
}

.btn-quick-export:hover {
  border-color: var(--border-accent);
}

.btn-generator-spark {
  background: transparent;
  border: 1px solid var(--accent-indigo);
  color: var(--accent-indigo);
  border-radius: 8px;
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}
</style>