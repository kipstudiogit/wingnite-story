<script setup lang="ts">
import type { AppTab, StudioViewMode, ThemeMode } from "../../types";

const studioMode = defineModel<StudioViewMode>("studioMode", { required: true });
const currentTheme = defineModel<ThemeMode>("currentTheme", { required: true });
const fontSize = defineModel<number>("fontSize", { required: true });

const { currentTab } = defineProps<{
  currentTab: AppTab;
}>();

const emit = defineEmits<{
  (e: "exportWstory"): void;
  (e: "exportHtml"): void;
  (e: "exportMd"): void;
  (e: "exportTxt"): void;
  (e: "getTwist"): void;
}>();
</script>

<template>
  <div class="workbench-subtoolbar">
    <div class="subtoolbar-scroller">
      <div class="subtool-left">
        <div class="zoom-stepper">
          <button class="btn-stepper" @click="fontSize = Math.max(13, fontSize - 1)">-</button>
          <span class="stepper-label">{{ fontSize }}px</span>
          <button class="btn-stepper" @click="fontSize = Math.min(28, fontSize + 1)">+</button>
        </div>

        <div class="theme-picker-segmented">
          <button :class="['theme-dot-btn', { active: currentTheme === 'obsidian' }]" @click="currentTheme = 'obsidian'">Dark</button>
          <button :class="['theme-dot-btn', { active: currentTheme === 'celestial' }]" @click="currentTheme = 'celestial'">Amber</button>
          <button :class="['theme-dot-btn', { active: currentTheme === 'sepia' }]" @click="currentTheme = 'sepia'">Sepia</button>
          <button :class="['theme-dot-btn', { active: currentTheme === 'paper' }]" @click="currentTheme = 'paper'">Paper</button>
          <button :class="['theme-dot-btn', { active: currentTheme === 'oled' }]" @click="currentTheme = 'oled'">OLED</button>
        </div>
      </div>

      <div v-if="currentTab === 'studio'" class="layout-segmented">
        <button :class="['btn-layout', { active: studioMode === 'split' }]" @click="studioMode = 'split'">Сплит</button>
        <button :class="['btn-layout', { active: studioMode === 'edit' }]" @click="studioMode = 'edit'">Редактор</button>
        <button :class="['btn-layout', { active: studioMode === 'preview' }]" @click="studioMode = 'preview'">Превью</button>
        <button :class="['btn-layout', { active: studioMode === 'zen' }]" @click="studioMode = 'zen'">Фокус</button>
      </div>

      <div class="subtool-right-actions">
        <button class="btn-quick-export" title="Экспорт сцены в TXT" @click="emit('exportTxt')">.TXT</button>
        <button class="btn-quick-export" title="Экспорт рукописи в Markdown" @click="emit('exportMd')">.MD</button>
        <button class="btn-quick-export" title="Экспорт издания в HTML" @click="emit('exportHtml')">.HTML</button>
        <button class="btn-quick-export" title="Экспорт контейнера .wstory" @click="emit('exportWstory')">.WSTORY</button>
        <button class="btn-generator-spark" @click="emit('getTwist')">Твист</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.workbench-subtoolbar {
  display: flex;
  align-items: center;
  background: var(--bg-surface-solid);
  border-bottom: 1px solid var(--border);
  height: 38px;
  flex-shrink: 0;
}

.subtoolbar-scroller {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 0 14px;
  gap: 10px;
}

.subtool-left {
  display: flex;
  align-items: center;
  gap: 8px;
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
  width: 26px;
  height: 26px;
  color: var(--text-muted);
  cursor: pointer;
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
  background: var(--bg-card);
  padding: 2px;
  border-radius: 8px;
  border: 1px solid var(--border);
}

.theme-dot-btn {
  background: transparent;
  border: none;
  font-size: 10px;
  font-weight: 700;
  padding: 3px 6px;
  border-radius: 6px;
  color: var(--text-muted);
  cursor: pointer;
}

.theme-dot-btn.active {
  background: var(--accent-indigo);
  color: #fff;
}

.layout-segmented {
  display: flex;
  gap: 2px;
  background: var(--bg-card);
  padding: 2px;
  border-radius: 8px;
  border: 1px solid var(--border);
}

.btn-layout {
  background: transparent;
  border: none;
  font-size: 11px;
  padding: 3px 8px;
  border-radius: 6px;
  color: var(--text-muted);
  cursor: pointer;
}

.btn-layout.active {
  background: var(--accent-indigo);
  color: #fff;
}

.subtool-right-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.btn-quick-export {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-main);
  border-radius: 6px;
  padding: 3px 8px;
  font-size: 10px;
  font-weight: 800;
  font-family: var(--font-mono);
  cursor: pointer;
}

.btn-quick-export:hover {
  border-color: var(--border-accent);
}

.btn-generator-spark {
  background: transparent;
  border: 1px solid var(--accent-indigo);
  color: var(--accent-indigo);
  border-radius: 8px;
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}
</style>