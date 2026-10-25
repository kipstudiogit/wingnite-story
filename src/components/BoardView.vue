<script setup lang="ts">
import { computed } from "vue";
import type { StoryItem, StoryOrderUpdate } from "../types";
import { useHaptics } from "../composables/useHaptics";

const { actsList, stories } = defineProps<{
  actsList: string[];
  stories: StoryItem[];
}>();

const emit = defineEmits<{
  (e: "select", story: StoryItem): void;
  (e: "create", category: string): void;
  (e: "reorder", updates: StoryOrderUpdate[]): void;
}>();

const { vibrateLight, vibrateSelection, vibrateSuccess } = useHaptics();

interface ProcessedStoryItem extends StoryItem {
  wordCount: number;
  excerpt: string;
  progressPercent: number;
}

function countWords(str: string): number {
  let count = 0;
  let inWord = false;
  for (let i = 0; i < str.length; i++) {
    if (str.charCodeAt(i) > 32) {
      if (!inWord) {
        inWord = true;
        count++;
      }
    } else {
      inWord = false;
    }
  }
  return count;
}

const resolvedActs = computed(() => {
  if (actsList.length > 0) return actsList;
  return ["Акт I", "Акт II", "Акт III", "Кульминация"];
});

const storiesByAct = computed(() => {
  const map = new Map<string, ProcessedStoryItem[]>();
  for (const act of resolvedActs.value) {
    map.set(act, []);
  }
  for (const s of stories) {
    const targetKey = map.has(s.category) ? s.category : resolvedActs.value[0];
    const list = map.get(targetKey);
    if (list) {
      const words = countWords(s.content);
      list.push({
        ...s,
        wordCount: words,
        excerpt: s.content.slice(0, 140).replace(/[#*`>]/g, "").trim(),
        progressPercent: Math.min(100, Math.round((words / 2000) * 100)),
      });
    }
  }
  for (const list of map.values()) {
    list.sort((a, b) => a.sortOrder - b.sortOrder);
  }
  return map;
});

function moveCardUp(act: string, index: number) {
  const list = storiesByAct.value.get(act);
  if (!list || index <= 0) return;
  const itemA = list[index];
  const itemB = list[index - 1];

  const updates: StoryOrderUpdate[] = [
    { id: itemA.id, sortOrder: itemB.sortOrder, category: act },
    { id: itemB.id, sortOrder: itemA.sortOrder, category: act },
  ];
  vibrateSuccess();
  emit("reorder", updates);
}

function moveCardDown(act: string, index: number) {
  const list = storiesByAct.value.get(act);
  if (!list || index >= list.length - 1) return;
  const itemA = list[index];
  const itemB = list[index + 1];

  const updates: StoryOrderUpdate[] = [
    { id: itemA.id, sortOrder: itemB.sortOrder, category: act },
    { id: itemB.id, sortOrder: itemA.sortOrder, category: act },
  ];
  vibrateSuccess();
  emit("reorder", updates);
}

function handleActMove(story: StoryItem, targetAct: string) {
  const targetList = storiesByAct.value.get(targetAct) || [];
  const maxOrder = targetList.reduce((max, s) => Math.max(max, s.sortOrder), 0);
  const updates: StoryOrderUpdate[] = [
    { id: story.id, sortOrder: maxOrder + 1, category: targetAct },
  ];
  vibrateSuccess();
  emit("reorder", updates);
}
</script>

<template>
  <section class="stage-board">
    <div class="board-acts-track">
      <div v-for="act in resolvedActs" :key="act" class="board-act-column">
        <header class="act-col-header">
          <div class="act-title-wrapper">
            <span class="act-status-flame"></span>
            <h3 class="act-name">{{ act }}</h3>
          </div>
          <div class="act-header-right">
            <span class="act-badge-counter">{{ storiesByAct.get(act)?.length ?? 0 }}</span>
            <button class="btn-act-add" title="Добавить сцену в этот акт" @click="vibrateLight(); emit('create', act)">+</button>
          </div>
        </header>

        <div class="act-cards-scroller">
          <div
            v-for="(item, idx) in (storiesByAct.get(act) ?? [])"
            :key="item.id"
            class="prismatic-card"
            @click="vibrateSelection(); emit('select', item)"
          >
            <div class="card-top-meta">
              <span class="card-act-tag">{{ item.category }}</span>
              <div class="card-reorder-pills" @click.stop>
                <button
                  class="btn-order-arrow"
                  :disabled="idx === 0"
                  title="Поднять выше"
                  @click="moveCardUp(act, idx)"
                >
                  ▲
                </button>
                <button
                  class="btn-order-arrow"
                  :disabled="idx === (storiesByAct.get(act)?.length ?? 0) - 1"
                  title="Опустить ниже"
                  @click="moveCardDown(act, idx)"
                >
                  ▼
                </button>
              </div>
            </div>

            <h4 class="card-heading">{{ item.title || 'Сцена без названия' }}</h4>
            <p class="card-snippet">{{ item.excerpt || 'Текст сцены...' }}...</p>

            <div class="card-bottom-telemetry">
              <div class="card-meter-track">
                <div class="card-meter-fill" :style="{ width: `${item.progressPercent}%` }"></div>
              </div>
              <span class="card-meter-label">{{ item.wordCount }} сл.</span>
            </div>

            <div class="card-mobile-act-mover" @click.stop>
              <select
                class="select-act-shift"
                :value="item.category"
                @change="handleActMove(item, ($event.target as HTMLSelectElement).value)"
              >
                <option v-for="targetAct in resolvedActs" :key="targetAct" :value="targetAct">
                  Переместить: {{ targetAct }}
                </option>
              </select>
            </div>
          </div>

          <div v-if="(storiesByAct.get(act) ?? []).length === 0" class="empty-column-box">
            <p>Нет сцен в блоке</p>
            <button class="btn-create-card-inline" @click="vibrateLight(); emit('create', act)">+ Новая сцена</button>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.stage-board {
  flex: 1;
  overflow-x: auto;
  padding: 32px;
  display: flex;
}

.board-acts-track {
  display: flex;
  gap: 26px;
  height: 100%;
  align-items: flex-start;
}

.board-act-column {
  width: 350px;
  background: var(--bg-surface-solid);
  border: 1px solid var(--border);
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.6);
  border-radius: 18px;
  display: flex;
  flex-direction: column;
  max-height: 100%;
  flex-shrink: 0;
  overflow: hidden;
}

.act-col-header {
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: rgba(255, 255, 255, 0.02);
}

.act-title-wrapper {
  display: flex;
  align-items: center;
  gap: 10px;
}

.act-status-flame {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent-cyan);
  box-shadow: 0 0 14px var(--accent-cyan-glow);
}

