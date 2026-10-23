<script setup lang="ts">
import { ref } from "vue";
import type { AppTab, ProjectItem } from "../types";
import { ambientEngine } from "../ambient";

const currentTab = defineModel<AppTab>("currentTab", { required: true });

const {
  sprintActive,
  sprintFormattedTime,
  sprintDelta,
  isSyncing = false,
  projects = [],
  activeProjectId = "default_project",
  updateAvailable = false,
} = defineProps<{
  sprintActive: boolean;
  sprintFormattedTime: string;
  sprintDelta: number;
  isSyncing?: boolean;
  projects?: ProjectItem[];
  activeProjectId?: string;
  updateAvailable?: boolean;
}>();

const emit = defineEmits<{
  (e: "toggleToc"): void;
  (e: "openPalette"): void;
  (e: "toggleSprint"): void;
  (e: "save"): void;
  (e: "selectProject", id: string): void;
  (e: "createProject"): void;
  (e: "deleteProject", id: string): void;
  (e: "exportWstory"): void;
  (e: "importWstory"): void;
  (e: "checkUpdate"): void;
}>();

const ambientRain = ref(false);
const ambientFire = ref(false);
const ambientWind = ref(false);
const projectDropdownOpen = ref(false);

function toggleRain() {
  ambientRain.value = !ambientRain.value;
  ambientEngine.toggleRain(ambientRain.value);
}

function toggleFire() {
  ambientFire.value = !ambientFire.value;
  ambientEngine.toggleFire(ambientFire.value);
}

function toggleWind() {
  ambientWind.value = !ambientWind.value;
  ambientEngine.toggleWind(ambientWind.value);
}

const currentProjectTitle = () => {
  const match = projects.find((p) => p.id === activeProjectId);
  return match ? match.title : "Произведение";
};
</script>

