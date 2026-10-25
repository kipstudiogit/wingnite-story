<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { EnterpriseStoryItem, FtsResultItem } from "../types";

const { stories } = defineProps<{
  stories: EnterpriseStoryItem[];
}>();

const emit = defineEmits<{
  (e: "select", story: EnterpriseStoryItem): void;
  (e: "close"): void;
}>();

const search = ref("");
const selectedIndex = ref(0);
const inputRef = ref<HTMLInputElement | null>(null);
const ftsResults = ref<FtsResultItem[]>([]);
let searchTimer: number | null = null;

watch(search, (query) => {
  if (searchTimer) clearTimeout(searchTimer);
  const q = query.trim();
  if (!q) {
    ftsResults.value = [];
    return;
  }
  searchTimer = window.setTimeout(async () => {
    try {
      ftsResults.value = await invoke<FtsResultItem[]>("execute_fts_search", { query: q });
      selectedIndex.value = 0;
    } catch {
      ftsResults.value = [];
    }
  }, 60);
});

const displayItems = computed(() => {
  if (search.value.trim() && ftsResults.value.length > 0) {
    return ftsResults.value.map((res) => {
      const match = stories.find((s) => s.id === res.id);
      return {
        id: res.id,
        title: res.title,
        category: res.category,
        snippet: res.snippet,
        story: match ?? {
          id: res.id,
          projectId: "default_project",
          title: res.title,
          category: res.category,
          content: "",
          revision: 1,
          sortOrder: 0,
          contentHash: "",
          updatedAt: 0,
        },
      };
    });
  }

  const q = search.value.toLowerCase().trim();
  const list = q
    ? stories.filter(
        (s) => s.title.toLowerCase().includes(q) || s.category.toLowerCase().includes(q)
      )
    : stories;

  return list.map((s) => ({
    id: s.id,
    title: s.title,
    category: s.category,
    snippet: "",
    story: s,
  }));
});

function handleKeyDown(e: KeyboardEvent) {
  if (e.key === "ArrowDown") {
    e.preventDefault();
    if (displayItems.value.length > 0) {
      selectedIndex.value = (selectedIndex.value + 1) % displayItems.value.length;
    }
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    if (displayItems.value.length > 0) {
      selectedIndex.value =
        (selectedIndex.value - 1 + displayItems.value.length) % displayItems.value.length;
    }
  } else if (e.key === "Enter") {
    e.preventDefault();
    const item = displayItems.value[selectedIndex.value];
    if (item) {
      emit("select", item.story);
    }
  } else if (e.key === "Escape") {
    emit("close");
  }
}

onMounted(() => {
  inputRef.value?.focus();
  window.addEventListener("keydown", handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeyDown);
});
</script>

<template>
  <div class="cmd-overlay" @click.self="emit('close')">
    <div class="cmd-modal">
      <div class="cmd-search-row">
        <span class="cmd-icon">⚡</span>
        <input
          ref="inputRef"
          v-model="search"
          class="cmd-input"
          placeholder="Полнотекстовый поиск FTS5 по всей книге..."
        />
        <span class="cmd-shortcut-tag">ESC</span>
      </div>

      <div class="cmd-meta-bar">
        <span>Результаты FTS5: {{ displayItems.length }}</span>
        <span>Навигация: ↑ ↓ &nbsp;•&nbsp; Выбор: ↵</span>
      </div>

      <div class="cmd-list">
        <div
          v-for="(item, idx) in displayItems"
          :key="item.id"
          :class="['cmd-item', { active: idx === selectedIndex }]"
          @mouseenter="selectedIndex = idx"
          @click="emit('select', item.story)"
        >
          <div class="cmd-item-main">
            <div class="cmd-item-top">
              <span class="item-title">{{ item.title }}</span>
              <span class="item-badge">{{ item.category }}</span>
            </div>
            <div v-if="item.snippet" class="item-snippet" v-html="item.snippet"></div>
          </div>
        </div>

        <div v-if="displayItems.length === 0" class="cmd-empty">
          Совпадений в базе данных не обнаружено
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.cmd-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(14px);
  z-index: 200;
  display: flex;
  justify-content: center;
  padding-top: 10vh;
}

.cmd-modal {
  width: 90%;
  max-width: 650px;
  background: var(--bg-surface-solid);
  border: 1px solid var(--border-strong);
  border-radius: 18px;
  box-shadow: 0 30px 80px rgba(0, 0, 0, 0.9);
  overflow: hidden;
  max-height: 480px;
  display: flex;
  flex-direction: column;
}

.cmd-search-row {
  display: flex;
  align-items: center;
  padding: 14px 18px;
  border-bottom: 1px solid var(--border);
  gap: 12px;
}

.cmd-icon {
  color: var(--accent-cyan);
  font-size: 16px;
}

.cmd-input {
  flex: 1;
  background: transparent;
  border: none;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-pure);
  outline: none;
}

.cmd-shortcut-tag {
  font-family: var(--font-mono);
  font-size: 9px;
  font-weight: 700;
  padding: 3px 6px;
  border: 1px solid var(--border);
  border-radius: 5px;
  color: var(--text-muted);
  background: var(--bg-card);
}

.cmd-meta-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 18px;
  background: rgba(0, 0, 0, 0.3);
  border-bottom: 1px solid var(--border-subtle);
  font-size: 11px;
  color: var(--text-dim);
  font-family: var(--font-mono);
}

.cmd-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.cmd-item {
  display: flex;
  padding: 10px 14px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.15s ease;
  margin-bottom: 4px;
}

.cmd-item:hover,
.cmd-item.active {
  background: var(--bg-card-hover);
  box-shadow: 0 0 16px var(--accent-indigo-glow);
}

.cmd-item-main {
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: 100%;
}

.cmd-item-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.item-title {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-pure);
}

.item-badge {
  font-size: 10px;
  font-weight: 700;
  color: var(--accent-cyan);
  background: rgba(6, 182, 212, 0.12);
  padding: 2px 8px;
  border-radius: 999px;
  border: 1px solid rgba(6, 182, 212, 0.25);
}

.item-snippet {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.4;
}

.item-snippet :deep(b) {
  color: var(--accent-gold);
  background: rgba(245, 158, 11, 0.15);
  padding: 0 2px;
  border-radius: 3px;
}

.cmd-empty {
  padding: 40px;
  text-align: center;
  font-size: 13px;
  color: var(--text-dim);
}
</style>