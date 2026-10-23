<script setup lang="ts">
import { ref, computed } from "vue";
import { renderMarkdown } from "../utils/markdown";
import type { StoryItem } from "../types";

const { stories } = defineProps<{
  stories: StoryItem[];
}>();

const emit = defineEmits<{
  (e: "select", story: StoryItem): void;
  (e: "createEntity", type: "artifact" | "pantheon" | "sanctuary" | "character"): void;
}>();

const filterCategory = ref<string>("all");
const searchQuery = ref<string>("");
const mouseCoords = ref<Map<string, { x: number; y: number }>>(new Map());

interface CodexDossierItem extends StoryItem {
  icon: string;
  isDivine: boolean;
  resonance: number;
  previewHtml: string;
}

function calculateResonance(id: string, text: string): number {
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = (hash << 5) - hash + id.charCodeAt(i);
    hash |= 0;
  }
  return 65 + (Math.abs(hash + text.length) % 35);
}

function getEntityIcon(category: string) {
  if (category.includes("Артефакт") || category.includes("Реликвия")) return "🔮";
  if (category.includes("Демиург") || category.includes("Сущность")) return "👑";
  if (category.includes("Святилище") || category.includes("Локация")) return "🏛️";
  return "👤";
}

const categorizedCodex = computed<CodexDossierItem[]>(() => {
  const result: CodexDossierItem[] = [];
  const q = searchQuery.value.toLowerCase().trim();

  for (const item of stories) {
    const isCodex =
      item.category.includes("Персонаж") ||
      item.category.includes("Артефакт") ||
      item.category.includes("Демиург") ||
      item.category.includes("Святилище") ||
      item.category.includes("Локация");

    if (!isCodex) continue;

    if (filterCategory.value !== "all") {
      if (!item.category.toLowerCase().includes(filterCategory.value)) continue;
    }

    if (q) {
      const match =
        item.title.toLowerCase().includes(q) ||
        item.content.toLowerCase().includes(q) ||
        item.category.toLowerCase().includes(q);
      if (!match) continue;
    }

    result.push({
      ...item,
      icon: getEntityIcon(item.category),
      isDivine: item.category.includes("Артефакт") || item.category.includes("Демиург"),
      resonance: calculateResonance(item.id, item.content),
      previewHtml: renderMarkdown(item.content.slice(0, 180)),
    });
  }
  return result;
});

function handleMouseMove(id: string, e: MouseEvent) {
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  mouseCoords.value.set(id, {
    x: e.clientX - rect.left,
    y: e.clientY - rect.top,
  });
}

function handleMouseLeave(id: string) {
  mouseCoords.value.delete(id);
}
</script>

