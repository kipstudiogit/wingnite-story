<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useStories } from "./composables/useStories";
import { useSprint } from "./composables/useSprint";
import type { AppTab, StudioViewMode, ThemeMode, StoryItem } from "./types";

import SplashScreen from "./components/SplashScreen.vue";
import Navigation from "./components/Navigation.vue";
import ReaderView from "./components/ReaderView.vue";
import StudioView from "./components/StudioView.vue";
import BoardView from "./components/BoardView.vue";
import CodexView from "./components/CodexView.vue";
import GraphView from "./components/GraphView.vue";
import TimelineView from "./components/TimelineView.vue";
import CommandPalette from "./components/CommandPalette.vue";

const {
  stories,
  snapshots,
  activeId,
  currentTitle,
  currentCategory,
  currentContent,
  statusMsg,
  wordCount,
  totalBookWords,
  actsList,
  loadStories,
  selectStory,
  saveCurrentStory,
  deleteCurrentStory,
  makeSnapshot,
  restoreSnapshot,
  createNewStory,
  createDivineEntity,
  exportFullManuscript,
} = useStories();

const {
  sprintActive,
  sprintDelta,
  sprintFormattedTime,
  toggleSprint,
  cleanupSprint,
} = useSprint(() => wordCount.value);

const isBackendLoaded = ref(false);
const showSplash = ref(true);

const currentTab = ref<AppTab>("studio");
const currentTheme = ref<ThemeMode>("obsidian");
const studioMode = ref<StudioViewMode>("split");
const fontSize = ref<number>(18);
const targetBookGoal = ref<number>(25000);
const plotTwist = ref<string>("");
const tocOpen = ref<boolean>(false);
const paletteOpen = ref<boolean>(false);

const goalPercent = computed(() => {
  return Math.min(100, Math.round((totalBookWords.value / targetBookGoal.value) * 100));
});

const readingTime = computed(() => Math.max(1, Math.ceil(wordCount.value / 200)));

async function initApplicationData() {
  await loadStories();
  isBackendLoaded.value = true;
}

async function getTwist() {
  try {
    plotTwist.value = await invoke<string>("generate_twist");
  } catch (err) {
    statusMsg.value = String(err);
  }
}

