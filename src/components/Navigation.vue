<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { APP_DISPLAY_VERSION, DEFAULT_PROJECT_ID } from "../constants/app";
import type { AppTab, ProjectItem } from "../types";
import { useI18n } from "../i18n";

const currentTab = defineModel<AppTab>("currentTab", { required: true });

const {
  sprintActive,
  sprintFormattedTime,
  sprintDelta,
  isSyncing = false,
  projects = [],
  activeProjectId = DEFAULT_PROJECT_ID,
  updateAvailable = false,
} = defineProps<{
  sprintActive: boolean;
  sprintFormattedTime: string;
  sprintDelta: number;
  isSyncing?: boolean;
  projects?: ProjectItem[];
  activeProjectId?: string;
  updateAvailable?: boolean;
}>();

const emit = defineEmits<{
  (e: "toggleToc"): void;
  (e: "openPalette"): void;
  (e: "toggleSprint"): void;
  (e: "save"): void;
  (e: "selectProject", id: string): void;
  (e: "createProject"): void;
  (e: "editProject", project: ProjectItem): void;
  (e: "deleteProject", id: string): void;
  (e: "exportWstory"): void;
  (e: "importWstory"): void;
  (e: "checkUpdate"): void;
}>();

const { t } = useI18n();

const projectDropdownOpen = ref(false);
const dropdownContainerRef = ref<HTMLElement | null>(null);

function handleDocumentClick(e: MouseEvent) {
  if (!projectDropdownOpen.value) return;
  const target = e.target as Node | null;
  if (dropdownContainerRef.value && target && !dropdownContainerRef.value.contains(target)) {
    projectDropdownOpen.value = false;
  }
}

function currentProjectTitle() {
  const match = projects.find((p) => p.id === activeProjectId);
  return match ? match.title : t("projects.defaultTitle");
}

onMounted(() => {
  document.addEventListener("mousedown", handleDocumentClick);
});

onUnmounted(() => {
  document.removeEventListener("mousedown", handleDocumentClick);
});
</script>

