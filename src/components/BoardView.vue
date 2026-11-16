<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { computed } from "vue";
import { isCodexEntityCategory } from "../constants/app";
import type { StoryItem } from "../types";
import { useHaptics } from "../composables/useHaptics";
import { useI18n } from "../i18n";

const { actsList, stories } = defineProps<{
  actsList: string[];
  stories: StoryItem[];
}>();

const emit = defineEmits<{
  (e: "select", story: StoryItem): void;
  (e: "create", category: string): void;
  (e: "swap", idA: string, idB: string): void;
  (e: "moveCategory", storyId: string, targetCategory: string): void;
}>();

const { vibrateLight, vibrateSelection, vibrateSuccess } = useHaptics();
const { t, messages } = useI18n();

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
  const defaults = messages.value.board.defaultActs;
  const narrativeActs = actsList.filter((a) => !isCodexEntityCategory(a));
  return Array.from(new Set<string>([...defaults, ...narrativeActs]));
});

const storiesByAct = computed(() => {
  const map = new Map<string, ProcessedStoryItem[]>();
  for (const act of resolvedActs.value) {
    map.set(act, []);
  }
  for (const s of stories) {
    if (isCodexEntityCategory(s.category)) {
      continue;
    }
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
  vibrateSuccess();
  emit("swap", list[index].id, list[index - 1].id);
}

function moveCardDown(act: string, index: number) {
  const list = storiesByAct.value.get(act);
  if (!list || index >= list.length - 1) return;
  vibrateSuccess();
  emit("swap", list[index].id, list[index + 1].id);
}

function handleActMove(story: StoryItem, targetAct: string) {
  vibrateSuccess();
  emit("moveCategory", story.id, targetAct);
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
            <button class="btn-act-add" :title="t('board.addSceneTooltip')" @click="vibrateLight(); emit('create', act)">+</button>
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
                  :title="t('board.moveUpTooltip')"
                  @click="moveCardUp(act, idx)"
                >
                  ▲
                </button>
                <button
                  class="btn-order-arrow"
                  :disabled="idx === (storiesByAct.get(act)?.length ?? 0) - 1"
                  :title="t('board.moveDownTooltip')"
                  @click="moveCardDown(act, idx)"
                >
                  ▼
                </button>
              </div>
            </div>

            <h4 class="card-heading">{{ item.title || t("common.untitled") }}</h4>
            <p class="card-snippet">{{ item.excerpt || "..." }}</p>

            <div class="card-bottom-telemetry">
              <div class="card-meter-track">
                <div class="card-meter-fill" :style="{ width: `${item.progressPercent}%` }"></div>
              </div>
              <span class="card-meter-label">{{ item.wordCount }} {{ t("board.wordsShort") }}</span>
            </div>

            <div class="card-mobile-act-mover" @click.stop>
              <select
                class="select-act-shift"
                :value="item.category"
                @change="handleActMove(item, ($event.target as HTMLSelectElement).value)"
              >
                <option v-for="targetAct in resolvedActs" :key="targetAct" :value="targetAct">
                  {{ t("board.moveToPrefix") }} {{ targetAct }}
                </option>
              </select>
            </div>
          </div>

          <div v-if="(storiesByAct.get(act)?.length ?? 0) === 0" class="empty-column-box">
            <p>{{ t("board.emptyColumn") }}</p>
            <button class="btn-create-card-inline" @click="vibrateLight(); emit('create', act)">
              {{ t("board.addInline") }}
            </button>
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
  overflow-y: hidden;
  padding: 24px max(24px, var(--safe-right)) 24px max(24px, var(--safe-left));
  display: flex;
  -webkit-overflow-scrolling: touch;
}

.board-acts-track {
  display: flex;
  gap: 20px;
  height: 100%;
  align-items: flex-start;
}

.board-act-column {
  width: 340px;
  background: var(--bg-surface-solid);
  border: 1px solid var(--border);
  box-shadow: var(--shadow-quantum);
  border-radius: 18px;
  display: flex;
  flex-direction: column;
  max-height: 100%;
  flex-shrink: 0;
  overflow: hidden;
}

.act-col-header {
  padding: 14px 18px;
  border-bottom: 1px solid var(--border);
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--bg-card);
}

.act-title-wrapper {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.act-status-flame {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent-cyan);
  box-shadow: 0 0 12px var(--accent-cyan-glow);
  flex-shrink: 0;
}

.act-name {
  font-size: 14px;
  font-weight: 800;
  color: var(--text-main);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.act-header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.act-badge-counter {
  background: var(--bg-input);
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
  border-radius: 8px;
  width: 28px;
  height: 28px;
  font-weight: 800;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.act-cards-scroller {
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow-y: auto;
  overscroll-behavior-y: contain;
}

.prismatic-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 14px 16px;
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
  width: 28px;
  height: 24px;
  border-radius: 6px;
  font-size: 10px;
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
  font-size: 11px;
  font-weight: 700;
  padding: 5px 8px;
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
  padding: 8px 14px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

@media (max-width: 900px) {
  .stage-board {
    padding: 12px max(12px, var(--safe-right)) 16px max(12px, var(--safe-left));
    scroll-snap-type: x mandatory;
  }

  .board-acts-track {
    gap: 12px;
  }

  .board-act-column {
    width: min(84vw, 330px);
    scroll-snap-align: center;
  }
}
</style>