.act-name {
  font-size: 14px;
  font-weight: 800;
  color: var(--text-main);
}

.act-header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.act-badge-counter {
  background: var(--bg-card);
  border: 1px solid var(--border);
  padding: 3px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  font-family: var(--font-mono);
  color: var(--text-muted);
}

.btn-act-add {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  border-radius: 6px;
  width: 26px;
  height: 26px;
  font-weight: 800;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.act-cards-scroller {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  overflow-y: auto;
}

.prismatic-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 16px 18px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 8px;
  transition: all 0.2s ease;
}

.prismatic-card:hover {
  border-color: var(--border-accent);
  transform: translateY(-2px);
}

.card-top-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-act-tag {
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 0.07em;
  font-size: 10px;
  font-weight: 800;
}

.card-reorder-pills {
  display: flex;
  gap: 4px;
}

.btn-order-arrow {
  background: var(--bg-input);
  border: 1px solid var(--border-subtle);
  color: var(--text-muted);
  width: 24px;
  height: 20px;
  border-radius: 4px;
  font-size: 9px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-order-arrow:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.btn-order-arrow:not(:disabled):hover {
  background: var(--accent-indigo);
  color: #fff;
}

.card-heading {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-pure);
  line-height: 1.35;
}

.card-snippet {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
}

.card-bottom-telemetry {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 4px;
}

.card-meter-track {
  flex: 1;
  height: 4px;
  background: var(--border-subtle);
  border-radius: 999px;
  overflow: hidden;
}

.card-meter-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--accent-indigo) 0%, var(--accent-cyan) 100%);
}

.card-meter-label {
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--text-dim);
}

.card-mobile-act-mover {
  display: flex;
  justify-content: flex-end;
  margin-top: 2px;
}

.select-act-shift {
  background: var(--bg-input);
  border: 1px solid var(--border);
  color: var(--text-muted);
  border-radius: 6px;
  font-size: 10px;
  font-weight: 700;
  padding: 3px 6px;
  outline: none;
  cursor: pointer;
}

.empty-column-box {
  padding: 30px 10px;
  text-align: center;
  color: var(--text-dim);
  font-size: 12px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.btn-create-card-inline {
  background: transparent;
  border: 1px dashed var(--border);
  color: var(--accent-cyan);
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}
</style>