<template>
  <section class="quantum-codex-stage">
    <header class="codex-quantum-header">
      <div class="header-branding">
        <h2>Реестр Сущностей и Реликвий</h2>
        <p>Архивная база персонажей, артефактов и лора текущего произведения</p>
      </div>

      <div class="codex-action-cluster">
        <button class="btn-summon character" @click="emit('createEntity', 'character')">
          <span class="btn-flare"></span>
          + Персонаж
        </button>
        <button class="btn-summon relic" @click="emit('createEntity', 'artifact')">
          <span class="btn-flare"></span>
          + Артефакт
        </button>
        <button class="btn-summon demiurge" @click="emit('createEntity', 'pantheon')">
          <span class="btn-flare"></span>
          + Демиург
        </button>
        <button class="btn-summon sanctuary" @click="emit('createEntity', 'sanctuary')">
          <span class="btn-flare"></span>
          + Святилище
        </button>
      </div>
    </header>

    <div class="codex-controls-bar">
      <div class="filter-pills-row">
        <button
          :class="['filter-pill', { active: filterCategory === 'all' }]"
          @click="filterCategory = 'all'"
        >
          Все ({{ categorizedCodex.length }})
        </button>
        <button
          :class="['filter-pill', { active: filterCategory === 'персонаж' }]"
          @click="filterCategory = 'персонаж'"
        >
          Персонажи
        </button>
        <button
          :class="['filter-pill', { active: filterCategory === 'артефакт' }]"
          @click="filterCategory = 'артефакт'"
        >
          Артефакты
        </button>
        <button
          :class="['filter-pill', { active: filterCategory === 'демиург' }]"
          @click="filterCategory = 'демиург'"
        >
          Демиурги
        </button>
        <button
          :class="['filter-pill', { active: filterCategory === 'святилище' }]"
          @click="filterCategory = 'святилище'"
        >
          Святилища
        </button>
      </div>

      <div class="codex-search-box">
        <span class="search-lens">⌕</span>
        <input
          v-model="searchQuery"
          class="codex-search-input"
          placeholder="Фильтр реестра..."
        />
      </div>
    </div>

    <div class="codex-hologram-grid">
      <div
        v-for="item in categorizedCodex"
        :key="item.id"
        :class="['holo-dossier-card', { 'divine-aurora': item.isDivine }]"
        :style="{
          '--light-x': `${mouseCoords.get(item.id)?.x ?? 150}px`,
          '--light-y': `${mouseCoords.get(item.id)?.y ?? 100}px`
        }"
        @mousemove="handleMouseMove(item.id, $event)"
        @mouseleave="handleMouseLeave(item.id)"
        @click="emit('select', item)"
      >
        <div class="card-gyro-refraction"></div>

        <div class="dossier-header-strip">
          <div class="avatar-quantum-frame">
            <span class="avatar-sigil">{{ item.icon }}</span>
          </div>

          <div class="dossier-heading-text">
            <span class="dossier-archetype">{{ item.category }}</span>
            <h3 class="dossier-entity-name">{{ item.title }}</h3>
          </div>
        </div>

        <div class="dossier-synopsis markdown-rendered" v-html="item.previewHtml"></div>

        <div class="dossier-telemetry-dock">
          <div class="resonance-gauge">
            <span class="resonance-label">Резонанс</span>
            <div class="resonance-track">
              <div class="resonance-bar" :style="{ width: `${item.resonance}%` }"></div>
            </div>
          </div>
          <span class="resonance-digit">{{ item.resonance }}%</span>
        </div>
      </div>

      <div v-if="categorizedCodex.length === 0" class="codex-empty-state">
        <p>Сущностей данной категории пока не зафиксировано</p>
        <div class="empty-btn-cluster">
          <button class="btn-summon character" @click="emit('createEntity', 'character')">+ Создать персонажа</button>
          <button class="btn-summon relic" @click="emit('createEntity', 'artifact')">+ Создать артефакт</button>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.quantum-codex-stage {
  flex: 1;
  overflow-y: auto;
  padding: 36px 40px calc(var(--safe-bottom) + 80px) 40px;
  max-width: 1360px;
  margin: 0 auto;
  width: 100%;
}

.codex-quantum-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 28px;
  gap: 20px;
  flex-wrap: wrap;
}

.header-branding h2 {
  font-size: 28px;
  font-weight: 900;
  letter-spacing: -0.03em;
  background: linear-gradient(135deg, #ffffff 40%, var(--accent-cyan) 100%);
  -webkit-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
}

.header-branding p {
  color: var(--text-muted);
  font-size: 13px;
  margin-top: 4px;
}

.codex-action-cluster {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.btn-summon {
  position: relative;
  border: none;
  border-radius: 12px;
  padding: 10px 18px;
  font-size: 12px;
  font-weight: 800;
  cursor: pointer;
  overflow: hidden;
  color: #fff;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}

.btn-summon.character {
  background: linear-gradient(135deg, #3b82f6 0%, #1d4ed8 100%);
  box-shadow: 0 4px 20px rgba(59, 130, 246, 0.4);
}

.btn-summon.relic {
  background: linear-gradient(135deg, #a855f7 0%, #7e22ce 100%);
  box-shadow: 0 4px 20px var(--accent-purple-glow);
}

.btn-summon.demiurge {
  background: linear-gradient(135deg, #f59e0b 0%, #b45309 100%);
  box-shadow: 0 4px 20px var(--accent-gold-glow);
}

.btn-summon.sanctuary {
  background: linear-gradient(135deg, #06b6d4 0%, #0891b2 100%);
  box-shadow: 0 4px 20px var(--accent-cyan-glow);
}

.btn-flare {
  position: absolute;
  top: 0;
  left: -100%;
  width: 50%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.45), transparent);
  transform: skewX(-25deg);
  transition: left 0.6s ease;
}

.btn-summon:hover .btn-flare {
  left: 200%;
}

.btn-summon:hover {
  transform: translateY(-2px);
  filter: brightness(1.1);
}

.codex-controls-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  margin-bottom: 30px;
  flex-wrap: wrap;
}

.filter-pills-row {
  display: flex;
  gap: 6px;
  overflow-x: auto;
  padding-bottom: 2px;
}

.filter-pill {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--border);
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 700;
  padding: 6px 14px;
  border-radius: 999px;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s ease;
}

.filter-pill:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--text-main);
}

.filter-pill.active {
  background: var(--accent-indigo);
  border-color: var(--accent-indigo);
  color: #fff;
  box-shadow: 0 0 16px var(--accent-indigo-glow);
}

.codex-search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 6px 14px;
  min-width: 260px;
}

