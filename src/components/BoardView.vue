<script setup lang="ts">
import { ref, computed } from "vue";
import type { StoryItem } from "../types";

const { actsList, stories } = defineProps<{
  actsList: string[];
  stories: StoryItem[];
}>();

const emit = defineEmits<{
  (e: "select", story: StoryItem): void;
}>();

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

const storiesByAct = computed(() => {
  const map = new Map<string, ProcessedStoryItem[]>();
  for (const act of actsList) {
    map.set(act, []);
  }
  for (const s of stories) {
    const list = map.get(s.category);
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
  return map;
});

const mousePositions = ref<Map<string, { x: number; y: number }>>(new Map());

function handleCardMouseMove(id: string, e: MouseEvent) {
  const el = e.currentTarget as HTMLElement;
  const rect = el.getBoundingClientRect();
  mousePositions.value.set(id, {
    x: e.clientX - rect.left,
    y: e.clientY - rect.top,
  });
}

function handleCardMouseLeave(id: string) {
  mousePositions.value.delete(id);
}
</script>

<template>
  <section class="stage-board">
    <div class="board-acts-track">
      <div v-for="act in actsList" :key="act" class="board-act-column">
        <header class="act-col-header">
          <div class="act-title-wrapper">
            <span class="act-status-flame"></span>
            <h3 class="act-name">{{ act }}</h3>
          </div>
          <span class="act-badge-counter">{{ storiesByAct.get(act)?.length ?? 0 }}</span>
        </header>

        <div class="act-cards-scroller">
          <div
            v-for="item in (storiesByAct.get(act) ?? [])"
            :key="item.id"
            class="prismatic-card"
            :style="{
              '--card-glow-x': `${mousePositions.get(item.id)?.x ?? 150}px`,
              '--card-glow-y': `${mousePositions.get(item.id)?.y ?? 80}px`
            }"
            @mousemove="handleCardMouseMove(item.id, $event)"
            @mouseleave="handleCardMouseLeave(item.id)"
            @click="emit('select', item)"
          >
            <div class="card-radial-spotlight"></div>

            <div class="card-top-meta">
              <span class="card-act-tag">{{ item.category }}</span>
              <span class="card-words-tag">{{ item.wordCount }} сл.</span>
            </div>

            <h4 class="card-heading">{{ item.title }}</h4>
            <p class="card-snippet">{{ item.excerpt }}...</p>

            <div class="card-bottom-telemetry">
              <div class="card-meter-track">
                <div class="card-meter-fill" :style="{ width: `${item.progressPercent}%` }"></div>
              </div>
              <span class="card-meter-label">{{ item.progressPercent }}% нормы</span>
            </div>
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
  background: var(--panel);
  backdrop-filter: blur(40px) saturate(220%);
  border: 1px solid var(--border);
  box-shadow: var(--shadow-ambient), var(--specular);
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
  background: var(--accent);
  box-shadow: 0 0 14px var(--accent-glow), 0 0 4px #fff;
}

.act-name {
  font-size: 14px;
  font-weight: 800;
  letter-spacing: -0.015em;
  color: var(--text-main);
}

.act-badge-counter {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid var(--border);
  padding: 3px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  font-family: var(--font-mono);
  color: var(--text-muted);
}

.act-cards-scroller {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  overflow-y: auto;
}

.prismatic-card {
  position: relative;
  background: var(--card);
  backdrop-filter: blur(20px);
  border: 1px solid var(--border);
  box-shadow: var(--shadow-card), var(--specular-prismatic);
  border-radius: 14px;
  padding: 18px 20px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 10px;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.card-radial-spotlight {
  position: absolute;
  inset: 0;
  background: radial-gradient(280px circle at var(--card-glow-x, 150px) var(--card-glow-y, 80px), rgba(255, 255, 255, 0.08), transparent 70%);
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.25s ease;
}

.prismatic-card:hover .card-radial-spotlight {
  opacity: 1;
}

.prismatic-card:hover {
  transform: translateY(-4px) scale(1.015);
  background: var(--card-hover);
  border-color: var(--border-focus);
  box-shadow: var(--shadow-card-hover);
}

.card-top-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
  font-weight: 700;
}

.card-act-tag {
  color: var(--accent-light);
  text-transform: uppercase;
  letter-spacing: 0.07em;
  font-size: 10px;
}

.card-words-tag {
  color: var(--text-dark);
  font-family: var(--font-mono);
}

.card-heading {
  font-size: 15px;
  font-weight: 700;
  letter-spacing: -0.01em;
  color: var(--text-main);
  line-height: 1.35;
}

.card-snippet {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.55;
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
  background: rgba(255, 255, 255, 0.08);
  border-radius: 999px;
  overflow: hidden;
}

.card-meter-fill {
  height: 100%;
  background: var(--accent-gradient);
  box-shadow: 0 0 10px var(--accent-glow);
  border-radius: 999px;
  transition: width 0.4s ease;
}

.card-meter-label {
  font-size: 10px;
  font-weight: 600;
  font-family: var(--font-mono);
  color: var(--text-dark);
}
</style>