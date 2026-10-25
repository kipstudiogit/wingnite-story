<script setup lang="ts">
import MobileDrawerSheet from "./MobileDrawerSheet.vue";
import type { AppTab, ProjectItem, ThemeMode } from "../../types";

const {
  isOpen,
  projects,
  activeProjectId,
  currentTheme,
  isCheckingUpdate,
} = defineProps<{
  isOpen: boolean;
  projects: ProjectItem[];
  activeProjectId: string;
  currentTheme: ThemeMode;
  isCheckingUpdate: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "switchProject", id: string): void;
  (e: "newProject"): void;
  (e: "exportWstory"): void;
  (e: "importWstory"): void;
  (e: "checkUpdate"): void;
  (e: "selectTab", tab: AppTab): void;
  (e: "getTwist"): void;
  (e: "setTheme", theme: ThemeMode): void;
  (e: "exportHtml"): void;
  (e: "exportMd"): void;
  (e: "exportTxt"): void;
  (e: "importTxt"): void;
}>();
</script>

<template>
  <MobileDrawerSheet
    :is-open="isOpen"
    title="Панель инструментов"
    @close="emit('close')"
  >
    <div class="mobile-menu-grid">
      <div class="menu-action-block">
        <h4>Произведение</h4>
        <div class="menu-btn-row">
          <button
            v-for="p in projects"
            :key="p.id"
            :class="['menu-chip-btn', { active: p.id === activeProjectId }]"
            @click="emit('switchProject', p.id)"
          >
            {{ p.title }}
          </button>
          <button class="menu-chip-btn accent" @click="emit('newProject')">
            + Новая книга
          </button>
        </div>
      </div>

      <div class="menu-action-block">
        <h4>Импорт и Экспорт файлов</h4>
        <div class="menu-btn-row">
          <button class="menu-chip-btn" @click="emit('importTxt')">Импорт .TXT</button>
          <button class="menu-chip-btn" @click="emit('exportTxt')">Экспорт .TXT</button>
          <button class="menu-chip-btn" @click="emit('exportWstory')">Экспорт .WSTORY</button>
          <button class="menu-chip-btn" @click="emit('importWstory')">Импорт .WSTORY</button>
        </div>
      </div>

      <div class="menu-action-block">
        <h4>Навигация модулей</h4>
        <div class="menu-btn-row">
          <button class="menu-chip-btn" @click="emit('selectTab', 'timeline')">Таймлайн</button>
          <button class="menu-chip-btn" @click="emit('selectTab', 'graph')">Космос (Граф)</button>
          <button class="menu-chip-btn accent-purple" @click="emit('selectTab', 'settings')">⚙ Настройки</button>
          <button class="menu-chip-btn" @click="emit('getTwist')">Генератор твиста</button>
        </div>
      </div>

      <div class="menu-action-block">
        <h4>Темы оформления</h4>
        <div class="menu-btn-row">
          <button :class="['theme-chip-btn', { active: currentTheme === 'obsidian' }]" @click="emit('setTheme', 'obsidian')">Dark</button>
          <button :class="['theme-chip-btn', { active: currentTheme === 'celestial' }]" @click="emit('setTheme', 'celestial')">Amber</button>
          <button :class="['theme-chip-btn', { active: currentTheme === 'sepia' }]" @click="emit('setTheme', 'sepia')">Sepia</button>
          <button :class="['theme-chip-btn', { active: currentTheme === 'paper' }]" @click="emit('setTheme', 'paper')">Paper</button>
          <button :class="['theme-chip-btn', { active: currentTheme === 'oled' }]" @click="emit('setTheme', 'oled')">OLED</button>
        </div>
      </div>

      <div class="menu-action-block">
        <h4>Обновление и сборка</h4>
        <div class="menu-btn-row vertical">
          <button class="menu-export-btn" @click="emit('checkUpdate')">
            {{ isCheckingUpdate ? 'Проверка...' : 'Проверить обновление (v1.3.0)' }}
          </button>
          <button class="menu-export-btn" @click="emit('exportHtml')">Экспорт издания (HTML)</button>
          <button class="menu-export-btn" @click="emit('exportMd')">Экспорт рукописи (Markdown)</button>
        </div>
      </div>
    </div>
  </MobileDrawerSheet>
</template>

<style scoped>
.mobile-menu-grid {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.menu-action-block h4 {
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--text-muted);
  margin-bottom: 8px;
}

.menu-btn-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.menu-btn-row.vertical {
  flex-direction: column;
}

.menu-chip-btn {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-main);
  border-radius: 10px;
  padding: 10px 14px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.menu-chip-btn.active {
  background: var(--accent-indigo);
  color: #fff;
  border-color: var(--accent-indigo);
}

.menu-chip-btn.accent {
  background: rgba(6, 182, 212, 0.15);
  border-color: var(--accent-cyan);
  color: var(--accent-cyan);
}

.menu-chip-btn.accent-purple {
  background: rgba(168, 85, 247, 0.15);
  border-color: var(--accent-purple);
  color: var(--accent-purple);
}

.theme-chip-btn {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-muted);
  border-radius: 8px;
  padding: 8px 12px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}

.theme-chip-btn.active {
  background: var(--accent-indigo);
  color: #fff;
  border-color: var(--accent-indigo);
}

.menu-export-btn {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-pure);
  border-radius: 12px;
  padding: 12px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  text-align: center;
}
</style>