<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import MobileDrawerSheet from "./MobileDrawerSheet.vue";
import type { AppTab, ProjectItem, ThemeMode } from "../../types";
import { useI18n, type LocaleCode } from "../../i18n";

const {
  isOpen,
  projects,
  activeProjectId,
  currentTheme,
  isCheckingUpdate,
} = defineProps<{
  isOpen: boolean;
  projects: ProjectItem[];
  activeProjectId: string;
  currentTheme: ThemeMode;
  isCheckingUpdate: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "switchProject", id: string): void;
  (e: "newProject"): void;
  (e: "exportWstory"): void;
  (e: "importWstory"): void;
  (e: "checkUpdate"): void;
  (e: "selectTab", tab: AppTab): void;
  (e: "openSearch"): void;
  (e: "getTwist"): void;
  (e: "setTheme", theme: ThemeMode): void;
  (e: "exportHtml"): void;
  (e: "exportMd"): void;
  (e: "exportTxt"): void;
  (e: "importTxt"): void;
}>();

const { t, locale, availableLocales, setLocale } = useI18n();
</script>

<template>
  <MobileDrawerSheet
    :is-open="isOpen"
    :title="t('mobile.toolsTitle')"
    @close="emit('close')"
  >
    <div class="mobile-menu-grid">
      <div class="menu-action-block">
        <h4>{{ t("projects.title") }}</h4>
        <div class="menu-btn-row">
          <button
            v-for="p in projects"
            :key="p.id"
            :class="['menu-chip-btn', { active: p.id === activeProjectId }]"
            @click="emit('switchProject', p.id)"
          >
            {{ p.title }}
          </button>
          <button class="menu-chip-btn accent" @click="emit('newProject')">
            {{ t("projects.newProject") }}
          </button>
        </div>
      </div>

      <div class="menu-action-block">
        <h4>{{ t("settings.localization.languageLabel") }}</h4>
        <div class="menu-btn-row">
          <button
            v-for="loc in availableLocales"
            :key="loc.code"
            :class="['theme-chip-btn', { active: locale === loc.code }]"
            @click="setLocale(loc.code as LocaleCode)"
          >
            {{ loc.nativeName }}
          </button>
        </div>
      </div>

      <div class="menu-action-block">
        <h4>{{ t("mobile.importExportTitle") }}</h4>
        <div class="menu-btn-row">
          <button class="menu-chip-btn" @click="emit('importTxt')">{{ t("toc.importTxt") }}</button>
          <button class="menu-chip-btn" @click="emit('exportTxt')">{{ t("toc.exportTxt") }}</button>
          <button class="menu-chip-btn" @click="emit('exportWstory')">{{ t("toc.exportWstory") }}</button>
          <button class="menu-chip-btn" @click="emit('importWstory')">{{ t("toc.importWstory") }}</button>
        </div>
      </div>

      <div class="menu-action-block">
        <h4>{{ t("mobile.modulesTitle") }}</h4>
        <div class="menu-btn-row">
          <button class="menu-chip-btn accent" @click="emit('openSearch')">⌕ {{ t("common.search") }}</button>
          <button class="menu-chip-btn" @click="emit('selectTab', 'timeline')">{{ t("nav.timeline") }}</button>
          <button class="menu-chip-btn" @click="emit('selectTab', 'graph')">{{ t("nav.graph") }}</button>
          <button class="menu-chip-btn accent-purple" @click="emit('selectTab', 'settings')">⚙ {{ t("nav.settings") }}</button>
          <button class="menu-chip-btn" @click="emit('getTwist')">{{ t("nav.twist") }}</button>
        </div>
      </div>

      <div class="menu-action-block">
        <h4>{{ t("mobile.themesTitle") }}</h4>
        <div class="menu-btn-row">
          <button :class="['theme-chip-btn', { active: currentTheme === 'obsidian' }]" @click="emit('setTheme', 'obsidian')">{{ t("subtoolbar.themes.obsidian") }}</button>
          <button :class="['theme-chip-btn', { active: currentTheme === 'celestial' }]" @click="emit('setTheme', 'celestial')">{{ t("subtoolbar.themes.celestial") }}</button>
          <button :class="['theme-chip-btn', { active: currentTheme === 'sepia' }]" @click="emit('setTheme', 'sepia')">{{ t("subtoolbar.themes.sepia") }}</button>
          <button :class="['theme-chip-btn', { active: currentTheme === 'paper' }]" @click="emit('setTheme', 'paper')">{{ t("subtoolbar.themes.paper") }}</button>
          <button :class="['theme-chip-btn', { active: currentTheme === 'oled' }]" @click="emit('setTheme', 'oled')">{{ t("subtoolbar.themes.oled") }}</button>
        </div>
      </div>

      <div class="menu-action-block">
        <h4>{{ t("mobile.buildAndUpdatesTitle") }}</h4>
        <div class="menu-btn-row vertical">
          <button class="menu-export-btn" @click="emit('checkUpdate')">
            {{ isCheckingUpdate ? t("mobile.checkingBtn") : t("mobile.checkUpdateBtn") }}
          </button>
          <button class="menu-export-btn" @click="emit('exportHtml')">{{ t("toc.buildHtml") }}</button>
          <button class="menu-export-btn" @click="emit('exportMd')">{{ t("toc.exportMarkdown") }}</button>
        </div>
      </div>
    </div>
  </MobileDrawerSheet>
</template>

<style scoped>
.mobile-menu-grid {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.menu-action-block h4 {
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--text-muted);
  margin-bottom: 8px;
}

.menu-btn-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.menu-btn-row.vertical {
  flex-direction: column;
}

.menu-chip-btn {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-main);
  border-radius: 10px;
  min-height: 42px;
  padding: 10px 14px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.menu-chip-btn.active {
  background: var(--accent-indigo);
  color: #fff;
  border-color: var(--accent-indigo);
}

.menu-chip-btn.accent {
  background: var(--bg-card-hover);
  border-color: var(--accent-cyan);
  color: var(--accent-cyan);
}

.menu-chip-btn.accent-purple {
  background: var(--bg-card-hover);
  border-color: var(--accent-purple);
  color: var(--accent-purple);
}

.theme-chip-btn {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-muted);
  border-radius: 8px;
  min-height: 38px;
  padding: 8px 12px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}

.theme-chip-btn.active {
  background: var(--accent-indigo);
  color: #fff;
  border-color: var(--accent-indigo);
}

.menu-export-btn {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-pure);
  border-radius: 12px;
  min-height: 44px;
  padding: 12px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  text-align: center;
}
</style>