<template>
  <header class="master-quantum-header" data-tauri-drag-region>
    <div class="header-quad-left" data-tauri-drag-region>
      <button class="tactile-menu-trigger" :title="t('nav.toc')" @click="emit('toggleToc')">
        <svg class="ui-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="3" y1="6" x2="21" y2="6"/>
          <line x1="3" y1="12" x2="15" y2="12"/>
          <line x1="3" y1="18" x2="18" y2="18"/>
        </svg>
        <span class="trigger-text">{{ t("nav.toc") }}</span>
      </button>

      <div ref="dropdownContainerRef" class="project-selector-wrapper">
        <button class="btn-project-selector" @click="projectDropdownOpen = !projectDropdownOpen">
          <svg class="ui-icon" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/>
            <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>
          </svg>
          <span class="project-current-label">{{ currentProjectTitle() }}</span>
          <span class="dropdown-arrow">▾</span>
        </button>

        <div v-if="projectDropdownOpen" class="project-dropdown-menu" @click.stop>
          <div class="dropdown-header">{{ t("projects.title") }}</div>
          <div class="project-items-scroller">
            <div
              v-for="p in projects"
              :key="p.id"
              :class="['project-choice-row', { active: p.id === activeProjectId }]"
              @click="emit('selectProject', p.id); projectDropdownOpen = false"
            >
              <div class="choice-main-col">
                <div class="choice-title">{{ p.title }}</div>
                <div class="choice-desc">{{ p.description }}</div>
              </div>
              <div class="choice-actions-row">
                <button
                  class="btn-edit-proj"
                  :title="t('projects.editTooltip')"
                  @click.stop="emit('editProject', p); projectDropdownOpen = false"
                >
                  ✎
                </button>
                <button
                  v-if="projects.length > 1"
                  class="btn-delete-proj"
                  :title="t('common.delete')"
                  @click.stop="emit('deleteProject', p.id); projectDropdownOpen = false"
                >
                  ✕
                </button>
              </div>
            </div>
          </div>

          <div class="dropdown-divider"></div>

          <button class="btn-menu-action" @click="emit('createProject'); projectDropdownOpen = false">
            {{ t("projects.newProject") }}
          </button>
          <button class="btn-menu-action" @click="emit('exportWstory'); projectDropdownOpen = false">
            ↗ {{ t("toc.exportWstory") }}
          </button>
          <button class="btn-menu-action" @click="emit('importWstory'); projectDropdownOpen = false">
            ↘ {{ t("toc.importWstory") }}
          </button>
        </div>
      </div>

      <button class="tactile-search-capsule" :title="t('nav.searchPlaceholder')" @click="emit('openPalette')">
        <svg class="ui-icon icon-search" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="11" cy="11" r="8"/>
          <line x1="21" y1="21" x2="16.65" y2="16.65"/>
        </svg>
        <span class="search-placeholder">{{ t("nav.searchPlaceholder") }}</span>
        <kbd class="hotkey-pill">⌘K</kbd>
      </button>
    </div>

    <nav class="tactile-dock-container">
      <button :class="['dock-tab-btn', { active: currentTab === 'reader' }]" @click="currentTab = 'reader'">
        <span>{{ t("nav.reader") }}</span>
      </button>
      <button :class="['dock-tab-btn', { active: currentTab === 'studio' }]" @click="currentTab = 'studio'">
        <span>{{ t("nav.studio") }}</span>
      </button>
      <button :class="['dock-tab-btn', { active: currentTab === 'board' }]" @click="currentTab = 'board'">
        <span>{{ t("nav.board") }}</span>
      </button>
      <button :class="['dock-tab-btn', { active: currentTab === 'codex' }]" @click="currentTab = 'codex'">
        <span>{{ t("nav.codex") }}</span>
      </button>
      <button :class="['dock-tab-btn', { active: currentTab === 'timeline' }]" @click="currentTab = 'timeline'">
        <span>{{ t("nav.timeline") }}</span>
      </button>
      <button :class="['dock-tab-btn', { active: currentTab === 'graph' }]" @click="currentTab = 'graph'">
        <span>{{ t("nav.graph") }}</span>
      </button>
      <button :class="['dock-tab-btn', { active: currentTab === 'settings' }]" @click="currentTab = 'settings'">
        <span>{{ t("nav.settings") }}</span>
      </button>
    </nav>

    <div class="header-quad-right" data-tauri-drag-region>
      <button
        class="tactile-update-trigger"
        :class="{ pulse: updateAvailable }"
        :title="t('nav.checkUpdates')"
        @click="emit('checkUpdate')"
      >
        <svg class="ui-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.3"/>
        </svg>
        <span class="update-version-label">v{{ APP_DISPLAY_VERSION }}</span>
        <span v-if="updateAvailable" class="update-dot-indicator"></span>
      </button>

      <button :class="['tactile-save-trigger', { saving: isSyncing }]" :disabled="isSyncing" @click="emit('save')">
        <svg class="ui-icon" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>
        <span class="save-label">{{ isSyncing ? "..." : t("common.save") }}</span>
      </button>

      <button :class="['tactile-sprint-trigger', { running: sprintActive }]" @click="emit('toggleSprint')">
        <svg class="ui-icon icon-lightning" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
        <span v-if="sprintActive" class="sprint-numeric">{{ sprintFormattedTime }} +{{ sprintDelta }}</span>
        <span v-else class="sprint-label">{{ t("nav.sprint") }}</span>
      </button>
    </div>
  </header>
</template>

<style scoped>
.master-quantum-header {
  position: relative;
  z-index: 50;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: calc(var(--safe-top) + 6px) 16px 6px 16px;
  background: var(--bg-surface);
  backdrop-filter: blur(40px) saturate(220%);
  border-bottom: 1px solid var(--border);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4);
  gap: 12px;
  flex-shrink: 0;
  user-select: none;
}

.header-quad-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.ui-icon {
  flex-shrink: 0;
}

