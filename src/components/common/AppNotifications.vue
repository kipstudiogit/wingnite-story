<script setup lang="ts">
import type { UpdateStatusPayload } from "../../types";

const {
  exportNotification,
  plotTwist,
  updateAvailable,
  updateInfo,
  isInstallingUpdate,
} = defineProps<{
  exportNotification: string | null;
  plotTwist: string;
  updateAvailable: boolean;
  updateInfo: UpdateStatusPayload | null;
  isInstallingUpdate: boolean;
}>();

const emit = defineEmits<{
  (e: "closeToast"): void;
  (e: "insertTwist"): void;
  (e: "dismissTwist"): void;
  (e: "installUpdate"): void;
  (e: "dismissUpdate"): void;
}>();
</script>

<template>
  <div class="notifications-layer">
    <div
      v-if="exportNotification"
      class="export-toast-banner"
      :class="{ error: exportNotification.startsWith('Ошибка') }"
    >
      <span>{{ exportNotification }}</span>
      <button class="toast-close" @click="emit('closeToast')">✕</button>
    </div>

    <div v-if="updateAvailable && updateInfo" class="update-ribbon-card">
      <div class="update-ribbon-left">
        <span class="ribbon-sparkle">✦</span>
        <div class="ribbon-text-col">
          <span class="ribbon-title">Доступно обновление Wingnite Story {{ updateInfo.version }}</span>
          <span class="ribbon-sub">{{ updateInfo.body || 'Новые функции, оптимизации и исправления' }}</span>
        </div>
      </div>
      <div class="update-ribbon-right">
        <button
          class="btn-ribbon-install"
          :disabled="isInstallingUpdate"
          @click="emit('installUpdate')"
        >
          {{ isInstallingUpdate ? 'Установка...' : 'Обновить и перезапустить' }}
        </button>
        <button class="btn-ribbon-close" @click="emit('dismissUpdate')">✕</button>
      </div>
    </div>

    <div v-if="plotTwist" class="twist-banner">
      <div class="twist-tag">ТВИСТ</div>
      <p class="twist-text">{{ plotTwist }}</p>
      <div class="twist-action-row">
        <button class="twist-insert-btn" @click="emit('insertTwist')">Вставить</button>
        <button class="twist-dismiss" @click="emit('dismissTwist')">✕</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.notifications-layer {
  display: flex;
  flex-direction: column;
  position: relative;
  z-index: 99;
}

.export-toast-banner {
  position: fixed;
  top: max(14px, env(safe-area-inset-top));
  right: 14px;
  left: 14px;
  z-index: 999;
  background: rgba(16, 185, 129, 0.95);
  color: #fff;
  padding: 12px 16px;
  border-radius: 14px;
  font-size: 13px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: space-between;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.6);
  max-width: 520px;
  margin: 0 auto;
}

.export-toast-banner.error {
  background: rgba(244, 63, 94, 0.95);
}

.toast-close {
  background: transparent;
  border: none;
  color: #fff;
  cursor: pointer;
  font-size: 14px;
}

.update-ribbon-card {
  background: linear-gradient(90deg, rgba(99, 102, 241, 0.95) 0%, rgba(6, 182, 212, 0.95) 100%);
  color: #fff;
  padding: 8px 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
  flex-shrink: 0;
}

.update-ribbon-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.ribbon-sparkle {
  color: #fef08a;
  font-size: 16px;
}

.ribbon-text-col {
  display: flex;
  flex-direction: column;
}

.ribbon-title {
  font-size: 12px;
  font-weight: 800;
  letter-spacing: -0.01em;
}

.ribbon-sub {
  font-size: 11px;
  opacity: 0.9;
}

.update-ribbon-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-ribbon-install {
  background: #ffffff;
  color: var(--accent-indigo);
  border: none;
  border-radius: 8px;
  padding: 6px 14px;
  font-size: 11px;
  font-weight: 800;
  cursor: pointer;
  transition: transform 0.15s ease;
}

.btn-ribbon-install:hover {
  transform: scale(1.03);
}

.btn-ribbon-close {
  background: transparent;
  border: none;
  color: #fff;
  cursor: pointer;
  font-size: 14px;
  padding: 4px;
}

.twist-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: rgba(99, 102, 241, 0.15);
  border-bottom: 1px solid var(--accent-indigo);
  font-size: 12px;
}

.twist-tag {
  font-size: 9px;
  font-weight: 900;
  background: var(--accent-indigo);
  color: #fff;
  padding: 2px 6px;
  border-radius: 4px;
}

.twist-text {
  flex: 1;
  margin: 0 10px;
}

.twist-action-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.twist-insert-btn {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  font-size: 11px;
  font-weight: 700;
  padding: 4px 10px;
  border-radius: 6px;
  cursor: pointer;
}

.twist-dismiss {
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--text-muted);
}
</style>