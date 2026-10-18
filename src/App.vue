<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useEnterpriseStory } from "./composables/useEnterpriseStory";
import { useSprint } from "./composables/useSprint";
import { useDevice } from "./composables/useDevice";
import type { AppTab, StudioViewMode, ThemeMode, EnterpriseStoryItem } from "./types";

import SplashScreen from "./components/SplashScreen.vue";
import Navigation from "./components/Navigation.vue";
import ReaderView from "./components/ReaderView.vue";
import StudioView from "./components/StudioView.vue";
import BoardView from "./components/BoardView.vue";
import CodexView from "./components/CodexView.vue";
import GraphView from "./components/GraphView.vue";
import TimelineView from "./components/TimelineView.vue";
import CommandPalette from "./components/CommandPalette.vue";

import MobileNavigation from "./components/mobile/MobileNavigation.vue";
import MobileKeyboardBar from "./components/mobile/MobileKeyboardBar.vue";
import MobileDrawerSheet from "./components/mobile/MobileDrawerSheet.vue";

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
  initializeBlankDraft,
  createEntityDraft,
} = useEnterpriseStory();

const {
  sprintActive,
  sprintDelta,
  sprintFormattedTime,
  toggleSprint,
  cleanupSprint,
} = useSprint(() => currentWords.value);

const { isMobile, isKeyboardOpen } = useDevice();

const isBackendLoaded = ref(false);
const showSplash = ref(true);

const currentTab = ref<AppTab>("studio");
const currentTheme = ref<ThemeMode>("obsidian");
const studioMode = ref<StudioViewMode>("split");
const fontSize = ref<number>(18);
const targetBookGoal = ref<number>(30000);
const plotTwist = ref<string>("");
const tocOpen = ref<boolean>(false);
const paletteOpen = ref<boolean>(false);
const mobileMenuOpen = ref<boolean>(false);
const exportNotification = ref<string | null>(null);

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

async function initApplicationData() {
  await loadSystemStories();
  isBackendLoaded.value = true;
}

async function handleSave() {
  const success = await persistStoryMutation();
  if (success) {
    exportNotification.value = "Сцена успешно сохранена ✓";
    setTimeout(() => {
      exportNotification.value = null;
    }, 2500);
  } else if (syncError.value) {
    exportNotification.value = `Ошибка сохранения: ${syncError.value}`;
  }
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
  if (!textarea) {
    currentContent.value += prefix + placeholder + suffix;
    return;
  }

  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;
  const selected = currentContent.value.substring(start, end) || placeholder;
  const replacement = prefix + selected + suffix;

  if (typeof textarea.setRangeText === "function") {
    textarea.setRangeText(replacement, start, end, "select");
    currentContent.value = textarea.value;
    textarea.dispatchEvent(new Event("input", { bubbles: true }));
  } else {
    currentContent.value =
      currentContent.value.substring(0, start) + replacement + currentContent.value.substring(end);
  }

  requestAnimationFrame(() => {
    textarea.focus();
    textarea.setSelectionRange(start + prefix.length, start + prefix.length + selected.length);
  });
}