.tactile-menu-trigger {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-pure);
  padding: 0 12px;
  height: 40px;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 700;
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  flex-shrink: 0;
}

.project-selector-wrapper {
  position: relative;
  flex-shrink: 0;
}

.btn-project-selector {
  background: var(--bg-card);
  border: 1px solid var(--border-accent);
  color: var(--text-pure);
  border-radius: 10px;
  height: 40px;
  padding: 0 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 800;
  cursor: pointer;
  max-width: 190px;
}

.project-current-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dropdown-arrow {
  font-size: 10px;
  color: var(--accent-cyan);
}

.project-dropdown-menu {
  position: absolute;
  top: 46px;
  left: 0;
  width: 290px;
  background: var(--bg-surface-solid);
  border: 1px solid var(--border-strong);
  border-radius: 12px;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.85);
  padding: 8px;
  z-index: 120;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.dropdown-header {
  font-size: 10px;
  font-weight: 800;
  color: var(--text-dim);
  text-transform: uppercase;
  padding: 4px 8px;
}

.project-items-scroller {
  max-height: 220px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.project-choice-row {
  padding: 8px 10px;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.project-choice-row:hover,
.project-choice-row.active {
  background: var(--bg-card-hover);
  border-left: 3px solid var(--accent-cyan);
}

.choice-main-col {
  flex: 1;
  overflow: hidden;
}

.choice-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-pure);
}

.choice-desc {
  font-size: 10px;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.choice-actions-row {
  display: flex;
  align-items: center;
  gap: 4px;
}

.btn-edit-proj,
.btn-delete-proj {
  background: transparent;
  border: none;
  color: var(--text-dim);
  font-size: 12px;
  cursor: pointer;
  padding: 2px 5px;
  border-radius: 4px;
}

.btn-edit-proj:hover {
  color: var(--accent-cyan);
}

.btn-delete-proj:hover {
  color: var(--accent-rose);
}

.dropdown-divider {
  height: 1px;
  background: var(--border-subtle);
  margin: 4px 0;
}

.btn-menu-action {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-main);
  border-radius: 8px;
  padding: 8px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  text-align: left;
}

.tactile-search-capsule {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-muted);
  padding: 0 12px;
  height: 40px;
  border-radius: 10px;
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  min-width: 140px;
  max-width: 240px;
  flex: 1;
}

.icon-search {
  color: var(--accent-cyan);
}

.search-placeholder {
  flex: 1;
  text-align: left;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.hotkey-pill {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 700;
  background: var(--border-subtle);
  border: 1px solid var(--border);
  padding: 2px 5px;
  border-radius: 5px;
  color: var(--text-pure);
}

.tactile-dock-container {
  display: flex;
  background: var(--bg-card);
  backdrop-filter: blur(24px);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 3px;
  gap: 2px;
  flex-shrink: 0;
}

.dock-tab-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 700;
  padding: 0 10px;
  height: 34px;
  border-radius: 9px;
  cursor: pointer;
}

.dock-tab-btn:hover {
  color: var(--text-pure);
}

.dock-tab-btn.active {
  background: var(--accent-indigo);
  color: #fff;
  box-shadow: 0 4px 18px var(--accent-indigo-glow);
}

.header-quad-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  justify-content: flex-end;
}

.tactile-update-trigger {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-muted);
  border-radius: 10px;
  height: 40px;
  padding: 0 10px;
  display: flex;
  align-items: center;
  gap: 6px;
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}

.tactile-save-trigger {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  border-radius: 10px;
  height: 40px;
  padding: 0 16px;
  font-size: 13px;
  font-weight: 800;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.tactile-sprint-trigger {
  background: linear-gradient(135deg, var(--accent-cyan) 0%, var(--accent-indigo) 100%);
  color: #fff;
  border: none;
  border-radius: 10px;
  height: 40px;
  padding: 0 14px;
  font-size: 12px;
  font-weight: 800;
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
}

.icon-lightning {
  color: #fef08a;
}
</style>