<template>
  <header class="master-quantum-header">
    <div class="header-quad-left">
      <button class="tactile-menu-trigger" title="Открыть оглавление" @click="emit('toggleToc')">
        <svg class="ui-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="3" y1="6" x2="21" y2="6"/>
          <line x1="3" y1="12" x2="15" y2="12"/>
          <line x1="3" y1="18" x2="18" y2="18"/>
        </svg>
        <span class="trigger-text">Оглавление</span>
      </button>

      <div class="project-selector-wrapper">
        <button class="btn-project-selector" @click="projectDropdownOpen = !projectDropdownOpen">
          <svg class="ui-icon" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/>
            <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>
          </svg>
          <span class="project-current-label">{{ currentProjectTitle() }}</span>
          <span class="dropdown-arrow">▾</span>
        </button>

        <div v-if="projectDropdownOpen" class="project-dropdown-menu" @click.stop>
          <div class="dropdown-header">Произведения в базе</div>
          <div class="project-items-scroller">
            <div
              v-for="p in projects"
              :key="p.id"
              :class="['project-choice-row', { active: p.id === activeProjectId }]"
              @click="emit('selectProject', p.id); projectDropdownOpen = false"
            >
              <div class="choice-main-col">
                <div class="choice-title">{{ p.title }}</div>
                <div class="choice-desc">{{ p.description }}</div>
              </div>
              <button
                v-if="projects.length > 1"
                class="btn-delete-proj"
                title="Удалить произведение"
                @click.stop="emit('deleteProject', p.id); projectDropdownOpen = false"
              >
                ✕
              </button>
            </div>
          </div>

          <div class="dropdown-divider"></div>

          <button class="btn-menu-action" @click="emit('createProject'); projectDropdownOpen = false">
            + Создать произведение
          </button>
          <button class="btn-menu-action" @click="emit('exportWstory'); projectDropdownOpen = false">
            ↗ Экспорт в .wstory
          </button>
          <button class="btn-menu-action" @click="emit('importWstory'); projectDropdownOpen = false">
            ↘ Импорт из .wstory
          </button>
        </div>
      </div>

      <button class="tactile-search-capsule" title="Полнотекстовый поиск (Ctrl+K)" @click="emit('openPalette')">
        <svg class="ui-icon icon-search" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="11" cy="11" r="8"/>
          <line x1="21" y1="21" x2="16.65" y2="16.65"/>
        </svg>
        <span class="search-placeholder">Поиск...</span>
        <kbd class="hotkey-pill">⌘K</kbd>
      </button>
    </div>

    <nav class="tactile-dock-container">
      <button
        :class="['dock-tab-btn', { active: currentTab === 'reader' }]"
        title="Книга"
        @click="currentTab = 'reader'"
      >
        <svg class="ui-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/>
          <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>
        </svg>
        <span>Книга</span>
      </button>

      <button
        :class="['dock-tab-btn', { active: currentTab === 'studio' }]"
        title="Студия"
        @click="currentTab = 'studio'"
      >
        <svg class="ui-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 19l7-7 3 3-7 7-3-3z"/>
          <path d="M18 13l-1.5-7.5L2 2l3.5 14.5L13 18l5-5z"/>
          <path d="M2 2l7.586 7.586"/>
          <circle cx="11" cy="11" r="2"/>
        </svg>
        <span>Студия</span>
      </button>

      <button
        :class="['dock-tab-btn', { active: currentTab === 'board' }]"
        title="Канбан"
        @click="currentTab = 'board'"
      >
        <svg class="ui-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
          <line x1="9" y1="3" x2="9" y2="21"/>
          <line x1="15" y1="3" x2="15" y2="21"/>
        </svg>
        <span>Канбан</span>
      </button>

      <button
        :class="['dock-tab-btn', { active: currentTab === 'codex' }]"
        title="Реестр"
        @click="currentTab = 'codex'"
      >
        <svg class="ui-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
        </svg>
        <span>Реестр</span>
      </button>

      <button
        :class="['dock-tab-btn', { active: currentTab === 'timeline' }]"
        title="Таймлайн"
        @click="currentTab = 'timeline'"
      >
        <svg class="ui-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10"/>
          <polyline points="12 6 12 12 16 14"/>
        </svg>
        <span>Таймлайн</span>
      </button>

      <button
        :class="['dock-tab-btn', { active: currentTab === 'graph' }]"
        title="Космос"
        @click="currentTab = 'graph'"
      >
        <svg class="ui-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="18" cy="5" r="3"/>
          <circle cx="6" cy="12" r="3"/>
          <circle cx="18" cy="19" r="3"/>
          <line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/>
          <line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/>
        </svg>
        <span>Космос</span>
      </button>
    </nav>

    <div class="header-quad-right">
      <button
        class="tactile-update-trigger"
        :class="{ pulse: updateAvailable }"
        title="Проверить обновления приложения (v1.2.0)"
        @click="emit('checkUpdate')"
      >
        <svg class="ui-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.3"/>
        </svg>
        <span class="update-version-label">v1.2.0</span>
        <span v-if="updateAvailable" class="update-dot-indicator"></span>
      </button>

      <div class="audio-control-pod">
        <button
          :class="['pod-sound-trigger', { active: ambientRain }]"
          title="Дождь"
          @click="toggleRain"
        >
          <svg class="ui-icon" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="16" y1="13" x2="16" y2="21"/>
            <line x1="8" y1="13" x2="8" y2="21"/>
            <line x1="12" y1="15" x2="12" y2="23"/>
            <path d="M20 16.58A5 5 0 0 0 18 7h-1.26A8 8 0 1 0 4 15.25"/>
          </svg>
          <span v-if="ambientRain" class="live-wave-indicator">
            <i></i><i></i><i></i>
          </span>
        </button>

        <button
          :class="['pod-sound-trigger', { active: ambientFire }]"
          title="Камин"
          @click="toggleFire"
        >
          <svg class="ui-icon" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z"/>
          </svg>
          <span v-if="ambientFire" class="live-wave-indicator">
            <i></i><i></i><i></i>
          </span>
        </button>

        <button
          :class="['pod-sound-trigger', { active: ambientWind }]"
          title="Ветер"
          @click="toggleWind"
        >
          <svg class="ui-icon" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M9.59 4.59A2 2 0 1 1 11 8H2m10.59 11.41A2 2 0 1 0 14 16H2m15.73-8.27A2.5 2.5 0 1 1 19.5 12H2"/>
          </svg>
          <span v-if="ambientWind" class="live-wave-indicator">
            <i></i><i></i><i></i>
          </span>
        </button>
      </div>

      <button
        :class="['tactile-save-trigger', { saving: isSyncing }]"
        :disabled="isSyncing"
        title="Сохранить коммит (Ctrl+S / Cmd+S)"
        @click="emit('save')"
      >
        <svg class="ui-icon" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
          <polyline points="17 21 17 13 7 13 7 21"/>
          <polyline points="7 3 7 8 15 8"/>
        </svg>
        <span class="save-label">{{ isSyncing ? 'Синхронизация...' : 'Сохранить' }}</span>
      </button>

      <button
        :class="['tactile-sprint-trigger', { running: sprintActive }]"
        title="Спринт"
        @click="emit('toggleSprint')"
      >
        <svg class="ui-icon icon-lightning" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
        </svg>
        <span v-if="sprintActive" class="sprint-numeric">
          {{ sprintFormattedTime }} <b class="sprint-velocity">+{{ sprintDelta }}</b>
        </span>
        <span v-else class="sprint-label">Спринт</span>
      </button>
    </div>
  </header>