async function exportStandaloneBook() {
  try {
    const htmlBook = await invoke<string>("compile_book_edition");
    try {
      const savedPath = await invoke<string>("save_export_file", {
        filename: "Wingnite_Enterprise_Book.html",
        content: htmlBook,
      });
      exportNotification.value = `Книга сохранена: ${savedPath}`;
    } catch {
      const blob = new Blob([htmlBook], { type: "text/html;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "Wingnite_Enterprise_Book.html";
      a.click();
      URL.revokeObjectURL(url);
      exportNotification.value = "Издание сохранено через веб-загрузку.";
    }
    setTimeout(() => {
      exportNotification.value = null;
    }, 5000);
  } catch (err) {
    syncError.value = String(err);
  }
}

async function exportFullManuscript() {
  try {
    const manuscript = await invoke<string>("export_manuscript");
    try {
      const savedPath = await invoke<string>("save_export_file", {
        filename: "Wingnite_Enterprise_Manuscript.md",
        content: manuscript,
      });
      exportNotification.value = `Рукопись сохранена: ${savedPath}`;
    } catch {
      const blob = new Blob([manuscript], { type: "text/markdown;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "Wingnite_Enterprise_Manuscript.md";
      a.click();
      URL.revokeObjectURL(url);
      exportNotification.value = "Рукопись сохранена через веб-загрузку.";
    }
    setTimeout(() => {
      exportNotification.value = null;
    }, 5000);
  } catch (err) {
    syncError.value = String(err);
  }
}

function handleSelectStory(story: EnterpriseStoryItem) {
  selectEnterpriseStory(story);
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

function handleEntityCreation(type: "artifact" | "pantheon" | "sanctuary" | "character") {
  createEntityDraft(type);
  currentTab.value = "studio";
  mobileMenuOpen.value = false;
}

function handleCreateSceneInAct(act: string) {
  initializeBlankDraft(act, "Новая сцена");
  currentTab.value = "studio";
  mobileMenuOpen.value = false;
}

async function handleDeleteActiveStory() {
  if (activeId.value) {
    await deleteSystemStory(activeId.value);
  } else {
    initializeBlankDraft();
  }
}

function handlePrevChapter() {
  if (hasPrevChapter.value) {
    selectEnterpriseStory(stories.value[activeStoryIndex.value - 1]);
  }
}

function handleNextChapter() {
  if (hasNextChapter.value) {
    selectEnterpriseStory(stories.value[activeStoryIndex.value + 1]);
  }
}

function handleKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
    e.preventDefault();
    paletteOpen.value = !paletteOpen.value;
  }
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
    e.preventDefault();
    handleSave();
  }
  if (e.key === "Escape") {
    paletteOpen.value = false;
    tocOpen.value = false;
    mobileMenuOpen.value = false;
  }
}

onMounted(() => {
  initApplicationData();
  window.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
  cleanupSprint();
});
</script>

<template>
  <div :class="['studio-app-shell', `theme-${currentTheme}`, { 'mobile-platform': isMobile, 'keyboard-active': isKeyboardOpen }]">
    <div class="cyber-grid-overlay"></div>

    <transition name="splash-dissolve">
      <SplashScreen
        v-if="showSplash"
        :is-ready="isBackendLoaded"
        @finish="showSplash = false"
      />
    </transition>

    <div v-if="exportNotification" class="export-toast-banner" :class="{ error: exportNotification.startsWith('Ошибка') }">
      <span>{{ exportNotification }}</span>
      <button class="toast-close" @click="exportNotification = null">✕</button>
    </div>

    <CommandPalette
      v-if="paletteOpen"
      :stories="stories"
      @select="handleSelectStory"
      @close="paletteOpen = false"
    />

    <div v-if="!isMobile && tocOpen" class="drawer-backdrop" @click="tocOpen = false"></div>

    <aside v-if="!isMobile" :class="['sidebar-mobile-drawer', { open: tocOpen }]">
      <div class="drawer-header">
        <div class="brand-crest">
          <svg class="crest-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M20.24 12.24a6 6 0 0 0-8.49-8.49L5 10.5V19h8.5z"/>
            <line x1="16" y1="8" x2="2" y2="22"/>
            <line x1="17.5" y1="15" x2="9" y2="15"/>
          </svg>
          <span>Wingnite Enterprise</span>
        </div>
        <button class="btn-drawer-close" title="Закрыть" @click="tocOpen = false">✕</button>
      </div>

      <div class="drawer-progress">
        <div class="meter-info">
          <span>Цель производства</span>
          <span>{{ goalPercent }}%</span>
        </div>
        <div class="meter-track">
          <div class="meter-fill" :style="{ width: `${goalPercent}%` }"></div>
        </div>
        <div class="meter-sub">{{ totalSystemWords }} / {{ targetBookGoal }} слов</div>
      </div>

      <div class="drawer-scrollable-list">
        <div
          v-for="(item, idx) in stories"
          :key="item.id"
          :class="['drawer-chapter-row', { active: item.id === activeId }]"
          @click="handleSelectStory(item)"
        >
          <span class="chapter-idx">{{ String(idx + 1).padStart(2, "0") }}</span>
          <div class="chapter-info">
            <span class="chapter-title">{{ item.title }}</span>
            <span class="chapter-cat">{{ item.category }} • r{{ item.revision }}</span>
          </div>
        </div>
      </div>

      <div class="drawer-footer">
        <button
          class="btn-drawer-action primary"
          @click="initializeBlankDraft(); currentTab = 'studio'; tocOpen = false"
        >
          <span>+ Новый драфт</span>
        </button>
        <button class="btn-drawer-action" @click="exportStandaloneBook">
          <span>Собрать издание (HTML)</span>
        </button>
        <button class="btn-drawer-action" @click="exportFullManuscript">
          <span>Экспорт Markdown</span>
        </button>
      </div>
    </aside>

    <header v-if="isMobile" class="mobile-master-header">
      <button class="mobile-action-icon-btn" title="Оглавление" @click="tocOpen = true">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="3" y1="6" x2="21" y2="6"/>
          <line x1="3" y1="12" x2="15" y2="12"/>
          <line x1="3" y1="18" x2="18" y2="18"/>
        </svg>
      </button>

      <div class="mobile-header-meta">
        <h2 class="mobile-title-text">{{ currentTitle || 'Сцена без названия' }}</h2>
        <span class="mobile-words-badge">{{ currentWords }} сл. • r{{ currentRevision }}</span>
      </div>

      <button
        class="mobile-save-pill"
        :class="{ saving: isSyncing }"
        :disabled="isSyncing"
        @click="handleSave"
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
          <polyline points="17 21 17 13 7 13 7 21"/>
          <polyline points="7 3 7 8 15 8"/>
        </svg>
      </button>
    </header>

    <Navigation
      v-else
      v-model:current-tab="currentTab"
      :sprint-active="sprintActive"
      :sprint-formatted-time="sprintFormattedTime"
      :sprint-delta="sprintDelta"
      :is-syncing="isSyncing"
      @toggle-toc="tocOpen = !tocOpen"
      @open-palette="paletteOpen = true"
      @toggle-sprint="toggleSprint"
      @save="handleSave"
    />

    <div v-if="!isMobile" class="workbench-subtoolbar">
      <div class="subtoolbar-scroller">
        <div class="subtool-left">
          <div class="zoom-stepper">
            <button class="btn-stepper" @click="fontSize = Math.max(13, fontSize - 1)">-</button>
            <span class="stepper-label">{{ fontSize }}px</span>
            <button class="btn-stepper" @click="fontSize = Math.min(28, fontSize + 1)">+</button>
          </div>

          <div class="theme-picker-segmented">
            <button
              :class="['theme-dot-btn', { active: currentTheme === 'obsidian' }]"
              @click="currentTheme = 'obsidian'"
            >
              Dark
            </button>
            <button
              :class="['theme-dot-btn', { active: currentTheme === 'celestial' }]"
              @click="currentTheme = 'celestial'"
            >
              Amber
            </button>
            <button
              :class="['theme-dot-btn', { active: currentTheme === 'sepia' }]"
              @click="currentTheme = 'sepia'"
            >
              Sepia
            </button>
            <button
              :class="['theme-dot-btn', { active: currentTheme === 'paper' }]"
              @click="currentTheme = 'paper'"
            >
              Paper
            </button>
            <button
              :class="['theme-dot-btn', { active: currentTheme === 'oled' }]"
              @click="currentTheme = 'oled'"
            >
              OLED
            </button>
          </div>
        </div>

        <div v-if="currentTab === 'studio'" class="layout-segmented">
          <button
            :class="['btn-layout', { active: studioMode === 'split' }]"
            @click="studioMode = 'split'"
          >
            <span>Сплит</span>
          </button>
          <button
            :class="['btn-layout', { active: studioMode === 'edit' }]"
            @click="studioMode = 'edit'"
          >
            <span>Редактор</span>
          </button>
          <button
            :class="['btn-layout', { active: studioMode === 'preview' }]"
            @click="studioMode = 'preview'"
          >
            <span>Превью</span>
          </button>
          <button
            :class="['btn-layout', { active: studioMode === 'zen' }]"
            @click="studioMode = 'zen'"
          >
            <span>Фокус</span>
          </button>
        </div>

        <div class="subtool-right-actions">
          <button class="btn-quick-export" title="Экспорт издания в HTML" @click="exportStandaloneBook">
            HTML
          </button>
          <button class="btn-quick-export" title="Экспорт рукописи в Markdown" @click="exportFullManuscript">
            MD
          </button>
          <button class="btn-generator-spark" @click="getTwist">
            <span>Твист</span>
          </button>
          <button
            class="btn-quick-save"
            :disabled="isSyncing"
            title="Сохранить изменения (⌘S)"
            @click="handleSave"
          >
            {{ isSyncing ? '...' : 'Сохранить' }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="plotTwist" class="twist-banner">
      <div class="twist-tag">ТВИСТ</div>
      <p class="twist-text">{{ plotTwist }}</p>
      <div class="twist-action-row">
        <button class="twist-insert-btn" @click="insertTwistToActiveScene">Вставить</button>
        <button class="twist-dismiss" @click="plotTwist = ''">✕</button>
      </div>
    </div>

    <main class="workbench-main-stage">
      <ReaderView
        v-if="currentTab === 'reader'"
        :title="currentTitle"
        :category="currentCategory"
        :content="currentContent"
        :font-size="fontSize"
        :has-prev="hasPrevChapter"
        :has-next="hasNextChapter"
        @prev="handlePrevChapter"
        @next="handleNextChapter"
        @edit="currentTab = 'studio'"
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
        :is-syncing="isSyncing"
        @save="handleSave"
        @delete="handleDeleteActiveStory"
        @select-story-by-id="handleSelectStoryById"
      />

      <BoardView
        v-else-if="currentTab === 'board'"
        :acts-list="actsList"
        :stories="stories"
        @select="handleSelectStory"
        @create="handleCreateSceneInAct"
      />

      <CodexView
        v-else-if="currentTab === 'codex'"
        :stories="stories"
        @select="handleSelectStory"
        @create-entity="handleEntityCreation"
      />

      <TimelineView
        v-else-if="currentTab === 'timeline'"
        :stories="stories"
        :operator="activeOperator"
        @select-story="handleSelectStory"
      />

      <GraphView
        v-else-if="currentTab === 'graph'"
        :stories="stories"
        :operator="activeOperator"
        @select="handleSelectStory"
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

    <footer v-if="!isMobile" class="system-statusbar">
      <div class="status-left">
        <span class="status-indicator" :class="{ syncing: isSyncing }"></span>
        <span>SQLite WAL Pool</span>
        <span class="status-divider">/</span>
        <span>Слов: <b>{{ currentWords }}</b></span>
        <span class="status-divider">/</span>
        <span>Всего: <b>{{ totalSystemWords }}</b></span>
        <span class="status-divider">/</span>
        <span>Ревизия: <b>r{{ currentRevision }}</b></span>
      </div>
      <div class="status-right">
        <span>{{ syncError || (isSyncing ? 'Синхронизация коммита...' : 'Система верифицирована') }}</span>
      </div>
    </footer>

    <MobileDrawerSheet
      :is-open="isMobile && tocOpen"
      title="Оглавление книги"
      @close="tocOpen = false"
    >
      <div class="mobile-toc-container">
        <div class="mobile-toc-progress-card">
          <div class="meter-info">
            <span>Прогресс произведения</span>
            <span>{{ goalPercent }}%</span>
          </div>
          <div class="meter-track">
            <div class="meter-fill" :style="{ width: `${goalPercent}%` }"></div>
          </div>
          <div class="meter-sub">{{ totalSystemWords }} из {{ targetBookGoal }} слов</div>
        </div>

        <button class="mobile-btn-create-draft" @click="initializeBlankDraft(); tocOpen = false; currentTab = 'studio'">
          + Создать новую сцену
        </button>

        <div class="mobile-chapters-list">
          <div
            v-for="(item, idx) in stories"
            :key="item.id"
            :class="['mobile-chapter-card', { active: item.id === activeId }]"
            @click="handleSelectStory(item)"
          >
            <span class="mobile-idx">{{ String(idx + 1).padStart(2, "0") }}</span>
            <div class="mobile-chap-details">
              <h4>{{ item.title }}</h4>
              <span>{{ item.category }} • r{{ item.revision }}</span>
            </div>
          </div>
        </div>
      </div>
    </MobileDrawerSheet>

    <MobileDrawerSheet
      :is-open="isMobile && mobileMenuOpen"
      title="Панель инструментов"
      @close="mobileMenuOpen = false"
    >
      <div class="mobile-menu-grid">
        <div class="menu-action-block">
          <h4>Навигация модулей</h4>
          <div class="menu-btn-row">
            <button class="menu-chip-btn" @click="currentTab = 'timeline'; mobileMenuOpen = false">Таймлайн</button>
            <button class="menu-chip-btn" @click="currentTab = 'graph'; mobileMenuOpen = false">Космос (Граф)</button>
            <button class="menu-chip-btn" @click="getTwist(); mobileMenuOpen = false">Генератор твиста</button>
          </div>
        </div>

        <div class="menu-action-block">
          <h4>Темы оформления</h4>
          <div class="menu-btn-row">
            <button :class="['theme-chip-btn', { active: currentTheme === 'obsidian' }]" @click="currentTheme = 'obsidian'">Dark</button>
            <button :class="['theme-chip-btn', { active: currentTheme === 'celestial' }]" @click="currentTheme = 'celestial'">Amber</button>
            <button :class="['theme-chip-btn', { active: currentTheme === 'sepia' }]" @click="currentTheme = 'sepia'">Sepia</button>
            <button :class="['theme-chip-btn', { active: currentTheme === 'paper' }]" @click="currentTheme = 'paper'">Paper</button>
            <button :class="['theme-chip-btn', { active: currentTheme === 'oled' }]" @click="currentTheme = 'oled'">OLED</button>
          </div>
        </div>

        <div class="menu-action-block">
          <h4>Экспорт книги</h4>
          <div class="menu-btn-row vertical">
            <button class="menu-export-btn" @click="exportStandaloneBook(); mobileMenuOpen = false">Экспорт издания (HTML)</button>
            <button class="menu-export-btn" @click="exportFullManuscript(); mobileMenuOpen = false">Экспорт рукописи (Markdown)</button>
          </div>
        </div>
      </div>
    </MobileDrawerSheet>
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

.export-toast-banner {
  position: fixed;
  top: max(14px, env(safe-area-inset-top));
  right: 14px;
  left: 14px;
  z-index: 999;
  background: rgba(16, 185, 129, 0.95);
  color: #fff;
  padding: 12px 16px;
  border-radius: 14px;
  font-size: 13px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: space-between;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.6);
  max-width: 520px;
  margin: 0 auto;
}

.export-toast-banner.error {
  background: rgba(244, 63, 94, 0.95);
}

.toast-close {
  background: transparent;
  border: none;
  color: #fff;
  cursor: pointer;
  font-size: 14px;
}

.mobile-master-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: max(10px, env(safe-area-inset-top)) 16px 10px 16px;
  background: var(--bg-surface);
  backdrop-filter: blur(28px);
  border-bottom: 1px solid var(--border);
  gap: 12px;
  flex-shrink: 0;
  z-index: 50;
}

.mobile-action-icon-btn {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border);
  color: var(--text-pure);
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.mobile-header-meta {
  flex: 1;
  overflow: hidden;
  text-align: center;
}

.mobile-title-text {
  font-size: 14px;
  font-weight: 800;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--text-pure);
}

