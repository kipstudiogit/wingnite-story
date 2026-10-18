<script setup lang="ts">
import { computed } from "vue";
import { marked } from "marked";

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
}>();

const renderedMarkdown = computed(() => marked.parse(content) as string);
</script>

<template>
  <section class="stage-reader">
    <div class="book-paper-sheet" :style="{ fontSize: `${fontSize}px` }">
      <div class="ribbon-bookmark">
        <span class="ribbon-shine"></span>
      </div>

      <header class="sheet-title-zone">
        <div class="sheet-top-tools">
          <span class="sheet-badge">{{ category }}</span>
          <button class="btn-jump-edit" @click="emit('edit')">Редактировать сцену ✎</button>
        </div>
        <h1 class="sheet-main-title">{{ title }}</h1>
        <div class="sheet-ornament">
          <span class="ornament-wing-left"></span>
          <span class="ornament-center">❦</span>
          <span class="ornament-wing-right"></span>
        </div>
      </header>

      <div class="sheet-prose markdown-rendered" v-html="renderedMarkdown"></div>

      <footer class="sheet-end-mark">
        <div class="end-symbol-cluster">
          <span>✦</span>
          <span class="large-star">✧</span>
          <span>✦</span>
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
      </footer>
    </div>
  </section>
</template>

<style scoped>
.stage-reader {
  flex: 1;
  overflow-y: auto;
  display: flex;
  justify-content: center;
  padding: 40px 20px calc(var(--safe-bottom) + 80px) 20px;
}

.book-paper-sheet {
  max-width: 820px;
  width: 100%;
  font-family: var(--font-serif);
  line-height: 2;
  user-select: text;
  background: var(--card);
  backdrop-filter: blur(36px);
  border: 1px solid var(--border);
  box-shadow: var(--shadow-ambient), var(--specular);
  border-radius: 20px;
  padding: 60px 50px;
  position: relative;
}

.ribbon-bookmark {
  position: absolute;
  top: -8px;
  right: 50px;
  width: 24px;
  height: 54px;
  background: var(--gold-gradient);
  box-shadow: 0 8px 20px var(--gold-glow);
  clip-path: polygon(0 0, 100% 0, 100% 100%, 50% 82%, 0 100%);
}

.ribbon-shine {
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, rgba(255, 255, 255, 0.3) 0%, transparent 80%);
}

.sheet-title-zone {
  text-align: center;
  margin-bottom: 40px;
}

.sheet-top-tools {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.sheet-badge {
  text-transform: uppercase;
  font-size: 11px;
  letter-spacing: 0.25em;
  color: var(--gold);
  font-family: var(--font-sans);
  font-weight: 800;
}

.btn-jump-edit {
  background: transparent;
  border: 1px solid var(--border);
  color: var(--accent-cyan);
  border-radius: 6px;
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}

.sheet-main-title {
  font-family: "Playfair Display", Georgia, serif;
  font-size: 2.6em;
  letter-spacing: -0.02em;
  margin: 12px 0 16px 0;
  line-height: 1.25;
  font-weight: 700;
  color: var(--text-main);
}

.sheet-ornament {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 14px;
}

.ornament-wing-left,
.ornament-wing-right {
  height: 1px;
  width: 60px;
  background: linear-gradient(90deg, transparent, var(--gold), transparent);
}

.ornament-center {
  font-size: 22px;
  color: var(--gold);
  opacity: 0.9;
}

.sheet-prose :deep(p:first-of-type::first-letter) {
  font-family: "Playfair Display", Georgia, serif;
  font-size: 3.8em;
  float: left;
  line-height: 0.8;
  padding-right: 0.14em;
  padding-top: 0.08em;
  color: var(--gold);
  font-weight: 700;
}

.sheet-end-mark {
  text-align: center;
  color: var(--gold);
  margin-top: 60px;
}

.end-symbol-cluster {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  font-size: 12px;
  opacity: 0.7;
}

.large-star {
  font-size: 16px;
  opacity: 1;
}

.reader-chapter-nav {
  display: flex;
  justify-content: space-between;
  margin-top: 36px;
  gap: 12px;
}

.btn-nav-chapter {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--border);
  color: var(--text-main);
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

.btn-nav-chapter:not(:disabled):hover {
  background: var(--bg-card-hover);
  border-color: var(--accent-indigo);
  color: #fff;
}

@media (max-width: 768px) {
  .stage-reader {
    padding: 16px 10px calc(var(--safe-bottom) + 80px) 10px;
  }

  .book-paper-sheet {
    padding: 36px 18px;
    border-radius: 14px;
  }

  .sheet-main-title {
    font-size: 2em;
  }
}
</style>