</template>

<style scoped>
.master-quantum-header {
  position: relative;
  z-index: 50;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: calc(var(--safe-top) + 6px) 16px 6px 16px;
  background: var(--bg-surface);
  backdrop-filter: blur(40px) saturate(220%);
  border-bottom: 1px solid var(--border);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.6);
  gap: 12px;
  flex-shrink: 0;
}

.header-quad-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.ui-icon {
  flex-shrink: 0;
}

.tactile-menu-trigger {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--border);
  box-shadow: var(--specular-high);
  color: var(--text-pure);
  padding: 0 12px;
  height: 40px;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 700;
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  flex-shrink: 0;
}

.project-selector-wrapper {
  position: relative;
  flex-shrink: 0;
}

.btn-project-selector {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border-accent);
  color: var(--text-pure);
  border-radius: 10px;
  height: 40px;
  padding: 0 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 800;
  cursor: pointer;
  max-width: 200px;
  box-shadow: var(--specular-prismatic);
}

.project-current-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dropdown-arrow {
  font-size: 10px;
  color: var(--accent-cyan);
}

.project-dropdown-menu {
  position: absolute;
  top: 46px;
  left: 0;
  width: 280px;
  background: var(--bg-surface-solid);
  border: 1px solid var(--border-strong);
  border-radius: 12px;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.9);
  padding: 8px;
  z-index: 120;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.dropdown-header {
  font-size: 10px;
  font-weight: 800;
  color: var(--text-dim);
  text-transform: uppercase;
  padding: 4px 8px;
}

.project-items-scroller {
  max-height: 200px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.project-choice-row {
  padding: 8px 10px;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  transition: all 0.15s ease;
}

.project-choice-row:hover,
.project-choice-row.active {
  background: var(--bg-card-hover);
  border-left: 3px solid var(--accent-cyan);
}

.choice-main-col {
  flex: 1;
  overflow: hidden;
}

.choice-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-pure);
}

.choice-desc {
  font-size: 10px;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.btn-delete-proj {
  background: transparent;
  border: none;
  color: var(--text-dim);
  font-size: 12px;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
}

.btn-delete-proj:hover {
  background: rgba(244, 63, 94, 0.2);
  color: var(--accent-rose);
}

.dropdown-divider {
  height: 1px;
  background: var(--border-subtle);
  margin: 4px 0;
}

.btn-menu-action {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-main);
  border-radius: 8px;
  padding: 8px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  text-align: left;
  transition: all 0.15s ease;
}

.btn-menu-action:hover {
  background: var(--accent-indigo);
  color: #fff;
}

.tactile-search-capsule {
  background: rgba(0, 0, 0, 0.5);
  border: 1px solid var(--border);
  box-shadow: var(--specular-high);
  color: var(--text-muted);
  padding: 0 12px;
  height: 40px;
  border-radius: 10px;
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  min-width: 140px;
  max-width: 260px;
  flex: 1;
}

.icon-search {
  color: var(--accent-cyan);
}

.search-placeholder {
  flex: 1;
  text-align: left;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.hotkey-pill {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 700;
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid var(--border);
  padding: 2px 5px;
  border-radius: 5px;
  color: var(--text-pure);
}

.tactile-dock-container {
  display: flex;
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(24px);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 3px;
  gap: 2px;
  flex-shrink: 0;
}

.dock-tab-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 700;
  padding: 0 12px;
  height: 34px;
  border-radius: 9px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 6px;
}