.mobile-words-badge {
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--accent-cyan);
}

.mobile-save-pill {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.mobile-save-pill.saving {
  opacity: 0.6;
}

.drawer-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(10px);
  z-index: 150;
}

.sidebar-mobile-drawer {
  position: fixed;
  top: 0;
  left: -320px;
  width: 310px;
  height: 100%;
  height: 100dvh;
  background: var(--bg-surface-solid);
  border-right: 1px solid var(--border);
  z-index: 160;
  display: flex;
  flex-direction: column;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.9);
  transition: left 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.sidebar-mobile-drawer.open {
  left: 0;
}

.drawer-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}

.brand-crest {
  font-size: 15px;
  font-weight: 800;
  display: flex;
  align-items: center;
  gap: 8px;
}

.crest-icon {
  color: var(--accent-cyan);
}

.btn-drawer-close {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 16px;
}

.drawer-progress {
  padding: 14px 20px;
  border-bottom: 1px solid var(--border-subtle);
  background: rgba(0, 0, 0, 0.25);
}

.meter-info {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
}

.meter-track {
  height: 5px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 999px;
  margin: 6px 0;
  overflow: hidden;
}

.meter-fill {
  height: 100%;
  background: var(--accent-indigo);
  border-radius: 999px;
}

.meter-sub {
  font-size: 10px;
  color: var(--text-dim);
}

