<script setup lang="ts">
import { ref } from "vue";

const { isOpen, title } = defineProps<{
  isOpen: boolean;
  title: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const touchStartY = ref(0);
const touchCurrentY = ref(0);
const sheetTranslateY = ref(0);
const isDragging = ref(false);

function handleTouchStart(e: TouchEvent) {
  touchStartY.value = e.touches[0].clientY;
  touchCurrentY.value = touchStartY.value;
  isDragging.value = true;
}

function handleTouchMove(e: TouchEvent) {
  if (!isDragging.value) return;
  touchCurrentY.value = e.touches[0].clientY;
  const delta = touchCurrentY.value - touchStartY.value;
  if (delta > 0) {
    sheetTranslateY.value = delta;
  }
}

function handleTouchEnd() {
  if (!isDragging.value) return;
  isDragging.value = false;
  if (sheetTranslateY.value > 120) {
    emit("close");
  }
  sheetTranslateY.value = 0;
}
</script>

<template>
  <div v-if="isOpen" class="bottom-sheet-backdrop" @click="emit('close')">
    <div
      class="bottom-sheet-surface"
      :style="{ transform: sheetTranslateY > 0 ? `translateY(${sheetTranslateY}px)` : undefined }"
      @click.stop
    >
      <div
        class="sheet-drag-handle-bar"
        @touchstart="handleTouchStart"
        @touchmove="handleTouchMove"
        @touchend="handleTouchEnd"
      >
        <span class="sheet-pill"></span>
      </div>

      <header
        class="sheet-header"
        @touchstart="handleTouchStart"
        @touchmove="handleTouchMove"
        @touchend="handleTouchEnd"
      >
        <h3 class="sheet-title">{{ title }}</h3>
        <button class="sheet-close-btn" @click="emit('close')">✕</button>
      </header>

      <div class="sheet-content-scroll">
        <slot></slot>
      </div>
    </div>
  </div>
</template>

<style scoped>
.bottom-sheet-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.78);
  backdrop-filter: blur(14px);
  z-index: 500;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
}

.bottom-sheet-surface {
  background: var(--bg-surface-solid);
  border-top: 1px solid var(--border-strong);
  border-radius: 24px 24px 0 0;
  max-height: 86vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 -12px 48px rgba(0, 0, 0, 0.95);
  padding-bottom: max(16px, env(safe-area-inset-bottom));
  animation: sheet-slide-up 0.28s cubic-bezier(0.16, 1, 0.3, 1);
  transition: transform 0.15s ease-out;
  touch-action: pan-y;
}

.sheet-drag-handle-bar {
  display: flex;
  justify-content: center;
  padding: 12px 0 6px 0;
  cursor: grab;
}

.sheet-pill {
  width: 42px;
  height: 5px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.24);
}

.sheet-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 20px 14px 20px;
  border-bottom: 1px solid var(--border);
  user-select: none;
}

.sheet-title {
  font-size: 16px;
  font-weight: 800;
  color: var(--text-pure);
}

.sheet-close-btn {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid var(--border);
  color: var(--text-muted);
  width: 32px;
  height: 32px;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
}

.sheet-content-scroll {
  overflow-y: auto;
  padding: 16px 20px;
  flex: 1;
  -webkit-overflow-scrolling: touch;
}

@keyframes sheet-slide-up {
  from {
    transform: translateY(100%);
  }
  to {
    transform: translateY(0);
  }
}
</style>