<script setup lang="ts">
import { ref, computed } from "vue";
import { renderMarkdown } from "../utils/markdown";
import { useHaptics } from "../composables/useHaptics";

const { title, category, content, fontSize, hasPrev, hasNext } = defineProps<{
  title: string;
  category: string;
  content: string;
  fontSize: number;
  hasPrev?: boolean;
  hasNext?: boolean;
}>();

const emit = defineEmits<{
  (e: "prev"): void;
  (e: "next"): void;
  (e: "edit"): void;
  (e: "selectEntityByTitle", entityTitle: string): void;
}>();

const { vibrateLight, vibrateSelection } = useHaptics();

type ReaderAtmosphere = "ivory" | "parchment" | "sepia" | "midnight";

const atmosphere = ref<ReaderAtmosphere>("parchment");
const isSpreadMode = ref(true);
const isBookmarkActive = ref(false);
const columnWidth = ref<"narrow" | "standard" | "wide">("standard");

const renderedHtml = computed(() => renderMarkdown(content));

const readingTimeMinutes = computed(() => {
  const words = content.trim() ? content.trim().split(/\s+/).length : 0;
  return Math.max(1, Math.ceil(words / 200));
});

function handleReaderClick(e: MouseEvent) {
  const target = e.target as HTMLElement;
  const wikilink = target.closest(".md-wikilink") as HTMLElement | null;
  if (wikilink && wikilink.dataset.entity) {
    e.preventDefault();
    vibrateSelection();
    emit("selectEntityByTitle", wikilink.dataset.entity);
  }
}

function toggleBookmark() {
  isBookmarkActive.value = !isBookmarkActive.value;
  vibrateLight();
}
</script>

<template>
  <section :class="['stage-reader', `atmosphere-${atmosphere}`, `width-${columnWidth}`, { 'spread-active': isSpreadMode }]">
    <div class="reader-floating-hud">
      <div class="hud-group">
        <button
          :class="['hud-pill-btn', { active: atmosphere === 'ivory' }]"
          title="Слоновая кость"
          @click="atmosphere = 'ivory'"
        >
          Ивори
        </button>
        <button
          :class="['hud-pill-btn', { active: atmosphere === 'parchment' }]"
          title="Имперский пергамент"
          @click="atmosphere = 'parchment'"
        >
          Пергамент
        </button>
        <button
          :class="['hud-pill-btn', { active: atmosphere === 'sepia' }]"
          title="Французская сепия"
          @click="atmosphere = 'sepia'"
        >
          Сепия
        </button>
        <button
          :class="['hud-pill-btn', { active: atmosphere === 'midnight' }]"
          title="Бархатная полночь"
          @click="atmosphere = 'midnight'"
        >
          Полночь
        </button>
      </div>

      <div class="hud-divider"></div>

      <div class="hud-group">
        <button
          :class="['hud-pill-btn', { active: isSpreadMode }]"
          title="Переключить книжный разворот"
          @click="isSpreadMode = !isSpreadMode"
        >
          <span>{{ isSpreadMode ? 'Разворот' : 'Свиток' }}</span>
        </button>
        <button
          class="hud-pill-btn"
          title="Ширина текста"
          @click="columnWidth = columnWidth === 'narrow' ? 'standard' : columnWidth === 'standard' ? 'wide' : 'narrow'"
        >
          {{ columnWidth === 'narrow' ? 'Компакт' : columnWidth === 'standard' ? 'Стандарт' : 'Широкий' }}
        </button>
      </div>

      <div class="hud-divider"></div>

      <button class="hud-edit-cta" @click="emit('edit')">Редактор ✎</button>
    </div>

    <div class="book-tome-chassis" :style="{ fontSize: `${fontSize}px` }">
      <div
        :class="['silk-bookmark-ribbon', { saved: isBookmarkActive }]"
        title="Поставить закладку"
        @click="toggleBookmark"
      >
        <span class="ribbon-shine"></span>
      </div>

      <div class="book-running-header">
        <span class="header-left-col">«Wingnite Story»</span>
        <span class="header-center-col">✦</span>
        <span class="header-right-col">{{ category || 'Черновик' }} • {{ title || 'Без названия' }}</span>
      </div>

      <header class="sheet-title-zone">
        <span class="sheet-badge-canon">{{ category }}</span>
        <h1 class="sheet-main-title">{{ title || 'Новая глава' }}</h1>
        <div class="sheet-ornament">
          <span class="ornament-wing-left"></span>
          <span class="ornament-center">❦</span>
          <span class="ornament-wing-right"></span>
        </div>
      </header>

      <div
        class="sheet-prose markdown-rendered"
        v-html="renderedHtml"
        @click="handleReaderClick"
      ></div>

      <div class="sheet-end-mark">
        <div class="end-symbol-cluster">
          <span>✦</span>
          <span class="large-star">✧</span>
          <span>✦</span>
        </div>
      </div>

      <div class="book-running-footer">
        <span class="footer-meta">~{{ readingTimeMinutes }} мин чтения</span>
        <span class="footer-fleuron">❦</span>
        <span class="footer-status">{{ isBookmarkActive ? 'Закладка сохранена' : 'Конец сцены' }}</span>
      </div>

      <nav class="reader-chapter-nav">
        <button
          class="btn-nav-chapter"
          :disabled="!hasPrev"
          @click="emit('prev')"
        >
          ← Предыдущая глава
        </button>
        <button class="btn-nav-chapter primary" @click="emit('edit')">
          Перейти в студию ✎
        </button>
        <button
          class="btn-nav-chapter"
          :disabled="!hasNext"
          @click="emit('next')"
        >
          Следующая глава →
        </button>
      </nav>
    </div>
  </section>
