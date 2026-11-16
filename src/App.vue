<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { DEFAULT_BOOK_WORD_GOAL, THEME_META_COLORS } from "./constants/app";
import { useI18n } from "./i18n";
import { useProjects } from "./composables/useProjects";
import { useEnterpriseStory } from "./composables/useEnterpriseStory";
import { useSprint } from "./composables/useSprint";
import { useDevice } from "./composables/useDevice";
import { useAppUpdater } from "./composables/useAppUpdater";
import { useHaptics } from "./composables/useHaptics";
import { useLifecycle } from "./composables/useLifecycle";
import { useAppSettings } from "./composables/useAppSettings";
import { useStoryActions } from "./composables/useStoryActions";
import { useKeyboardShortcuts } from "./composables/useKeyboardShortcuts";
import { useConfirmDialog } from "./composables/useConfirmDialog";
import type {
  AppTab,
  EnterpriseStoryItem,
  ProjectItem,
  StudioViewMode,
  ThemeMode,
} from "./types";

import SplashScreen from "./components/SplashScreen.vue";
import Navigation from "./components/Navigation.vue";
import ReaderView from "./components/ReaderView.vue";
import StudioView from "./components/StudioView.vue";
import BoardView from "./components/BoardView.vue";
import CodexView from "./components/CodexView.vue";
import GraphView from "./components/GraphView.vue";
import TimelineView from "./components/TimelineView.vue";
import SettingsView from "./components/SettingsView.vue";
import CommandPalette from "./components/CommandPalette.vue";
import ProjectModal from "./components/modals/ProjectModal.vue";
import ConfirmModal from "./components/modals/ConfirmModal.vue";

import WorkbenchSubtoolbar from "./components/layout/WorkbenchSubtoolbar.vue";
import SidebarToc from "./components/layout/SidebarToc.vue";
import SystemStatusbar from "./components/layout/SystemStatusbar.vue";
import AppNotifications from "./components/common/AppNotifications.vue";

import MobileHeader from "./components/mobile/MobileHeader.vue";
import MobileNavigation from "./components/mobile/MobileNavigation.vue";
import MobileKeyboardBar from "./components/mobile/MobileKeyboardBar.vue";
import MobileTocSheet from "./components/mobile/MobileTocSheet.vue";
import MobileMenuSheet from "./components/mobile/MobileMenuSheet.vue";

const { t, locale } = useI18n();
const { requestConfirm } = useConfirmDialog();

const {
  projects,
  activeProjectId,
  loadSystemProjects,
  createProject,
  updateProject,
  deleteActiveProject,
  switchProject,
} = useProjects();

const {
  stories,
  activeId,
  currentTitle,
  currentCategory,
  currentContent,
  currentRevision,
  activeOperator,
  isSyncing,
  isDirty,
  syncError,
  currentWords,
  totalSystemWords,
  registerExternalStory,
  loadSystemStories,
  selectEnterpriseStory,
  persistStoryMutation,
  persistSpecificStory,
  deleteSystemStory,
  swapStoryPairNormalized,
  moveStoryToCategory,
  exportProjectWstory,
  importProjectWstory,
  initializeBlankDraft,
} = useEnterpriseStory();

const { settings } = useAppSettings();

const {
  sprintActive,
  sprintDelta,
  sprintFormattedTime,
  toggleSprint,
  cleanupSprint,
} = useSprint(
  () => currentWords.value,
  (wordsWritten) => {
    showToast(t("notifications.sprintFinished", { words: wordsWritten }), "success", 5000);
  }
);

const {
  isMobile,
  isKeyboardOpen,
  scrollToCursor,
  syncAndroidThemeColor,
} = useDevice();

const {
  isChecking: isCheckingUpdate,
  isInstalling: isInstallingUpdate,
  updateAvailable,
  updateInfo,
  checkForUpdates,
  installAndRestart,
  dismissUpdate,
} = useAppUpdater();

const { vibrateLight } = useHaptics();

