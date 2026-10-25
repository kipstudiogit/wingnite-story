<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
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
import type { AppTab, StudioViewMode, ThemeMode, EnterpriseStoryItem } from "./types";

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

import WorkbenchSubtoolbar from "./components/layout/WorkbenchSubtoolbar.vue";
import SidebarToc from "./components/layout/SidebarToc.vue";
import SystemStatusbar from "./components/layout/SystemStatusbar.vue";
import AppNotifications from "./components/common/AppNotifications.vue";

import MobileHeader from "./components/mobile/MobileHeader.vue";
import MobileNavigation from "./components/mobile/MobileNavigation.vue";
import MobileKeyboardBar from "./components/mobile/MobileKeyboardBar.vue";
import MobileTocSheet from "./components/mobile/MobileTocSheet.vue";
import MobileMenuSheet from "./components/mobile/MobileMenuSheet.vue";

const {
  projects,
  activeProjectId,
  loadSystemProjects,
  createProject,
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
  syncError,
  currentWords,
  totalSystemWords,
  loadSystemStories,
  selectEnterpriseStory,
  persistStoryMutation,
  deleteSystemStory,
  batchReorderStories,
  exportProjectWstory,
  importProjectWstory,
  initializeBlankDraft,
  createEntityDraft,
} = useEnterpriseStory();

const { settings } = useAppSettings();

const {
  sprintActive,
  sprintDelta,
  sprintFormattedTime,
  toggleSprint,
  cleanupSprint,
} = useSprint(() => currentWords.value);

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