.dock-tab-btn:hover {
  color: var(--text-pure);
  background: rgba(255, 255, 255, 0.06);
}

.dock-tab-btn.active {
  background: linear-gradient(135deg, var(--accent-indigo) 0%, #4338ca 100%);
  color: #fff;
  box-shadow: 0 4px 18px var(--accent-indigo-glow), var(--specular-prismatic);
}

.header-quad-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  justify-content: flex-end;
}

.tactile-update-trigger {
  position: relative;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border);
  color: var(--text-muted);
  border-radius: 10px;
  height: 40px;
  padding: 0 10px;
  display: flex;
  align-items: center;
  gap: 6px;
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tactile-update-trigger:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-pure);
}

.tactile-update-trigger.pulse {
  border-color: var(--accent-cyan);
  color: var(--accent-cyan);
}

.update-dot-indicator {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent-rose);
  animation: pulse-update 1.2s infinite alternate;
}

@keyframes pulse-update {
  from { transform: scale(0.8); opacity: 0.6; }
  to { transform: scale(1.3); opacity: 1; }
}

.audio-control-pod {
  display: flex;
  background: rgba(0, 0, 0, 0.5);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 2px;
  gap: 2px;
}

.pod-sound-trigger {
  position: relative;
  background: transparent;
  border: none;
  width: 36px;
  height: 36px;
  border-radius: 8px;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.pod-sound-trigger:hover {
  color: var(--text-pure);
}

.pod-sound-trigger.active {
  color: var(--accent-cyan);
  background: rgba(6, 182, 212, 0.16);
}

.live-wave-indicator {
  position: absolute;
  bottom: 3px;
  display: flex;
  align-items: flex-end;
  gap: 1.5px;
  height: 5px;
}

.live-wave-indicator i {
  width: 1.5px;
  background: var(--accent-cyan);
  border-radius: 1px;
  animation: wave-bar-motion 0.7s ease-in-out infinite alternate;
}

.live-wave-indicator i:nth-child(2) { animation-delay: 0.18s; }
.live-wave-indicator i:nth-child(3) { animation-delay: 0.36s; }

@keyframes wave-bar-motion {
  0% { height: 1.5px; }
  100% { height: 5px; }
}

.tactile-save-trigger {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  border-radius: 10px;
  height: 40px;
  padding: 0 16px;
  font-size: 13px;
  font-weight: 800;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  box-shadow: 0 4px 18px var(--accent-indigo-glow);
  transition: all 0.2s ease;
}

.tactile-save-trigger.saving {
  opacity: 0.7;
}

.tactile-sprint-trigger {
  background: linear-gradient(135deg, #06b6d4 0%, #3b82f6 50%, #8b5cf6 100%);
  color: #fff;
  border: none;
  box-shadow: 0 4px 18px var(--accent-cyan-glow), var(--specular-prismatic);
  border-radius: 10px;
  height: 40px;
  padding: 0 14px;
  font-size: 12px;
  font-weight: 800;
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
}

.icon-lightning {
  color: #fef08a;
}

.sprint-numeric {
  font-family: var(--font-mono);
}

.sprint-velocity {
  color: #fef08a;
}

@media (max-width: 900px) {
  .search-placeholder,
  .hotkey-pill {
    display: none;
  }
  .tactile-search-capsule {
    min-width: 40px;
    width: 40px;
    padding: 0;
    justify-content: center;
  }
}

@media (max-width: 768px) {
  .master-quantum-header {
    padding: calc(var(--safe-top) + 6px) 12px 6px 12px;
  }

  .trigger-text {
    display: none;
  }

  .tactile-dock-container {
    position: fixed;
    bottom: calc(var(--safe-bottom) + 8px);
    left: 8px;
    right: 8px;
    z-index: 100;
    justify-content: space-around;
    padding: 4px;
    background: rgba(6, 9, 16, 0.96);
    backdrop-filter: blur(28px);
    border-radius: 16px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.9), 0 0 0 1px var(--border);
  }

  .dock-tab-btn {
    flex: 1;
    flex-direction: column;
    justify-content: center;
    padding: 0;
    height: 46px;
    font-size: 9px;
    gap: 2px;
  }

  .audio-control-pod {
    display: none;
  }
}
</style>