const currentProjectTitle = () =>
  projects.value.find((p) => p.id === activeProjectId.value)?.title || t("projects.defaultTitle");

const {
  exportWstory,
  exportTxtBook,
  exportMarkdownBook,
  exportHtmlBook,
  importTxtFile,
  importWstoryFile,
} = useStoryActions(
  () => activeProjectId.value,
  currentProjectTitle,
  exportProjectWstory,
  importProjectWstory
);

useLifecycle(async () => {
  if (activeId.value && isDirty.value) {
    await persistStoryMutation(activeProjectId.value);
  }
});

useKeyboardShortcuts({
  onSave: handleSave,
  onTogglePalette: () => (paletteOpen.value = !paletteOpen.value),
  onEscape: () => {
    paletteOpen.value = false;
    tocOpen.value = false;
    mobileMenuOpen.value = false;
    showProjectModal.value = false;
  },
});

const isBackendLoaded = ref(false);
const showSplash = ref(true);

const currentTab = ref<AppTab>("studio");
const currentTheme = ref<ThemeMode>("obsidian");
const studioMode = ref<StudioViewMode>("split");
const fontSize = ref<number>(18);
const plotTwist = ref<string>("");
const tocOpen = ref<boolean>(false);
const paletteOpen = ref<boolean>(false);
const mobileMenuOpen = ref<boolean>(false);
const showProjectModal = ref<boolean>(false);
const editingProject = ref<ProjectItem | null>(null);
const exportNotification = ref<string | null>(null);
const notificationType = ref<"success" | "error">("success");
const wstoryFileInputRef = ref<HTMLInputElement | null>(null);
const txtFileInputRef = ref<HTMLInputElement | null>(null);

let autoSaveTimer: number | null = null;
let toastTimer: number | null = null;

function showToast(msg: string, kind: "success" | "error" = "success", durationMs = 3000) {
  if (toastTimer) clearTimeout(toastTimer);
  exportNotification.value = msg;
  notificationType.value = kind;
  toastTimer = window.setTimeout(() => {
    exportNotification.value = null;
  }, durationMs);
}

watch([currentTitle, currentCategory, currentContent], () => {
  if (!isDirty.value || settings.value.autoSaveIntervalMs <= 0 || !activeId.value) {
    return;
  }
  if (autoSaveTimer) clearTimeout(autoSaveTimer);
  autoSaveTimer = window.setTimeout(async () => {
    if (isDirty.value && activeId.value) {
      await persistStoryMutation(activeProjectId.value);
    }
  }, settings.value.autoSaveIntervalMs);
});

const showMobileDock = computed(() => isMobile.value && !isKeyboardOpen.value);

const targetBookGoal = computed(() => settings.value.bookWordGoal || DEFAULT_BOOK_WORD_GOAL);

const goalPercent = computed(() => {
  return Math.min(100, Math.round((totalSystemWords.value / targetBookGoal.value) * 100));
});

const actsList = computed(() => {
  const set = new Set<string>();
  for (const s of stories.value) {
    set.add(s.category);
  }
  return Array.from(set);
});

const activeStoryIndex = computed(() =>
  stories.value.findIndex((s) => s.id === activeId.value)
);

const hasPrevChapter = computed(() => activeStoryIndex.value > 0);
const hasNextChapter = computed(() =>
  activeStoryIndex.value >= 0 && activeStoryIndex.value < stories.value.length - 1
);

watch(currentTheme, (newTheme) => {
  syncAndroidThemeColor(THEME_META_COLORS[newTheme] || "#0e121c");
});

watch(activeProjectId, async (newPid) => {
  if (autoSaveTimer) clearTimeout(autoSaveTimer);
  await loadSystemStories(newPid);
});

async function initApplicationData() {
  await loadSystemProjects();
  await loadSystemStories(activeProjectId.value);
  isBackendLoaded.value = true;
  syncAndroidThemeColor(THEME_META_COLORS[currentTheme.value] || "#0e121c");
  checkForUpdates(true);
}

