<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { useConfirmDialog } from "../../composables/useConfirmDialog";
import { useI18n } from "../../i18n";

const { confirmState, resolveConfirm } = useConfirmDialog();
const { t } = useI18n();
</script>

<template>
  <div v-if="confirmState.isOpen" class="confirm-backdrop" @click.self="resolveConfirm(false)">
    <div class="confirm-window">
      <h3>{{ confirmState.title }}</h3>
      <p>{{ confirmState.message }}</p>
      <div class="confirm-actions">
        <button class="btn-confirm-cancel" @click="resolveConfirm(false)">
          {{ confirmState.cancelLabel || t("common.cancel") }}
        </button>
        <button
          :class="['btn-confirm-accept', { danger: confirmState.danger }]"
          @click="resolveConfirm(true)"
        >
          {{ confirmState.confirmLabel || t("common.confirm") }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.confirm-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.78);
  backdrop-filter: blur(14px);
  z-index: 600;
  display: flex;
  align-items: center;
  justify-content: center;
}

.confirm-window {
  width: 90%;
  max-width: 420px;
  background: var(--bg-surface-solid);
  border: 1px solid var(--border-strong);
  border-radius: 16px;
  padding: 22px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.9);
}

.confirm-window h3 {
  font-size: 16px;
  font-weight: 800;
  color: var(--text-pure);
}

.confirm-window p {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.55;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 6px;
}

.btn-confirm-cancel {
  background: transparent;
  border: 1px solid var(--border);
  color: var(--text-muted);
  padding: 8px 16px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.btn-confirm-accept {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  padding: 8px 18px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 800;
  cursor: pointer;
}

.btn-confirm-accept.danger {
  background: var(--accent-rose);
}
</style>