.drawer-scrollable-list {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.drawer-chapter-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 10px;
  cursor: pointer;
}

.drawer-chapter-row.active {
  background: var(--bg-card);
  border: 1px solid var(--border-accent);
}

.chapter-idx {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-dim);
}

.chapter-info {
  flex: 1;
  overflow: hidden;
}

.chapter-title {
  display: block;
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.chapter-cat {
  font-size: 10px;
  color: var(--text-muted);
}

.drawer-footer {
  padding: 14px;
  border-top: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.btn-drawer-action {
  width: 100%;
  height: 40px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--bg-card);
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  color: var(--text-main);
}

.btn-drawer-action.primary {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
}

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
}

.theme-picker-segmented {
  display: flex;
  gap: 2px;
  background: rgba(0, 0, 0, 0.4);
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
  background: var(--bg-card);
  color: var(--text-pure);
}

.layout-segmented {
  display: flex;
  gap: 2px;
  background: rgba(0, 0, 0, 0.4);
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
  background: var(--bg-card);
  color: var(--text-pure);
}

.subtool-right-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.btn-quick-export {
  background: rgba(255, 255, 255, 0.05);
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
  background: rgba(255, 255, 255, 0.12);
  border-color: var(--accent-cyan);
}

.btn-generator-spark {
  background: transparent;
  border: 1px solid var(--accent-indigo);
  color: var(--accent-cyan);
  border-radius: 8px;
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}