async function handleSave() {
  if (autoSaveTimer) clearTimeout(autoSaveTimer);
  const success = await persistStoryMutation(activeProjectId.value);
  if (success) {
    showToast(t("notifications.sceneSaved"), "success", 2400);
  } else if (syncError.value) {
    showToast(t("notifications.exportError", { error: syncError.value }), "error", 4000);
  }
}

async function handleDeleteActiveScene() {
  if (!activeId.value) return;
  const confirmed = await requestConfirm({
    title: t("studio.deleteSceneConfirmTitle"),
    message: t("studio.deleteSceneConfirmMessage"),
    confirmLabel: t("common.delete"),
    danger: true,
  });
  if (confirmed) {
    await deleteSystemStory(activeId.value);
  }
}

async function handleSaveCodexEntity(entity: EnterpriseStoryItem) {
  const saved = await persistSpecificStory(entity, activeProjectId.value);
  if (saved) {
    showToast(t("codex.dossier.canonSavedToast"), "success", 2500);
  } else if (syncError.value) {
    showToast(t("notifications.exportError", { error: syncError.value }), "error", 4000);
  }
}

function openCreateProjectModal() {
  editingProject.value = null;
  showProjectModal.value = true;
}

function openEditProjectModal(project: ProjectItem) {
  editingProject.value = project;
  showProjectModal.value = true;
}

async function handleProjectModalSubmit(payload: {
  id?: string;
  title: string;
  description: string;
}) {
  if (payload.id) {
    const updated = await updateProject(payload.id, payload.title, payload.description);
    if (updated) {
      showProjectModal.value = false;
      editingProject.value = null;
      showToast(t("projects.updatedToast"), "success", 2500);
    }
    return;
  }

  const saved = await createProject(payload.title, payload.description);
  if (saved) {
    showProjectModal.value = false;
    showToast(t("projects.createdToast"), "success", 2500);
  }
}

async function handleDeleteProject(id: string) {
  const confirmed = await requestConfirm({
    title: t("projects.deleteConfirmTitle"),
    message: t("projects.deleteConfirmMessage"),
    confirmLabel: t("common.delete"),
    danger: true,
  });
  if (!confirmed) return;
  const success = await deleteActiveProject(id);
  if (success) {
    showToast(t("projects.deletedToast"), "success", 2500);
  }
}

async function runExportWstory() {
  try {
    const status = await exportWstory();
    showToast(status, "success", 4000);
  } catch (err) {
    showToast(t("notifications.exportError", { error: String(err) }), "error", 4000);
  }
}

async function runExportTxt() {
  try {
    const status = await exportTxtBook();
    showToast(status, "success", 4000);
  } catch (err) {
    showToast(t("notifications.exportError", { error: String(err) }), "error", 4000);
  }
}

async function runExportMd() {
  try {
    const status = await exportMarkdownBook();
    showToast(status, "success", 4000);
  } catch (err) {
    showToast(t("notifications.exportError", { error: String(err) }), "error", 4000);
  }
}

async function runExportHtml() {
  try {
    const status = await exportHtmlBook();
    showToast(status, "success", 4000);
  } catch (err) {
    showToast(t("notifications.exportError", { error: String(err) }), "error", 4000);
  }
}

async function handleWstoryFileChange(e: Event) {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  try {
    const imported = await importWstoryFile(file);
    await loadSystemProjects();
    switchProject(imported.id);
    showToast(t("projects.importedToast", { title: imported.title }), "success", 4000);
  } catch (err) {
    showToast(t("notifications.importError", { error: String(err) }), "error", 4000);
  } finally {
    input.value = "";
  }
}

async function handleTxtFileChange(e: Event) {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  try {
    const imported = await importTxtFile(file);
    await loadSystemStories(activeProjectId.value);
    showToast(t("notifications.importedChapters", { count: imported.length }), "success", 4000);
  } catch (err) {
    showToast(t("notifications.importError", { error: String(err) }), "error", 4000);
  } finally {
    input.value = "";
  }
}

