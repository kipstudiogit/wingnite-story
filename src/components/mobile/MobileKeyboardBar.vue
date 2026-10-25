<script setup lang="ts">
const { isSyncing = false } = defineProps<{
  isSyncing?: boolean;
}>();

const emit = defineEmits<{
  (e: "insert", prefix: string, suffix: string, placeholder?: string): void;
  (e: "save"): void;
}>();
</script>

<template>
  <div class="mobile-keyboard-accessory-bar" @mousedown.prevent @touchstart.prevent>
    <div class="accessory-scroll-track">
      <button class="acc-btn" @click="emit('insert', '\n# ', '\n', 'Заголовок')">H1</button>
      <button class="acc-btn" @click="emit('insert', '\n## ', '\n', 'Подзаголовок')">H2</button>
      <button class="acc-btn" @click="emit('insert', '\n### ', '\n', 'Секция')">H3</button>
      <button class="acc-btn font-bold" @click="emit('insert', '**', '**', 'текст')">B</button>
      <button class="acc-btn font-italic" @click="emit('insert', '*', '*', 'текст')">I</button>
      <button class="acc-btn" @click="emit('insert', '~~', '~~', 'текст')">S</button>
      <button class="acc-btn accent-btn" @click="emit('insert', '[[', ']]', 'Сущность')">[[ Wiki ]]</button>
      <button class="acc-btn accent-btn" @click="emit('insert', '@', ' ', 'Персонаж')">@Имя</button>
      <button class="acc-btn highlight-btn" @click="emit('insert', '==', '==', 'важное')">==Выделить==</button>
      <button class="acc-btn spoiler-btn" @click="emit('insert', '||', '||', 'спойлер')">||Спойлер||</button>
      <button class="acc-btn" @click="emit('insert', '\n> «', '»\n', 'Цитата')">« »</button>
      <button class="acc-btn" @click="emit('insert', '\n- [ ] ', '\n', 'Задача')">☑ Чеклист</button>
      <button class="acc-btn" @click="emit('insert', '\n- ', '\n', 'Пункт')">• Список</button>
      <button class="acc-btn" @click="emit('insert', '`', '`', 'код')">&lt;/&gt;</button>
      <button class="acc-btn" @click="emit('insert', '\n\n---\n\n', '', '')">— Разделитель</button>
    </div>

    <button
      class="acc-save-btn"
      :disabled="isSyncing"
      @click="emit('save')"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
        <polyline points="17 21 17 13 7 13 7 21"/>
        <polyline points="7 3 7 8 15 8"/>
      </svg>
      <span>{{ isSyncing ? '...' : 'Сохранить' }}</span>
    </button>
  </div>
</template>

<style scoped>
.mobile-keyboard-accessory-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: rgba(8, 12, 22, 0.98);
  backdrop-filter: blur(24px);
  border-top: 1px solid var(--border);
  padding: 6px 10px;
  gap: 8px;
  width: 100%;
  flex-shrink: 0;
  z-index: 45;
  touch-action: pan-x;
}

.accessory-scroll-track {
  display: flex;
  align-items: center;
  gap: 6px;
  overflow-x: auto;
  flex: 1;
  padding-bottom: 2px;
  -webkit-overflow-scrolling: touch;
}

.acc-btn {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid var(--border);
  color: var(--text-main);
  border-radius: 8px;
  min-width: 36px;
  height: 36px;
  padding: 0 10px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  white-space: nowrap;
  display: flex;
  align-items: center;
  justify-content: center;
  user-select: none;
  touch-action: manipulation;
}

.acc-btn:active {
  background: var(--accent-indigo);
  color: #fff;
  transform: scale(0.95);
}

.font-bold {
  font-weight: 900;
}

.font-italic {
  font-style: italic;
}

.accent-btn {
  color: var(--accent-cyan);
  border-color: rgba(6, 182, 212, 0.35);
  background: rgba(6, 182, 212, 0.1);
}

.highlight-btn {
  color: #fef08a;
  border-color: rgba(245, 158, 11, 0.35);
  background: rgba(245, 158, 11, 0.1);
}

.spoiler-btn {
  color: var(--accent-purple);
  border-color: rgba(168, 85, 247, 0.35);
  background: rgba(168, 85, 247, 0.1);
}

.acc-save-btn {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  border-radius: 8px;
  height: 36px;
  padding: 0 12px;
  font-size: 11px;
  font-weight: 800;
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  cursor: pointer;
  user-select: none;
}

.acc-save-btn:disabled {
  opacity: 0.6;
}
</style>