.search-lens {
  color: var(--text-dim);
  font-size: 16px;
}

.codex-search-input {
  background: transparent;
  border: none;
  font-size: 13px;
  color: var(--text-main);
  outline: none;
  width: 100%;
}

.codex-hologram-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
  gap: 24px;
}

.holo-dossier-card {
  position: relative;
  background: var(--bg-card);
  backdrop-filter: blur(28px);
  border: 1px solid var(--border);
  box-shadow: var(--shadow-quantum), var(--specular-high);
  border-radius: 18px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  cursor: pointer;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.card-gyro-refraction {
  position: absolute;
  inset: 0;
  background: radial-gradient(380px circle at var(--light-x, 150px) var(--light-y, 100px), rgba(255, 255, 255, 0.09), transparent 65%);
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.3s ease;
}

.holo-dossier-card:hover .card-gyro-refraction {
  opacity: 1;
}

.holo-dossier-card:hover {
  transform: translateY(-4px) scale(1.015);
  background: var(--bg-card-hover);
  border-color: var(--border-accent);
  box-shadow: 0 24px 60px -10px rgba(0, 0, 0, 0.9), 0 0 24px var(--accent-indigo-glow);
}

.holo-dossier-card.divine-aurora {
  border-color: rgba(245, 158, 11, 0.3);
}

.holo-dossier-card.divine-aurora:hover {
  border-color: var(--accent-gold);
  box-shadow: 0 24px 60px -10px rgba(0, 0, 0, 0.9), var(--shadow-neon-gold);
}

.dossier-header-strip {
  display: flex;
  align-items: center;
  gap: 16px;
}

.avatar-quantum-frame {
  width: 58px;
  height: 58px;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border-strong);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 30px;
  box-shadow: var(--specular-prismatic);
  flex-shrink: 0;
}

.dossier-heading-text {
  flex: 1;
  overflow: hidden;
}

.dossier-archetype {
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--accent-cyan);
}

.divine-aurora .dossier-archetype {
  color: var(--accent-gold);
}

.dossier-entity-name {
  font-size: 18px;
  font-weight: 800;
  letter-spacing: -0.015em;
  color: var(--text-pure);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}

.dossier-synopsis {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.6;
  max-height: 80px;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  line-clamp: 3;
  -webkit-box-orient: vertical;
}

.dossier-telemetry-dock {
  display: flex;
  align-items: center;
  gap: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--border-subtle);
}

.resonance-gauge {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.resonance-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--text-dim);
}

.resonance-track {
  height: 5px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 999px;
  overflow: hidden;
}

.resonance-bar {
  height: 100%;
  background: linear-gradient(90deg, var(--accent-indigo) 0%, var(--accent-cyan) 100%);
  border-radius: 999px;
}

.divine-aurora .resonance-bar {
  background: linear-gradient(90deg, var(--accent-gold) 0%, #fef08a 100%);
}

.resonance-digit {
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 700;
  color: var(--text-pure);
}

.codex-empty-state {
  grid-column: 1 / -1;
  text-align: center;
  padding: 60px 20px;
  color: var(--text-dim);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.empty-btn-cluster {
  display: flex;
  gap: 10px;
}

@media (max-width: 768px) {
  .quantum-codex-stage {
    padding: 16px 14px calc(var(--safe-bottom) + 84px) 14px;
  }

  .codex-hologram-grid {
    grid-template-columns: 1fr;
  }
}
</style>