async function getTwist() {
  try {
    plotTwist.value = await invoke<string>("generate_twist", { locale: locale.value });
  } catch (err) {
    showToast(t("notifications.twistError", { error: String(err) }), "error", 3500);
  }
}

function insertTwistToActiveScene() {
  if (!plotTwist.value) return;
  currentContent.value += `\n\n> 💡 **${t("notifications.twistPrefix")}:** ${plotTwist.value}\n\n`;
  plotTwist.value = "";
  currentTab.value = "studio";
}

function handleMobileInsert(prefix: string, suffix: string, placeholder = "") {
  const textarea = document.querySelector(".enterprise-textarea") as HTMLTextAreaElement | null;
  if (!textarea) return;
  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;
  const selected = currentContent.value.substring(start, end) || placeholder;
  const replacement = prefix + selected + suffix;

  currentContent.value =
    currentContent.value.substring(0, start) + replacement + currentContent.value.substring(end);
  requestAnimationFrame(() => {
    textarea.focus();
    textarea.setSelectionRange(start + prefix.length, start + prefix.length + selected.length);
    scrollToCursor(textarea);
  });
}

function handleSelectStory(story: EnterpriseStoryItem) {
  if (!story.id) {
    initializeBlankDraft(story.category, story.title, story.content);
  } else {
    selectEnterpriseStory(story);
  }
  tocOpen.value = false;
  paletteOpen.value = false;
  mobileMenuOpen.value = false;
  if (currentTab.value !== "reader") {
    currentTab.value = "studio";
  }
}

function handleSelectStoryById(id: string) {
  const match = stories.value.find((s) => s.id === id);
  if (match) {
    handleSelectStory(match);
  }
}

function handleSelectEntityByTitle(title: string) {
  const match = stories.value.find(
    (s) => s.title.trim().toLowerCase() === title.trim().toLowerCase()
  );
  if (match) {
    handleSelectStory(match);
  } else {
    paletteOpen.value = true;
  }
}

onMounted(() => {
  initApplicationData();
});

onUnmounted(() => {
  cleanupSprint();
  if (autoSaveTimer) clearTimeout(autoSaveTimer);
  if (toastTimer) clearTimeout(toastTimer);
});
</script>

