<script setup lang="ts">
import MobileDrawerSheet from "./MobileDrawerSheet.vue";
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
}>();

function moveUp(index: number) {
  if (index <= 0) return;
  const a = stories[index];
  const b = stories[index - 1];
  emit("reorder", [
    { id: a.id, sortOrder: b.sortOrder, category: a.category },
    { id: b.id, sortOrder: a.sortOrder, category: b.category },
  ]);
}

function moveDown(index: number) {
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
  <MobileDrawerSheet
    :is-open="isOpen"
    title="Оглавление книги"
    @close="emit('close')"
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

      <button class="mobile-btn-create-draft" @click="emit('newDraft')">
        + Создать новую сцену
      </button>

      <div class="mobile-chapters-list">
        <div
          v-for="(item, idx) in stories"
          :key="item.id"
          :class="['mobile-chapter-card', { active: item.id === activeId }]"
          @click="emit('selectStory', item)"
        >
          <span class="mobile-idx">{{ String(idx + 1).padStart(2, "0") }}</span>
          <div class="mobile-chap-details">
            <h4>{{ item.title || 'Сцена без названия' }}</h4>
            <span>{{ item.category }} • r{{ item.revision }}</span>
          </div>
          <div class="reorder-controls-mobile" @click.stop>
            <button class="btn-reorder-touch" :disabled="idx === 0" @click="moveUp(idx)">▲</button>
            <button class="btn-reorder-touch" :disabled="idx === stories.length - 1" @click="moveDown(idx)">▼</button>
          </div>
        </div>
      </div>
    </div>
  </MobileDrawerSheet>
</template>

<style scoped>
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
  gap: 10px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 10px 12px;
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

.mobile-chap-details {
  flex: 1;
  overflow: hidden;
}

.mobile-chap-details h4 {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-pure);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mobile-chap-details span {
  font-size: 10px;
  color: var(--text-muted);
}

.reorder-controls-mobile {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.btn-reorder-touch {
  background: var(--bg-input);
  border: 1px solid var(--border);
  color: var(--text-muted);
  width: 28px;
  height: 22px;
  border-radius: 6px;
  font-size: 9px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-reorder-touch:disabled {
  opacity: 0.25;
}
</style>