</template>

<style scoped>
.stage-reader {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 30px 20px calc(var(--safe-bottom) + 80px) 20px;
  position: relative;
  transition: background 0.3s ease;
}

.stage-reader.atmosphere-ivory {
  background: #fbfaf5;
  --book-bg: #ffffff;
  --book-border: #e6e2d3;
  --book-text: #19140f;
  --book-gold: #824d26;
  --book-shadow: 0 20px 60px rgba(0, 0, 0, 0.08);
}

.stage-reader.atmosphere-parchment {
  background: #0d111a;
  --book-bg: #141924;
  --book-border: rgba(245, 158, 11, 0.25);
  --book-text: #f8fafc;
  --book-gold: #f59e0b;
  --book-shadow: 0 24px 70px rgba(0, 0, 0, 0.7);
}

.stage-reader.atmosphere-sepia {
  background: #eadecb;
  --book-bg: #f5ece0;
  --book-border: #d4c2a5;
  --book-text: #24180d;
  --book-gold: #7c441f;
  --book-shadow: 0 20px 60px rgba(70, 50, 30, 0.15);
}

.stage-reader.atmosphere-midnight {
  background: #000000;
  --book-bg: #080a10;
  --book-border: rgba(255, 255, 255, 0.16);
  --book-text: #f1f5f9;
  --book-gold: #38bdf8;
  --book-shadow: 0 24px 80px rgba(0, 0, 0, 0.95);
}

.reader-floating-hud {
  display: flex;
  align-items: center;
  gap: 10px;
  background: var(--bg-surface);
  backdrop-filter: blur(24px);
  border: 1px solid var(--border);
  padding: 6px 14px;
  border-radius: 999px;
  margin-bottom: 24px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
  z-index: 20;
}

.hud-group {
  display: flex;
  gap: 4px;
}

.hud-divider {
  width: 1px;
  height: 18px;
  background: var(--border);
}

.hud-pill-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 700;
  padding: 4px 10px;
  border-radius: 999px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.hud-pill-btn:hover {
  color: var(--text-pure);
}

.hud-pill-btn.active {
  background: var(--accent-indigo);
  color: #fff;
  box-shadow: 0 2px 10px var(--accent-indigo-glow);
}

.hud-edit-cta {
  background: transparent;
  border: 1px solid var(--accent-cyan);
  color: var(--accent-cyan);
  border-radius: 999px;
  padding: 4px 12px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}

.book-tome-chassis {
  width: 100%;
  max-width: 860px;
  background: var(--book-bg);
  color: var(--book-text);
  border: 1px solid var(--book-border);
  box-shadow: var(--book-shadow);
  border-radius: 20px;
  padding: 64px 60px;
  position: relative;
  font-family: var(--font-serif);
  line-height: 1.95;
  user-select: text;
  transition: max-width 0.3s ease;
}