<template>
  <div :class="['studio-app-shell', `theme-${currentTheme}`, { 'mobile-platform': isMobile, 'keyboard-active': isKeyboardOpen }]">
    <div class="cyber-grid-overlay"></div>

    <input ref="wstoryFileInputRef" type="file" accept=".wstory,.json" style="display: none" @change="handleWstoryFileChange" />
    <input ref="txtFileInputRef" type="file" accept=".txt,.md" style="display: none" @change="handleTxtFileChange" />

    <transition name="splash-dissolve">
      <SplashScreen v-if="showSplash" :is-ready="isBackendLoaded" @finish="showSplash = false" />
    </transition>

    <AppNotifications
      :export-notification="exportNotification"
      :notification-type="notificationType"
      :plot-twist="plotTwist"
      :update-available="updateAvailable"
      :update-info="updateInfo"
      :is-installing-update="isInstallingUpdate"
      @close-toast="exportNotification = null"
      @insert-twist="insertTwistToActiveScene"
      @dismiss-twist="plotTwist = ''"
      @install-update="installAndRestart"
      @dismiss-update="dismissUpdate"
    />

    <CommandPalette
      v-if="paletteOpen"
      :stories="stories"
      :project-id="activeProjectId"
      @select="handleSelectStory"
      @close="paletteOpen = false"
    />
    <ProjectModal
      :is-open="showProjectModal"
      :editing-project="editingProject"
      @close="showProjectModal = false; editingProject = null"
      @submit="handleProjectModalSubmit"
    />
    <ConfirmModal />

    <SidebarToc
      v-if="!isMobile"
      :is-open="tocOpen"
      :stories="stories"
      :active-id="activeId"
      :goal-percent="goalPercent"
      :total-system-words="totalSystemWords"
      :target-book-goal="targetBookGoal"
      @close="tocOpen = false"
      @select-story="handleSelectStory"
      @new-draft="initializeBlankDraft(t('studio.defaultCategory'), '', ''); currentTab = 'studio'; tocOpen = false"
      @swap="swapStoryPairNormalized"
      @export-wstory="runExportWstory"
      @import-wstory="wstoryFileInputRef?.click()"
      @export-txt="runExportTxt"
      @import-txt="txtFileInputRef?.click()"
      @export-html="runExportHtml"
      @export-md="runExportMd"
    />

    <MobileHeader
      v-if="isMobile"
      :current-title="currentTitle"
      :current-words="currentWords"
      :current-revision="currentRevision"
      :is-syncing="isSyncing"
      :compact="isKeyboardOpen"
      @open-toc="vibrateLight(); tocOpen = true"
      @save="handleSave"
    />

    <Navigation
      v-else
      v-model:current-tab="currentTab"
      :sprint-active="sprintActive"
      :sprint-formatted-time="sprintFormattedTime"
      :sprint-delta="sprintDelta"
      :is-syncing="isSyncing"
      :projects="projects"
      :active-project-id="activeProjectId"
      :update-available="updateAvailable"
      @toggle-toc="tocOpen = !tocOpen"
      @open-palette="paletteOpen = true"
      @toggle-sprint="toggleSprint"
      @save="handleSave"
      @select-project="switchProject"
      @create-project="openCreateProjectModal"
      @edit-project="openEditProjectModal"
      @delete-project="handleDeleteProject"
      @export-wstory="runExportWstory"
      @import-wstory="wstoryFileInputRef?.click()"
      @check-update="checkForUpdates(false)"
    />

    <WorkbenchSubtoolbar
      v-if="!isMobile && currentTab !== 'settings'"
      v-model:studio-mode="studioMode"
      v-model:current-theme="currentTheme"
      v-model:font-size="fontSize"
      :current-tab="currentTab"
      @export-wstory="runExportWstory"
      @export-txt="runExportTxt"
      @export-html="runExportHtml"
      @export-md="runExportMd"
      @get-twist="getTwist"
    />

    <main class="workbench-main-stage">
      <ReaderView
        v-if="currentTab === 'reader'"
        :title="currentTitle"
        :category="currentCategory"
        :content="currentContent"
        :font-size="fontSize"
        :has-prev="hasPrevChapter"
        :has-next="hasNextChapter"
        @prev="selectEnterpriseStory(stories[activeStoryIndex - 1])"
        @next="selectEnterpriseStory(stories[activeStoryIndex + 1])"
        @edit="currentTab = 'studio'"
        @select-entity-by-title="handleSelectEntityByTitle"
      />

      <StudioView
        v-else-if="currentTab === 'studio'"
        v-model:title="currentTitle"
        v-model:category="currentCategory"
        v-model:content="currentContent"
        v-model:revision="currentRevision"
        :project-id="activeProjectId"
        :studio-mode="studioMode"
        :font-size="fontSize"
        :has-active="!!activeId"
        :compact-keyboard-mode="isMobile && isKeyboardOpen"
        :operator="activeOperator"
        :story-id="activeId"
        @save="handleSave"
        @delete="handleDeleteActiveScene"
        @select-story-by-id="handleSelectStoryById"
        @select-entity-by-title="handleSelectEntityByTitle"
        @entity-created="registerExternalStory"
        @notify="(msg, kind) => showToast(msg, kind, 3500)"
      />

      <BoardView
        v-else-if="currentTab === 'board'"
        :acts-list="actsList"
        :stories="stories"
        @select="handleSelectStory"
        @create="(act) => { initializeBlankDraft(act, '', ''); currentTab = 'studio' }"
        @swap="swapStoryPairNormalized"
        @move-category="moveStoryToCategory"
      />

      <CodexView
        v-else-if="currentTab === 'codex'"
        :project-id="activeProjectId"
        :stories="stories"
        :operator="activeOperator"
        @select="handleSelectStory"
        @save-entity="handleSaveCodexEntity"
        @notify="(msg, kind) => showToast(msg, kind, 2500)"
      />

      <TimelineView
        v-else-if="currentTab === 'timeline'"
        :project-id="activeProjectId"
        :stories="stories"
        :operator="activeOperator"
        @select-story="handleSelectStory"
      />

      <GraphView
        v-else-if="currentTab === 'graph'"
        :project-id="activeProjectId"
        :stories="stories"
        :operator="activeOperator"
        @select="handleSelectStory"
        @notify="(msg, kind) => showToast(msg, kind, 2500)"
      />

      <SettingsView
        v-else-if="currentTab === 'settings'"
        @notification="(msg, kind) => showToast(msg, kind, 3000)"
      />
    </main>

    <MobileKeyboardBar
      v-if="isMobile && currentTab === 'studio'"
      :is-syncing="isSyncing"
      @insert="handleMobileInsert"
      @save="handleSave"
    />

    <MobileNavigation
      v-if="showMobileDock"
      v-model:current-tab="currentTab"
      :sprint-active="sprintActive"
      :sprint-formatted-time="sprintFormattedTime"
      @open-menu="mobileMenuOpen = true"
      @open-search="paletteOpen = true"
      @toggle-sprint="toggleSprint"
    />

    <SystemStatusbar
      v-if="!isMobile"
      :is-syncing="isSyncing"
      :current-words="currentWords"
      :total-system-words="totalSystemWords"
      :current-revision="currentRevision"
      :sync-error="syncError"
    />

    <MobileTocSheet
      :is-open="isMobile && tocOpen"
      :stories="stories"
      :active-id="activeId"
      :goal-percent="goalPercent"
      :total-system-words="totalSystemWords"
      :target-book-goal="targetBookGoal"
      @close="tocOpen = false"
      @select-story="handleSelectStory"
      @new-draft="initializeBlankDraft(t('studio.defaultCategory'), '', ''); tocOpen = false; currentTab = 'studio'"
      @swap="swapStoryPairNormalized"
    />

    <MobileMenuSheet
      :is-open="isMobile && mobileMenuOpen"
      :projects="projects"
      :active-project-id="activeProjectId"
      :current-theme="currentTheme"
      :is-checking-update="isCheckingUpdate"
      @close="mobileMenuOpen = false"
      @switch-project="(id) => { switchProject(id); mobileMenuOpen = false }"
      @new-project="openCreateProjectModal(); mobileMenuOpen = false"
      @export-wstory="runExportWstory"
      @import-wstory="wstoryFileInputRef?.click()"
      @export-txt="runExportTxt"
      @import-txt="txtFileInputRef?.click()"
      @check-update="checkForUpdates(false)"
      @select-tab="(tab) => { currentTab = tab; mobileMenuOpen = false }"
      @open-search="paletteOpen = true; mobileMenuOpen = false"
      @get-twist="getTwist"
      @set-theme="(th) => (currentTheme = th)"
      @export-html="runExportHtml"
      @export-md="runExportMd"
    />
  </div>
</template>

<style scoped>
.studio-app-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  height: 100dvh;
  width: 100vw;
  background: var(--bg-deep);
  color: var(--text-main);
  overflow: hidden;
  position: relative;
}

.studio-app-shell.mobile-platform {
  user-select: none;
}

.splash-dissolve-leave-active {
  transition: opacity 0.35s cubic-bezier(0.16, 1, 0.3, 1), transform 0.35s cubic-bezier(0.16, 1, 0.3, 1);
}

.splash-dissolve-leave-to {
  opacity: 0;
  transform: scale(1.04);
}

.workbench-main-stage {
  flex: 1;
  display: flex;
  overflow: hidden;
  position: relative;
  min-height: 0;
}
</style>