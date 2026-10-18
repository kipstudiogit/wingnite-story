<script setup lang="ts">
import { computed } from "vue";
import type { AppTab } from "../../types";

const currentTab = defineModel<AppTab>("currentTab", { required: true });

const { sprintActive, sprintFormattedTime } = defineProps<{
  sprintActive: boolean;
  sprintFormattedTime: string;
}>();

const emit = defineEmits<{
  (e: "openMenu"): void;
  (e: "openSearch"): void;
  (e: "toggleSprint"): void;
}>();

const sprintDisplayLabel = computed(() =>
  sprintActive ? sprintFormattedTime : "Спринт"
);
</script>

<template>
  <nav class="mobile-bottom-dock">
    <button
      :class="['mobile-dock-item', { active: currentTab === 'studio' }]"
      @click="currentTab = 'studio'"
    >
      <svg class="dock-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 19l7-7 3 3-7 7-3-3z"/>
        <path d="M18 13l-1.5-7.5L2 2l3.5 14.5L13 18l5-5z"/>
        <path d="M2 2l7.586 7.586"/>
        <circle cx="11" cy="11" r="2"/>
      </svg>
      <span class="dock-label">Студия</span>
    </button>

    <button
      :class="['mobile-dock-item', { active: currentTab === 'reader' }]"
      @click="currentTab = 'reader'"
    >
      <svg class="dock-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/>
        <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>
      </svg>
      <span class="dock-label">Книга</span>
    </button>

    <button
      :class="['mobile-dock-item', { active: currentTab === 'board' }]"
      @click="currentTab = 'board'"
    >
      <svg class="dock-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
        <line x1="9" y1="3" x2="9" y2="21"/>
        <line x1="15" y1="3" x2="15" y2="21"/>
      </svg>
      <span class="dock-label">Канбан</span>
    </button>

    <button
      :class="['mobile-dock-item', { active: currentTab === 'codex' }]"
      @click="currentTab = 'codex'"
    >
      <svg class="dock-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
      </svg>
      <span class="dock-label">Реестр</span>
    </button>

    <button
      :class="['mobile-dock-item', 'sprint-action-btn', { active: sprintActive }]"
      @click="emit('toggleSprint')"
    >
      <svg class="dock-icon icon-lightning" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
      </svg>
      <span class="dock-label">{{ sprintDisplayLabel }}</span>
    </button>

    <button class="mobile-dock-item" @click="emit('openMenu')">
      <svg class="dock-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="1"/>
        <circle cx="19" cy="12" r="1"/>
        <circle cx="5" cy="12" r="1"/>
      </svg>
      <span class="dock-label">Меню</span>
    </button>
  </nav>
</template>

<style scoped>
.mobile-bottom-dock {
  display: flex;
  align-items: center;
  justify-content: space-around;
  background: rgba(6, 9, 16, 0.95);
  backdrop-filter: blur(28px) saturate(200%);
  border-top: 1px solid var(--border);
  padding: 6px 4px max(8px, env(safe-area-inset-bottom)) 4px;
  position: relative;
  z-index: 100;
  flex-shrink: 0;
  box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.8);
}

.mobile-dock-item {
  background: transparent;
  border: none;
  color: var(--text-muted);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 3px;
  min-width: 48px;
  height: 48px;
  cursor: pointer;
  border-radius: 10px;
  transition: all 0.15s ease;
}

.mobile-dock-item:active {
  transform: scale(0.92);
}

.mobile-dock-item.active {
  color: var(--accent-cyan);
}

.sprint-action-btn.active {
  color: #fef08a;
}

.icon-lightning {
  color: inherit;
}

.dock-icon {
  flex-shrink: 0;
}

.dock-label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: -0.01em;
}
</style>