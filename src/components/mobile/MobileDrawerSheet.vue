<script setup lang="ts">
const { isOpen, title } = defineProps<{
  isOpen: boolean;
  title: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();
</script>

<template>
  <div v-if="isOpen" class="bottom-sheet-backdrop" @click="emit('close')">
    <div class="bottom-sheet-surface" @click.stop>
      <div class="sheet-drag-handle-bar">
        <span class="sheet-pill"></span>
      </div>

      <header class="sheet-header">
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
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(12px);
  z-index: 500;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
}

.bottom-sheet-surface {
  background: var(--bg-surface-solid);
  border-top: 1px solid var(--border-strong);
  border-radius: 24px 24px 0 0;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 -10px 40px rgba(0, 0, 0, 0.9);
  padding-bottom: max(16px, env(safe-area-inset-bottom));
  animation: sheet-slide-up 0.28s cubic-bezier(0.16, 1, 0.3, 1);
}

.sheet-drag-handle-bar {
  display: flex;
  justify-content: center;
  padding: 10px 0 4px 0;
}

.sheet-pill {
  width: 38px;
  height: 4px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.2);
}

.sheet-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 20px 14px 20px;
  border-bottom: 1px solid var(--border);
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
  width: 30px;
  height: 30px;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
}

.sheet-content-scroll {
  overflow-y: auto;
  padding: 16px 20px;
  flex: 1;
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