.stage-reader.width-narrow .book-tome-chassis {
  max-width: 680px;
}

.stage-reader.width-wide .book-tome-chassis {
  max-width: 1060px;
}

@media (min-width: 1180px) {
  .stage-reader.spread-active .book-tome-chassis {
    max-width: 1160px;
    column-count: 2;
    column-gap: 60px;
    column-rule: 1px solid var(--book-border);
  }

  .stage-reader.spread-active .sheet-title-zone,
  .stage-reader.spread-active .book-running-header,
  .stage-reader.spread-active .book-running-footer,
  .stage-reader.spread-active .reader-chapter-nav {
    column-span: all;
  }
}

.silk-bookmark-ribbon {
  position: absolute;
  top: -6px;
  right: 50px;
  width: 26px;
  height: 60px;
  background: linear-gradient(135deg, var(--book-gold) 0%, #b45309 100%);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
  clip-path: polygon(0 0, 100% 0, 100% 100%, 50% 82%, 0 100%);
  cursor: pointer;
  transition: transform 0.25s ease, filter 0.25s ease;
}

.silk-bookmark-ribbon:hover {
  transform: translateY(4px);
  filter: brightness(1.15);
}

.silk-bookmark-ribbon.saved {
  height: 80px;
  background: linear-gradient(135deg, #f43f5e 0%, #be123c 100%);
}

.ribbon-shine {
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, rgba(255, 255, 255, 0.35) 0%, transparent 80%);
}

.book-running-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--book-gold);
  border-bottom: 1px solid var(--book-border);
  padding-bottom: 12px;
  margin-bottom: 40px;
  opacity: 0.85;
}

.sheet-title-zone {
  text-align: center;
  margin-bottom: 40px;
}

.sheet-badge-canon {
  text-transform: uppercase;
  font-size: 11px;
  letter-spacing: 0.28em;
  color: var(--book-gold);
  font-family: var(--font-sans);
  font-weight: 800;
}

.sheet-main-title {
  font-family: "Playfair Display", Georgia, serif;
  font-size: 2.8em;
  letter-spacing: -0.02em;
  margin: 12px 0 18px 0;
  line-height: 1.22;
  font-weight: 700;
  color: var(--book-text);
}

.sheet-ornament {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
}

.ornament-wing-left,
.ornament-wing-right {
  height: 1px;
  width: 70px;
  background: linear-gradient(90deg, transparent, var(--book-gold), transparent);
}

.ornament-center {
  font-size: 24px;
  color: var(--book-gold);
  opacity: 0.9;
}

.sheet-prose :deep(p:first-of-type::first-letter) {
  font-family: "Playfair Display", Georgia, serif;
  font-size: 3.8em;
  float: left;
  line-height: 0.8;
  padding-right: 0.14em;
  padding-top: 0.08em;
  color: var(--book-gold);
  font-weight: 700;
}

.sheet-end-mark {
  text-align: center;
  color: var(--book-gold);
  margin: 50px 0 20px 0;
}

.end-symbol-cluster {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  font-size: 12px;
  opacity: 0.75;
}

.large-star {
  font-size: 18px;
  opacity: 1;
}

.book-running-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--book-gold);
  border-top: 1px solid var(--book-border);
  padding-top: 14px;
  margin-top: 30px;
  opacity: 0.85;
}

.footer-fleuron {
  font-size: 16px;
}

.reader-chapter-nav {
  display: flex;
  justify-content: space-between;
  margin-top: 36px;
  gap: 12px;
}

.btn-nav-chapter {
  background: var(--bg-card);
  border: 1px solid var(--book-border);
  color: var(--book-text);
  padding: 10px 18px;
  border-radius: 10px;
  font-family: var(--font-sans);
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-nav-chapter.primary {
  background: var(--accent-indigo);
  color: #fff;
  border: none;
}

.btn-nav-chapter:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

@media (max-width: 768px) {
  .stage-reader {
    padding: 16px 10px calc(var(--safe-bottom) + 80px) 10px;
  }

  .book-tome-chassis {
    padding: 40px 20px;
    border-radius: 14px;
  }

  .sheet-main-title {
    font-size: 2em;
  }

  .book-running-header {
    display: none;
  }
}
</style>