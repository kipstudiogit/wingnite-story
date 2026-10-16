<script setup lang="ts">
import { computed } from "vue";
import { marked } from "marked";

const props = defineProps<{
  title: string;
  category: string;
  content: string;
  fontSize: number;
}>();

const renderedMarkdown = computed(() => marked.parse(props.content) as string);
</script>

<template>
  <section class="stage-reader">
    <div class="book-paper-sheet" :style="{ fontSize: `${fontSize}px` }">
      <div class="ribbon-bookmark">
        <span class="ribbon-shine"></span>
      </div>

      <header class="sheet-title-zone">
        <span class="sheet-badge">{{ category }}</span>
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

.sheet-badge {
  text-transform: uppercase;
  font-size: 11px;
  letter-spacing: 0.25em;
  color: var(--gold);
  font-family: var(--font-sans);
  font-weight: 800;
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