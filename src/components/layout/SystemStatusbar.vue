<script setup lang="ts">
const {
  isSyncing,
  currentWords,
  totalSystemWords,
  currentRevision,
  syncError,
} = defineProps<{
  isSyncing: boolean;
  currentWords: number;
  totalSystemWords: number;
  currentRevision: number;
  syncError: string | null;
}>();
</script>

<template>
  <footer class="system-statusbar">
    <div class="status-left">
      <span class="status-indicator" :class="{ syncing: isSyncing }"></span>
      <span>SQLite WAL</span>
      <span class="status-divider">/</span>
      <span>Слов: <b>{{ currentWords }}</b></span>
      <span class="status-divider">/</span>
      <span>Всего: <b>{{ totalSystemWords }}</b></span>
      <span class="status-divider">/</span>
      <span>Ревизия: <b>r{{ currentRevision }}</b></span>
    </div>
    <div class="status-right">
      <span>Wingnite v1.3.0 • {{ syncError || (isSyncing ? 'Синхронизация...' : 'Ядро верифицировано') }}</span>
    </div>
  </footer>
</template>

<style scoped>
.system-statusbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 24px;
  padding: 0 12px;
  background: var(--bg-surface-solid);
  border-top: 1px solid var(--border);
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--text-muted);
}

.status-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-indicator {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent-emerald);
}

.status-indicator.syncing {
  background: var(--accent-gold);
}

.status-divider {
  color: var(--border);
}

.status-right {
  color: var(--text-dim);
}
</style>