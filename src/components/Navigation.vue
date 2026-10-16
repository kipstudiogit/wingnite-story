<script setup lang="ts">
import { ref } from "vue";
import type { AppTab } from "../types";
import { ambientEngine } from "../ambient";

const currentTab = defineModel<AppTab>("currentTab", { required: true });

const { sprintActive, sprintFormattedTime, sprintDelta } = defineProps<{
  sprintActive: boolean;
  sprintFormattedTime: string;
  sprintDelta: number;
}>();

const emit = defineEmits<{
  (e: "toggleToc"): void;
  (e: "openPalette"): void;
  (e: "toggleSprint"): void;
}>();

const ambientRain = ref(false);
const ambientFire = ref(false);
const ambientWind = ref(false);

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
        <span class="trigger-text">Кодекс</span>
      </button>

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
  min-width: 180px;
  max-width: 320px;
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