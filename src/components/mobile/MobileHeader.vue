<script setup lang="ts">
const {
  currentTitle,
  currentWords,
  currentRevision,
  isSyncing,
} = defineProps<{
  currentTitle: string;
  currentWords: number;
  currentRevision: number;
  isSyncing: boolean;
}>();

const emit = defineEmits<{
  (e: "openToc"): void;
  (e: "save"): void;
}>();
</script>

<template>
  <header class="mobile-master-header">
    <button class="mobile-action-icon-btn" title="Оглавление" @click="emit('openToc')">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="3" y1="6" x2="21" y2="6"/>
        <line x1="3" y1="12" x2="15" y2="12"/>
        <line x1="3" y1="18" x2="18" y2="18"/>
      </svg>
    </button>

    <div class="mobile-header-meta">
      <h2 class="mobile-title-text">{{ currentTitle || 'Сцена без названия' }}</h2>
      <span class="mobile-words-badge">{{ currentWords }} сл. • r{{ currentRevision }}</span>
    </div>

    <button
      class="mobile-save-pill"
      :class="{ saving: isSyncing }"
      :disabled="isSyncing"
      @click="emit('save')"
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
        <polyline points="17 21 17 13 7 13 7 21"/>
        <polyline points="7 3 7 8 15 8"/>
      </svg>
    </button>
  </header>
</template>

<style scoped>
.mobile-master-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: max(10px, env(safe-area-inset-top)) 16px 10px 16px;
  background: var(--bg-surface);
  backdrop-filter: blur(28px);
  border-bottom: 1px solid var(--border);
  gap: 12px;
  flex-shrink: 0;
  z-index: 50;
}

.mobile-action-icon-btn {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border);
  color: var(--text-pure);
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.mobile-header-meta {
  flex: 1;
  overflow: hidden;
  text-align: center;
}

.mobile-title-text {
  font-size: 14px;
  font-weight: 800;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--text-pure);
}

.mobile-words-badge {
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--accent-cyan);
}

.mobile-save-pill {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.mobile-save-pill.saving {
  opacity: 0.6;
}
</style>