async function exportStandaloneBook() {
  try {
    const htmlBook = await invoke<string>("compile_book_edition");
    const blob = new Blob([htmlBook], { type: "text/html;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "Wingnite_Book_Edition.html";
    a.click();
    URL.revokeObjectURL(url);
    statusMsg.value = "Книга скомпилирована в HTML";
  } catch (err) {
    statusMsg.value = String(err);
  }
}

function handleSelectStory(story: StoryItem) {
  selectStory(story);
  tocOpen.value = false;
  paletteOpen.value = false;
}

function handleEntityCreation(type: "artifact" | "pantheon" | "sanctuary") {
  createDivineEntity(type);
  currentTab.value = "studio";
}

function handleKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
    e.preventDefault();
    paletteOpen.value = !paletteOpen.value;
  }
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
    e.preventDefault();
    saveCurrentStory();
  }
  if (e.key === "Escape") {
    paletteOpen.value = false;
    tocOpen.value = false;
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
  <div :class="['studio-app-shell', `theme-${currentTheme}`]">
    <div class="cyber-grid-overlay"></div>

    <transition name="splash-dissolve">
      <SplashScreen
        v-if="showSplash"
        :is-ready="isBackendLoaded"
        @finish="showSplash = false"
      />
    </transition>

    <CommandPalette
      v-if="paletteOpen"
      :stories="stories"
      @select="handleSelectStory"
      @close="paletteOpen = false"
    />

    <div v-if="tocOpen" class="drawer-backdrop" @click="tocOpen = false"></div>

    <aside :class="['sidebar-mobile-drawer', { open: tocOpen }]">
      <div class="drawer-header">
        <div class="brand-crest">
          <svg class="crest-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M20.24 12.24a6 6 0 0 0-8.49-8.49L5 10.5V19h8.5z"/>
            <line x1="16" y1="8" x2="2" y2="22"/>
            <line x1="17.5" y1="15" x2="9" y2="15"/>
          </svg>
          <span>Wingnite Studio</span>
        </div>
        <button class="btn-drawer-close" title="Закрыть" @click="tocOpen = false">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <div class="drawer-progress">
        <div class="meter-info">
          <span>Прогресс романа</span>
          <span>{{ goalPercent }}%</span>
        </div>
        <div class="meter-track">
          <div class="meter-fill" :style="{ width: `${goalPercent}%` }"></div>
        </div>
        <div class="meter-sub">{{ totalBookWords }} / {{ targetBookGoal }} слов</div>
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
            <span class="chapter-cat">{{ item.category }}</span>
          </div>
        </div>
      </div>

      <div class="drawer-footer">
        <button
          class="btn-drawer-action primary"
          @click="createNewStory(); currentTab = 'studio'; tocOpen = false"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"/>
            <line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          <span>Новая сцена</span>
        </button>
        <button class="btn-drawer-action" @click="exportStandaloneBook">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/>
            <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>
          </svg>
          <span>Издание книги (HTML)</span>
        </button>
        <button class="btn-drawer-action" @click="exportFullManuscript">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          <span>Экспорт Markdown</span>
        </button>
      </div>
    </aside>

    <Navigation
      v-model:current-tab="currentTab"
      :sprint-active="sprintActive"
      :sprint-formatted-time="sprintFormattedTime"
      :sprint-delta="sprintDelta"
      @toggle-toc="tocOpen = !tocOpen"
      @open-palette="paletteOpen = true"
      @toggle-sprint="toggleSprint"
    />

    <div class="workbench-subtoolbar">
      <div class="subtoolbar-scroller">
        <div class="subtool-left">
          <div class="zoom-stepper">
            <button class="btn-stepper" title="Уменьшить шрифт" @click="fontSize = Math.max(13, fontSize - 1)">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <line x1="5" y1="12" x2="19" y2="12"/>
              </svg>
            </button>
            <span class="stepper-label">{{ fontSize }}px</span>
            <button class="btn-stepper" title="Увеличить шрифт" @click="fontSize = Math.min(28, fontSize + 1)">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <line x1="12" y1="5" x2="12" y2="19"/>
                <line x1="5" y1="12" x2="19" y2="12"/>
              </svg>
            </button>
          </div>

          <div class="theme-picker-segmented">
            <button
              :class="['theme-dot-btn', { active: currentTheme === 'obsidian' }]"
              title="Obsidian Dark"
              @click="currentTheme = 'obsidian'"
            >
              Dark
            </button>
            <button
              :class="['theme-dot-btn', { active: currentTheme === 'celestial' }]"
              title="Celestial Amber"
              @click="currentTheme = 'celestial'"
            >
              Amber
            </button>
            <button
              :class="['theme-dot-btn', { active: currentTheme === 'sepia' }]"
              title="Vintage Sepia"
              @click="currentTheme = 'sepia'"
            >
              Sepia
            </button>
            <button
              :class="['theme-dot-btn', { active: currentTheme === 'paper' }]"
              title="Clean Paper"
              @click="currentTheme = 'paper'"
            >
              Paper
            </button>
            <button
              :class="['theme-dot-btn', { active: currentTheme === 'oled' }]"
              title="Pure OLED"
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
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
              <line x1="12" y1="3" x2="12" y2="21"/>
            </svg>
            <span>Сплит</span>
          </button>
          <button
            :class="['btn-layout', { active: studioMode === 'edit' }]"
            @click="studioMode = 'edit'"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
            </svg>
            <span>Редактор</span>
          </button>
          <button
            :class="['btn-layout', { active: studioMode === 'zen' }]"
            @click="studioMode = 'zen'"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="3"/>
              <path d="M12 2v2m0 16v2M4.93 4.93l1.41 1.41m11.32 11.32 1.41 1.41M2 12h2m16 0h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/>
            </svg>
            <span>Фокус</span>
          </button>
        </div>

        <button class="btn-generator-spark" title="Сгенерировать поворот" @click="getTwist">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="m12 3-1.9 5.8a2 2 0 0 1-1.3 1.3L3 12l5.8 1.9a2 2 0 0 1 1.3 1.3L12 21l1.9-5.8a2 2 0 0 1 1.3-1.3L21 12l-5.8-1.9a2 2 0 0 1-1.3-1.3Z"/>
          </svg>
          <span>Твист</span>
        </button>
      </div>
    </div>

    <div v-if="plotTwist" class="twist-banner">
      <div class="twist-tag">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
        </svg>
        <span>ТВИСТ</span>
      </div>
      <p class="twist-text">{{ plotTwist }}</p>
      <button class="twist-dismiss" title="Закрыть" @click="plotTwist = ''">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <main class="workbench-main-stage">
      <ReaderView
        v-if="currentTab === 'reader'"
        :title="currentTitle"
        :category="currentCategory"
        :content="currentContent"
        :font-size="fontSize"
      />

      <StudioView
        v-else-if="currentTab === 'studio'"
        v-model:title="currentTitle"
        v-model:category="currentCategory"
        v-model:content="currentContent"
        :studio-mode="studioMode"
        :font-size="fontSize"
        :has-active="!!activeId"
        :snapshots="snapshots"
        @save="saveCurrentStory"
        @delete="deleteCurrentStory"
        @make-snapshot="makeSnapshot"
        @restore-snapshot="restoreSnapshot"
      />

      <BoardView
        v-else-if="currentTab === 'board'"
        :acts-list="actsList"
        :stories="stories"
        @select="handleSelectStory($event); currentTab = 'studio'"
      />

      <CodexView
        v-else-if="currentTab === 'codex'"
        :stories="stories"
        @select="handleSelectStory($event); currentTab = 'studio'"
        @create-entity="handleEntityCreation"
      />

      <TimelineView
        v-else-if="currentTab === 'timeline'"
        :stories="stories"
        @select-story="handleSelectStory($event); currentTab = 'studio'"
      />

      <GraphView
        v-else-if="currentTab === 'graph'"
        :stories="stories"
        @select="handleSelectStory($event); currentTab = 'studio'"
      />
    </main>

    <footer class="system-statusbar">
      <div class="status-left">
        <span class="status-indicator"></span>
        <span>SQLite WAL + FTS5</span>
        <span class="status-divider">/</span>
        <span>Слов: <b>{{ wordCount }}</b></span>
        <span class="status-divider">/</span>
        <span>Чтение: <b>~{{ readingTime }} мин</b></span>
      </div>
      <div class="status-right">
        <span>{{ statusMsg || 'Все системы активны' }}</span>
      </div>
    </footer>
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

.splash-dissolve-leave-active {
  transition: opacity 0.45s cubic-bezier(0.16, 1, 0.3, 1), transform 0.45s cubic-bezier(0.16, 1, 0.3, 1), filter 0.45s ease;
}

.splash-dissolve-leave-to {
  opacity: 0;
  transform: scale(1.06);
  filter: blur(12px);
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
  padding-top: var(--safe-top);
  padding-bottom: var(--safe-bottom);
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
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  cursor: pointer;
  color: var(--text-muted);
}

.btn-drawer-close:hover {
  color: var(--text-pure);
  background: rgba(255, 255, 255, 0.08);
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
  border: 1px solid transparent;
  transition: all 0.15s ease;
}

.drawer-chapter-row:hover {
  background: rgba(255, 255, 255, 0.04);
}

.drawer-chapter-row.active {
  background: var(--bg-card);
  border-color: var(--border-accent);
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
  height: 42px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--bg-card);
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  color: var(--text-main);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.btn-drawer-action:hover {
  border-color: var(--border-strong);
  background: var(--bg-card-hover);
}

.btn-drawer-action.primary {
  background: linear-gradient(135deg, var(--accent-indigo) 0%, #4338ca 100%);
  border: none;
  color: #fff;
  box-shadow: 0 4px 16px var(--accent-indigo-glow);
}

.workbench-subtoolbar {
  display: flex;
  align-items: center;
  background: var(--bg-surface-solid);
  border-bottom: 1px solid var(--border);
  overflow: hidden;
  flex-shrink: 0;
  height: 38px;
}

.subtoolbar-scroller {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 0 14px;
  gap: 10px;
  overflow-x: auto;
  scrollbar-width: none;
}

.subtoolbar-scroller::-webkit-scrollbar {
  display: none;
}

.subtool-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
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
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: var(--text-muted);
}

.btn-stepper:hover {
  color: var(--text-pure);
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
  padding: 4px 8px;
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
  flex-shrink: 0;
}

.btn-layout {
  background: transparent;
  border: none;
  font-size: 11px;
  font-weight: 600;
  padding: 4px 8px;
  border-radius: 6px;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
}

.btn-layout.active {
  background: var(--bg-card);
  color: var(--text-pure);
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
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
  flex-shrink: 0;
}

.twist-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: rgba(99, 102, 241, 0.15);
  border-bottom: 1px solid var(--accent-indigo);
  font-size: 12px;
  gap: 12px;
  flex-shrink: 0;
}

.twist-tag {
  font-size: 9px;
  font-weight: 900;
  background: var(--accent-indigo);
  color: #fff;
  padding: 2px 6px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.twist-text {
  flex: 1;
}

.twist-dismiss {
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
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
  padding: 0 12px calc(var(--safe-bottom)) 12px;
  background: var(--bg-surface-solid);
  border-top: 1px solid var(--border);
  font-size: 10px;
  color: var(--text-muted);
  font-family: var(--font-mono);
  flex-shrink: 0;
}

.status-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-divider {
  color: var(--border);
}

.status-indicator {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent-emerald);
}

.status-right {
  font-family: var(--font-mono);
  color: var(--text-dim);
}

@media (max-width: 768px) {
  .layout-segmented {
    display: none;
  }

  .system-statusbar {
    display: none;
  }

  .workbench-main-stage {
    padding-bottom: calc(var(--safe-bottom) + 68px);
  }
}
</style>