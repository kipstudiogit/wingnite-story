<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { computed } from "vue";
import type { AppTab } from "../../types";
import { useI18n } from "../../i18n";

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

const { t } = useI18n();

const sprintDisplayLabel = computed(() =>
  sprintActive ? sprintFormattedTime : t("mobile.sprintTab")
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
      <span class="dock-label">{{ t("mobile.studioTab") }}</span>
    </button>

    <button
      :class="['mobile-dock-item', { active: currentTab === 'reader' }]"
      @click="currentTab = 'reader'"
    >
      <svg class="dock-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/>
        <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>
      </svg>
      <span class="dock-label">{{ t("mobile.readerTab") }}</span>
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
      <span class="dock-label">{{ t("mobile.boardTab") }}</span>
    </button>

    <button
      :class="['mobile-dock-item', { active: currentTab === 'codex' }]"
      @click="currentTab = 'codex'"
    >
      <svg class="dock-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
      </svg>
      <span class="dock-label">{{ t("mobile.codexTab") }}</span>
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
      <span class="dock-label">{{ t("mobile.menuTab") }}</span>
    </button>
  </nav>
</template>

<style scoped>
.mobile-bottom-dock {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-surface-solid);
  border-top: 1px solid var(--border);
  padding: 6px max(6px, var(--safe-right)) max(10px, var(--safe-bottom)) max(6px, var(--safe-left));
  gap: 2px;
  position: relative;
  z-index: 100;
  flex-shrink: 0;
  box-shadow: 0 -6px 24px rgba(0, 0, 0, 0.25);
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
  flex: 1;
  min-width: 0;
  height: 48px;
  padding: 0 2px;
  cursor: pointer;
  border-radius: 12px;
  transition: all 0.15s ease;
  user-select: none;
  touch-action: manipulation;
}

.mobile-dock-item:active {
  transform: scale(0.92);
  background: var(--bg-card);
}

.mobile-dock-item.active {
  color: var(--accent-indigo);
  background: var(--bg-card);
}

.sprint-action-btn.active {
  color: var(--accent-gold);
  background: var(--bg-card);
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
  max-width: 100%;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

@media (max-width: 360px) {
  .dock-label {
    font-size: 9px;
  }

  .mobile-dock-item {
    height: 44px;
  }
}
</style>