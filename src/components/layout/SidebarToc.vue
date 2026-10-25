<script setup lang="ts">
import type { EnterpriseStoryItem, StoryOrderUpdate } from "../../types";

const {
  isOpen,
  stories,
  activeId,
  goalPercent,
  totalSystemWords,
  targetBookGoal,
} = defineProps<{
  isOpen: boolean;
  stories: EnterpriseStoryItem[];
  activeId: string;
  goalPercent: number;
  totalSystemWords: number;
  targetBookGoal: number;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "selectStory", story: EnterpriseStoryItem): void;
  (e: "newDraft"): void;
  (e: "reorder", updates: StoryOrderUpdate[]): void;
  (e: "exportWstory"): void;
  (e: "importWstory"): void;
  (e: "exportHtml"): void;
  (e: "exportMd"): void;
  (e: "exportTxt"): void;
  (e: "importTxt"): void;
}>();

function moveStoryUp(index: number) {
  if (index <= 0) return;
  const a = stories[index];
  const b = stories[index - 1];
  emit("reorder", [
    { id: a.id, sortOrder: b.sortOrder, category: a.category },
    { id: b.id, sortOrder: a.sortOrder, category: b.category },
  ]);
}

function moveStoryDown(index: number) {
  if (index >= stories.length - 1) return;
  const a = stories[index];
  const b = stories[index + 1];
  emit("reorder", [
    { id: a.id, sortOrder: b.sortOrder, category: a.category },
    { id: b.id, sortOrder: a.sortOrder, category: b.category },
  ]);
}
</script>

<template>
  <div v-if="isOpen" class="drawer-backdrop" @click="emit('close')"></div>

  <aside :class="['sidebar-mobile-drawer', { open: isOpen }]">
    <div class="drawer-header">
      <div class="brand-crest">
        <svg class="crest-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.24 12.24a6 6 0 0 0-8.49-8.49L5 10.5V19h8.5z"/><line x1="16" y1="8" x2="2" y2="22"/><line x1="17.5" y1="15" x2="9" y2="15"/></svg>
        <span>Оглавление книги</span>
      </div>
      <button class="btn-drawer-close" title="Закрыть" @click="emit('close')">✕</button>
    </div>

    <div class="drawer-progress">
      <div class="meter-info">
        <span>Прогресс рукописи</span>
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
        @click="emit('selectStory', item)"
      >
        <span class="chapter-idx">{{ String(idx + 1).padStart(2, "0") }}</span>
        <div class="chapter-info">
          <span class="chapter-title">{{ item.title || 'Сцена без названия' }}</span>
          <span class="chapter-cat">{{ item.category }} • r{{ item.revision }}</span>
        </div>
        <div class="chapter-reorder-pills" @click.stop>
          <button class="btn-toc-arrow" :disabled="idx === 0" title="Поднять сцену выше" @click="moveStoryUp(idx)">▲</button>
          <button class="btn-toc-arrow" :disabled="idx === stories.length - 1" title="Опустить сцену ниже" @click="moveStoryDown(idx)">▼</button>
        </div>
      </div>

      <div v-if="stories.length === 0" class="toc-empty">
        <span>В книге пока нет сцен</span>
      </div>
    </div>

    <div class="drawer-footer">
      <button class="btn-drawer-action primary" @click="emit('newDraft')">
        <span>+ Новая сцена</span>
      </button>
      <div class="footer-btn-grid">
        <button class="btn-drawer-sub" @click="emit('importTxt')">Импорт .TXT</button>
        <button class="btn-drawer-sub" @click="emit('exportTxt')">Экспорт .TXT</button>
        <button class="btn-drawer-sub" @click="emit('importWstory')">Импорт .WSTORY</button>
        <button class="btn-drawer-sub" @click="emit('exportWstory')">Экспорт .WSTORY</button>
      </div>
      <button class="btn-drawer-action" @click="emit('exportHtml')">
        <span>Собрать издание (HTML)</span>
      </button>
      <button class="btn-drawer-action" @click="emit('exportMd')">
        <span>Экспорт всей книги (Markdown)</span>
      </button>
    </div>
  </aside>
</template>

<style scoped>
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
  left: -340px;
  width: 330px;
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
  font-size: 14px;
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
  background: var(--bg-card);
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
  background: var(--border-subtle);
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
  gap: 10px;
  padding: 8px 10px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s ease;
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
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.chapter-cat {
  font-size: 10px;
  color: var(--text-muted);
}

.chapter-reorder-pills {
  display: flex;
  gap: 3px;
}

.btn-toc-arrow {
  background: var(--bg-input);
  border: 1px solid var(--border-subtle);
  color: var(--text-muted);
  width: 22px;
  height: 20px;
  border-radius: 4px;
  font-size: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-toc-arrow:disabled {
  opacity: 0.25;
  cursor: not-allowed;
}

.btn-toc-arrow:not(:disabled):hover {
  background: var(--accent-indigo);
  color: #fff;
}

.toc-empty {
  padding: 40px 10px;
  text-align: center;
  color: var(--text-dim);
  font-size: 12px;
}

.drawer-footer {
  padding: 14px;
  border-top: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.footer-btn-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
}

.btn-drawer-sub {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-main);
  padding: 6px;
  border-radius: 6px;
  font-size: 10px;
  font-weight: 700;
  cursor: pointer;
}

.btn-drawer-action {
  width: 100%;
  height: 36px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--bg-card);
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  color: var(--text-main);
}

.btn-drawer-action.primary {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
}
</style>