<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import type { StudioDrawerType } from "../../types";
import { useI18n } from "../../i18n";

const title = defineModel<string>("title", { required: true });
const category = defineModel<string>("category", { required: true });

const {
  revision,
  vaultSealed,
  vaultLocked,
  consistencyScore,
  activeDrawer,
  hasActive,
} = defineProps<{
  revision: number;
  vaultSealed: boolean;
  vaultLocked: boolean;
  consistencyScore: number;
  activeDrawer: StudioDrawerType;
  hasActive: boolean;
}>();

const emit = defineEmits<{
  (e: "toggleDrawer", drawer: Exclude<StudioDrawerType, null>): void;
  (e: "delete"): void;
}>();

const { t } = useI18n();
</script>

<template>
  <div class="editor-telemetry-header">
    <div class="meta-input-capsules">
      <input
        v-model="title"
        class="capsule-title-input"
        :readonly="vaultLocked"
        :placeholder="t('studio.titlePlaceholder')"
      />
      <input
        v-model="category"
        class="capsule-category-input"
        :readonly="vaultLocked"
        :placeholder="t('studio.categoryPlaceholder')"
      />
    </div>

    <div class="header-action-controls">
      <div class="revision-telemetry-badge">
        <span class="rev-label">{{ t("studio.revBadge") }}</span>
        <span class="rev-digit">{{ revision }}</span>
      </div>

      <div
        class="vault-status-indicator"
        :class="{ sealed: vaultSealed && !vaultLocked, locked: vaultLocked }"
        :title="t('studio.vaultTooltip')"
        @click="emit('toggleDrawer', 'vault')"
      >
        <span class="vault-orb"></span>
        <span class="vault-text">
          {{ vaultLocked ? t("studio.vaultLocked") : vaultSealed ? t("studio.vaultSealed") : t("studio.vaultUnsealed") }}
        </span>
      </div>

      <div
        class="lore-score-badge"
        :class="{ warning: consistencyScore < 80, danger: consistencyScore < 50 }"
        :title="t('studio.loreBadgeTooltip')"
        @click="emit('toggleDrawer', 'lore')"
      >
        <span>LOR</span>
        <b>{{ Math.round(consistencyScore) }}%</b>
      </div>

      <button
        class="tactile-icon-btn"
        :class="{ active: activeDrawer === 'lore' }"
        :title="t('studio.loreTooltip')"
        @click="emit('toggleDrawer', 'lore')"
      >
        <svg class="ui-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="m4.93 4.93 4.24 4.24"/><path d="m14.83 9.17 4.24-4.24"/><path d="m14.83 14.83 4.24 4.24"/><path d="m9.17 14.83-4.24 4.24"/></svg>
      </button>

      <button
        class="tactile-icon-btn"
        :class="{ active: activeDrawer === 'vault' }"
        :title="t('studio.vaultTooltipBtn')"
        @click="emit('toggleDrawer', 'vault')"
      >
        <svg class="ui-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
      </button>

      <button
        class="tactile-icon-btn"
        :class="{ active: activeDrawer === 'analytics' }"
        :title="t('studio.analyticsTooltip')"
        @click="emit('toggleDrawer', 'analytics')"
      >
        <svg class="ui-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="20" x2="18" y2="10"/><line x1="12" y1="20" x2="12" y2="4"/><line x1="6" y1="20" x2="6" y2="14"/></svg>
      </button>

      <button
        v-if="hasActive && !vaultLocked"
        class="tactile-icon-btn btn-danger-tone"
        :title="t('studio.deleteSceneTooltip')"
        @click="emit('delete')"
      >
        <svg class="ui-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.editor-telemetry-header {
  display: flex;
  gap: 10px;
  align-items: center;
}

.meta-input-capsules {
  display: flex;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.capsule-title-input {
  flex: 3;
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-pure);
  padding: 0 14px;
  height: 42px;
  border-radius: 12px;
  font-size: 14px;
  font-weight: 700;
  outline: none;
  min-width: 0;
}

.capsule-category-input {
  flex: 1;
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-main);
  padding: 0 12px;
  height: 42px;
  border-radius: 12px;
  font-size: 13px;
  font-weight: 600;
  outline: none;
  min-width: 84px;
}

.header-action-controls {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-shrink: 0;
}

.revision-telemetry-badge {
  display: flex;
  align-items: center;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  height: 40px;
  padding: 0 10px;
  gap: 6px;
  font-family: var(--font-mono);
  flex-shrink: 0;
}

.rev-label {
  font-size: 10px;
  color: var(--accent-cyan);
  font-weight: 800;
}

.rev-digit {
  font-size: 13px;
  font-weight: 800;
  color: var(--text-pure);
}

.vault-status-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  height: 40px;
  padding: 0 10px;
  cursor: pointer;
  flex-shrink: 0;
}

.vault-orb {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--accent-cyan);
}

.vault-status-indicator.sealed .vault-orb {
  background: var(--accent-emerald);
}

.vault-status-indicator.locked .vault-orb {
  background: var(--accent-rose);
}

.vault-text {
  font-size: 11px;
  font-weight: 800;
  font-family: var(--font-mono);
  color: var(--text-muted);
}

.lore-score-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--accent-emerald);
  height: 40px;
  padding: 0 10px;
  border-radius: 10px;
  font-size: 11px;
  font-family: var(--font-mono);
  cursor: pointer;
  flex-shrink: 0;
}

.lore-score-badge.warning { color: var(--accent-gold); }
.lore-score-badge.danger { color: var(--accent-rose); }

.tactile-icon-btn {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-muted);
  border-radius: 12px;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
}

.tactile-icon-btn.active {
  background: var(--accent-indigo);
  color: #fff;
}

.btn-danger-tone:hover {
  background: rgba(244, 63, 94, 0.2);
  color: var(--accent-rose);
}

@media (max-width: 900px) {
  .editor-telemetry-header {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }

  .header-action-controls {
    width: 100%;
    overflow-x: auto;
    padding-bottom: 2px;
    -webkit-overflow-scrolling: touch;
  }

  .tactile-icon-btn {
    width: 42px;
    height: 42px;
  }
}
</style>