const currentProjectTitle = () => projects.value.find((p) => p.id === activeProjectId.value)?.title || "Проект";

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
  if (activeId.value && currentContent.value) {
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

let autoSaveTimer: number | null = null;
watch(currentContent, () => {
  if (settings.value.autoSaveIntervalMs > 0 && activeId.value) {
    if (autoSaveTimer) clearTimeout(autoSaveTimer);
    autoSaveTimer = window.setTimeout(async () => {
      await persistStoryMutation(activeProjectId.value);
    }, settings.value.autoSaveIntervalMs);
  }
});

const isBackendLoaded = ref(false);
const showSplash = ref(true);

const currentTab = ref<AppTab>("studio");
const currentTheme = ref<ThemeMode>("obsidian");
const studioMode = ref<StudioViewMode>("split");
const fontSize = ref<number>(18);
const targetBookGoal = ref<number>(settings.value.dailyWordGoal || 30000);
const plotTwist = ref<string>("");
const tocOpen = ref<boolean>(false);
const paletteOpen = ref<boolean>(false);
const mobileMenuOpen = ref<boolean>(false);
const showProjectModal = ref<boolean>(false);
const exportNotification = ref<string | null>(null);
const wstoryFileInputRef = ref<HTMLInputElement | null>(null);
const txtFileInputRef = ref<HTMLInputElement | null>(null);

const showMobileDock = computed(() => isMobile.value && !isKeyboardOpen.value);

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

const themeColors: Record<ThemeMode, string> = {
  obsidian: "#0a0712",
  celestial: "#0f0b04",
  sepia: "#f4ece0",
  paper: "#ffffff",
  oled: "#000000",
};

watch(currentTheme, (newTheme) => {
  syncAndroidThemeColor(themeColors[newTheme] || "#090d17");
});

watch(activeProjectId, async (newPid) => {
  await loadSystemStories(newPid);
});

async function initApplicationData() {
  await loadSystemProjects();
  await loadSystemStories(activeProjectId.value);
  isBackendLoaded.value = true;
  syncAndroidThemeColor(themeColors[currentTheme.value] || "#090d17");
  checkForUpdates(true);
}

async function handleSave() {
  const success = await persistStoryMutation(activeProjectId.value);
  if (success) {
    exportNotification.value = "Сцена сохранена ✓";
    setTimeout(() => (exportNotification.value = null), 2500);
  } else if (syncError.value) {
    exportNotification.value = `Ошибка: ${syncError.value}`;
  }
}

async function handleCreateProjectSubmit(payload: { title: string; description: string }) {
  const saved = await createProject(payload.title, payload.description);
  if (saved) {
    showProjectModal.value = false;
    exportNotification.value = "Новое произведение создано";
    setTimeout(() => (exportNotification.value = null), 2500);
  }
}

async function handleDeleteProject(id: string) {
  if (!confirm("Вы действительно хотите удалить это произведение и все его сцены?")) return;
  const success = await deleteActiveProject(id);
  if (success) {
    exportNotification.value = "Произведение удалено";
    setTimeout(() => (exportNotification.value = null), 2500);
  }
}

async function runExportWstory() {
  try {
    exportNotification.value = await exportWstory();
  } catch (err) {
    exportNotification.value = `Ошибка экспорта: ${String(err)}`;
  }
  setTimeout(() => (exportNotification.value = null), 4000);
}

async function runExportTxt() {
  try {
    exportNotification.value = await exportTxtBook();
  } catch (err) {
    exportNotification.value = `Ошибка: ${String(err)}`;
  }
  setTimeout(() => (exportNotification.value = null), 4000);
}

async function runExportMd() {
  try {
    exportNotification.value = await exportMarkdownBook();
  } catch (err) {
    exportNotification.value = `Ошибка: ${String(err)}`;
  }
  setTimeout(() => (exportNotification.value = null), 4000);
}

async function runExportHtml() {
  try {
    exportNotification.value = await exportHtmlBook();
  } catch (err) {
    exportNotification.value = `Ошибка: ${String(err)}`;
  }
  setTimeout(() => (exportNotification.value = null), 4000);
}

async function handleWstoryFileChange(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (!file) return;
  try {
    const imported = await importWstoryFile(file);
    await loadSystemProjects();
    switchProject(imported.id);
    exportNotification.value = `Импортировано: ${imported.title}`;
  } catch (err) {
    exportNotification.value = `Ошибка: ${String(err)}`;
  }
  setTimeout(() => (exportNotification.value = null), 4000);
}

async function handleTxtFileChange(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (!file) return;
  try {
    const imported = await importTxtFile(file);
    await loadSystemStories(activeProjectId.value);
    exportNotification.value = `Импортировано сцен: ${imported.length}`;
  } catch (err) {
    exportNotification.value = `Ошибка: ${String(err)}`;
  }
  setTimeout(() => (exportNotification.value = null), 4000);
}

async function getTwist() {
  try {
    plotTwist.value = await invoke<string>("generate_twist");
  } catch (err) {
    syncError.value = String(err);
  }
}

function insertTwistToActiveScene() {
  if (!plotTwist.value) return;
  currentContent.value += `\n\n> 💡 **Сюжетный поворот:** ${plotTwist.value}\n\n`;
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

  currentContent.value = currentContent.value.substring(0, start) + replacement + currentContent.value.substring(end);
  requestAnimationFrame(() => {
    textarea.focus();
    textarea.setSelectionRange(start + prefix.length, start + prefix.length + selected.length);
    scrollToCursor(textarea);
  });
}

function handleSelectStory(story: EnterpriseStoryItem) {
  selectEnterpriseStory(story);
  tocOpen.value = false;
  paletteOpen.value = false;
  mobileMenuOpen.value = false;
  if (currentTab.value !== "reader") currentTab.value = "studio";
}

function handleSelectEntityByTitle(title: string) {
  const match = stories.value.find((s) => s.title.trim().toLowerCase() === title.trim().toLowerCase());
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

    <CommandPalette v-if="paletteOpen" :stories="stories" @select="handleSelectStory" @close="paletteOpen = false" />
    <ProjectModal :is-open="showProjectModal" @close="showProjectModal = false" @submit="handleCreateProjectSubmit" />

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
      @new-draft="initializeBlankDraft('Акт I', '', ''); currentTab = 'studio'; tocOpen = false"
      @reorder="batchReorderStories"
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
      @create-project="showProjectModal = true"
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
        :studio-mode="studioMode"
        :font-size="fontSize"
        :has-active="!!activeId"
        :operator="activeOperator"
        :story-id="activeId"
        @save="handleSave"
        @delete="deleteSystemStory(activeId)"
        @select-story-by-id="(id) => handleSelectStory(stories.find(s => s.id === id)!)"
        @select-entity-by-title="handleSelectEntityByTitle"
      />

      <BoardView
        v-else-if="currentTab === 'board'"
        :acts-list="actsList"
        :stories="stories"
        @select="handleSelectStory"
        @create="(act) => { initializeBlankDraft(act, '', ''); currentTab = 'studio' }"
        @reorder="batchReorderStories"
      />

      <CodexView
        v-else-if="currentTab === 'codex'"
        :stories="stories"
        @select="handleSelectStory"
        @create-entity="(type) => { createEntityDraft(type); currentTab = 'studio' }"
        @save-entity="handleSave"
      />

      <TimelineView v-else-if="currentTab === 'timeline'" :stories="stories" :operator="activeOperator" @select-story="handleSelectStory" />
      <GraphView v-else-if="currentTab === 'graph'" :stories="stories" :operator="activeOperator" @select="handleSelectStory" />
      <SettingsView v-else-if="currentTab === 'settings'" @notification="(msg) => exportNotification = msg" />
    </main>

    <MobileKeyboardBar v-if="isMobile && currentTab === 'studio'" :is-syncing="isSyncing" @insert="handleMobileInsert" @save="handleSave" />
    <MobileNavigation v-if="showMobileDock" v-model:current-tab="currentTab" :sprint-active="sprintActive" :sprint-formatted-time="sprintFormattedTime" @open-menu="mobileMenuOpen = true" @open-search="paletteOpen = true" @toggle-sprint="toggleSprint" />

    <SystemStatusbar v-if="!isMobile" :is-syncing="isSyncing" :current-words="currentWords" :total-system-words="totalSystemWords" :current-revision="currentRevision" :sync-error="syncError" />

    <MobileTocSheet
      :is-open="isMobile && tocOpen"
      :stories="stories"
      :active-id="activeId"
      :goal-percent="goalPercent"
      :total-system-words="totalSystemWords"
      :target-book-goal="targetBookGoal"
      @close="tocOpen = false"
      @select-story="handleSelectStory"
      @new-draft="initializeBlankDraft('Акт I', '', ''); tocOpen = false; currentTab = 'studio'"
      @reorder="batchReorderStories"
    />

    <MobileMenuSheet
      :is-open="isMobile && mobileMenuOpen"
      :projects="projects"
      :active-project-id="activeProjectId"
      :current-theme="currentTheme"
      :is-checking-update="isCheckingUpdate"
      @close="mobileMenuOpen = false"
      @switch-project="(id) => { switchProject(id); mobileMenuOpen = false }"
      @new-project="showProjectModal = true; mobileMenuOpen = false"
      @export-wstory="runExportWstory"
      @import-wstory="wstoryFileInputRef?.click()"
      @export-txt="runExportTxt"
      @import-txt="txtFileInputRef?.click()"
      @check-update="checkForUpdates(false)"
      @select-tab="(tab) => { currentTab = tab; mobileMenuOpen = false }"
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
  transition: opacity 0.45s cubic-bezier(0.16, 1, 0.3, 1), transform 0.45s cubic-bezier(0.16, 1, 0.3, 1);
}

.splash-dissolve-leave-to {
  opacity: 0;
  transform: scale(1.05);
}

.workbench-main-stage {
  flex: 1;
  display: flex;
  overflow: hidden;
  position: relative;
  min-height: 0;
}
</style>