.btn-quick-save {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  border-radius: 8px;
  padding: 4px 12px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}

.btn-quick-save:disabled {
  opacity: 0.6;
}

.twist-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: rgba(99, 102, 241, 0.15);
  border-bottom: 1px solid var(--accent-indigo);
  font-size: 12px;
}

.twist-tag {
  font-size: 9px;
  font-weight: 900;
  background: var(--accent-indigo);
  color: #fff;
  padding: 2px 6px;
  border-radius: 4px;
}

.twist-text {
  flex: 1;
  margin: 0 10px;
}

.twist-action-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.twist-insert-btn {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  font-size: 11px;
  font-weight: 700;
  padding: 4px 10px;
  border-radius: 6px;
  cursor: pointer;
}

.twist-dismiss {
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--text-muted);
}

.workbench-main-stage {
  flex: 1;
  display: flex;
  overflow: hidden;
  position: relative;
  min-height: 0;
}

.system-statusbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 24px;
  padding: 0 12px;
  background: var(--bg-surface-solid);
  border-top: 1px solid var(--border);
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--text-muted);
}

.status-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-indicator {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent-emerald);
}

.status-indicator.syncing {
  background: var(--accent-gold);
}

.status-divider {
  color: var(--border);
}

.status-right {
  color: var(--text-dim);
}

.mobile-toc-container {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.mobile-toc-progress-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 14px;
}

.mobile-btn-create-draft {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  border-radius: 12px;
  padding: 12px;
  font-weight: 800;
  font-size: 13px;
  cursor: pointer;
}

.mobile-chapters-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.mobile-chapter-card {
  display: flex;
  align-items: center;
  gap: 12px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 12px;
  cursor: pointer;
}

.mobile-chapter-card.active {
  border-color: var(--accent-indigo);
  background: var(--bg-card-hover);
}

.mobile-idx {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--accent-cyan);
}

.mobile-chap-details h4 {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-pure);
}

.mobile-chap-details span {
  font-size: 10px;
  color: var(--text-muted);
}

.mobile-menu-grid {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.menu-action-block h4 {
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--text-muted);
  margin-bottom: 10px;
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
  padding: 10px 14px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.theme-chip-btn {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-muted);
  border-radius: 8px;
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
  padding: 12px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  text-align: center;
}
</style>