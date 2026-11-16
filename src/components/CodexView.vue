<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { classifyEntityCategory, isCodexEntityCategory } from "../constants/app";
import { renderMarkdown } from "../utils/markdown";
import type { EntityKind, EntityRelationItem, StoryItem } from "../types";
import { useHaptics } from "../composables/useHaptics";
import { useI18n } from "../i18n";
import CodexCreatorDrawer from "./codex/CodexCreatorDrawer.vue";
import CodexDossierModal from "./codex/CodexDossierModal.vue";

const { projectId, stories, operator = "Codex-Custodian" } = defineProps<{
  projectId: string;
  stories: StoryItem[];
  operator?: string;
}>();

const emit = defineEmits<{
  (e: "select", story: StoryItem): void;
  (e: "saveEntity", story: StoryItem): void;
  (e: "notify", msg: string, kind: "success" | "error"): void;
}>();

const { vibrateLight, vibrateSelection, vibrateSuccess } = useHaptics();
const { t, messages } = useI18n();

const activeFilter = ref<"ALL" | EntityKind>("ALL");
const searchQuery = ref("");
const sortBy = ref<"title" | "impact" | "revision">("impact");
const viewLayout = ref<"grid" | "table">("grid");
const showCreatorDrawer = ref(false);
const selectedEntity = ref<StoryItem | null>(null);
const projectRelations = ref<EntityRelationItem[]>([]);

interface EntityImpactMetrics {
  mentions: number;
  links: number;
  score: number;
}

async function loadProjectRelations() {
  try {
    projectRelations.value = await invoke<EntityRelationItem[]>("fetch_entity_relations", {
      projectId,
    });
  } catch {
    projectRelations.value = [];
  }
}

const filterPills = computed(() => [
  { key: "ALL" as const, label: messages.value.codex.filters.all, icon: "❖" },
  { key: "character" as const, label: messages.value.codex.filters.character, icon: "👤" },
  { key: "artifact" as const, label: messages.value.codex.filters.artifact, icon: "🔮" },
  { key: "pantheon" as const, label: messages.value.codex.filters.pantheon, icon: "👑" },
  { key: "sanctuary" as const, label: messages.value.codex.filters.sanctuary, icon: "🏛️" },
]);

function getCategoryColor(cat: string): string {
  const kind = classifyEntityCategory(cat);
  if (kind === "artifact") return "var(--accent-purple)";
  if (kind === "pantheon") return "var(--accent-gold)";
  if (kind === "sanctuary") return "var(--accent-cyan)";
  return "var(--accent-indigo)";
}

const allCodexEntities = computed(() => {
  return stories.filter((item) => isCodexEntityCategory(item.category));
});

const impactMap = computed(() => {
  const map = new Map<string, EntityImpactMetrics>();
  const normalizedScenes = stories.map((s) => ({
    id: s.id,
    lower: s.content.toLowerCase(),
  }));

  for (const entity of allCodexEntities.value) {
    const titleLower = entity.title.trim().toLowerCase();
    let mentions = 0;
    if (titleLower.length >= 2) {
      for (const sc of normalizedScenes) {
        if (sc.id !== entity.id && sc.lower.includes(titleLower)) {
          mentions++;
        }
      }
    }

    let links = 0;
    for (const rel of projectRelations.value) {
      if (rel.sourceId === entity.id || rel.targetId === entity.id) {
        links++;
      }
    }

    map.set(entity.id, {
      mentions,
      links,
      score: mentions + links * 2,
    });
  }
  return map;
});

const maxImpactScore = computed(() => {
  let max = 1;
  for (const m of impactMap.value.values()) {
    if (m.score > max) max = m.score;
  }
  return max;
});

function getEntityMetrics(id: string): EntityImpactMetrics {
  return impactMap.value.get(id) ?? { mentions: 0, links: 0, score: 0 };
}

function getImpactPercent(id: string): number {
  const score = getEntityMetrics(id).score;
  if (score === 0) return 0;
  return Math.max(12, Math.round((score / maxImpactScore.value) * 100));
}

const filteredEntities = computed(() => {
  const q = searchQuery.value.toLowerCase().trim();
  let list = allCodexEntities.value.filter((item) => {
    if (activeFilter.value === "ALL") return true;
    return classifyEntityCategory(item.category) === activeFilter.value;
  });

  if (q) {
    list = list.filter(
      (i) => i.title.toLowerCase().includes(q) || i.content.toLowerCase().includes(q)
    );
  }

  const copy = [...list];
  if (sortBy.value === "title") {
    copy.sort((a, b) => a.title.localeCompare(b.title));
  } else if (sortBy.value === "impact") {
    copy.sort((a, b) => getEntityMetrics(b.id).score - getEntityMetrics(a.id).score);
  } else {
    copy.sort((a, b) => b.revision - a.revision);
  }

  return copy;
});

watch(
  [() => projectId, () => stories.length],
  () => {
    loadProjectRelations();
  }
);

watch(
  () => stories,
  (nextStories) => {
    if (selectedEntity.value) {
      const updated = nextStories.find((s) => s.id === selectedEntity.value?.id);
      if (updated) {
        selectedEntity.value = { ...updated };
      }
    }
  },
  { deep: true }
);

function openDossier(item: StoryItem) {
  selectedEntity.value = { ...item };
  vibrateSelection();
}

function handleCreateEntity(draft: StoryItem) {
  showCreatorDrawer.value = false;
  vibrateSuccess();
  emit("select", draft);
}

onMounted(() => {
  loadProjectRelations();
});
</script>

<template>
  <section class="quantum-codex-stage">
    <header class="codex-master-header">
      <div class="header-left">
        <span class="codex-sigil">◈</span>
        <div>
          <h2>{{ t("codex.title") }}</h2>
          <p>{{ t("codex.subtitle") }}</p>
        </div>
      </div>

      <div class="header-actions">
        <button class="btn-primary-creator" @click="showCreatorDrawer = true; vibrateLight()">
          <span>{{ t("codex.createEntityBtn") }}</span>
        </button>
      </div>
    </header>

    <div class="codex-control-panel">
      <div class="taxonomy-rail">
        <button
          v-for="pill in filterPills"
          :key="pill.key"
          :class="['taxonomy-pill', { active: activeFilter === pill.key }]"
          @click="activeFilter = pill.key; vibrateLight()"
        >
          <span class="pill-icon">{{ pill.icon }}</span>
          <span>{{ pill.label }}</span>
        </button>
      </div>

      <div class="filter-controls-group">
        <div class="layout-switch-pills">
          <button
            :class="['btn-layout-mode', { active: viewLayout === 'grid' }]"
            :title="t('codex.viewGrid')"
            @click="viewLayout = 'grid'"
          >
            ⊞
          </button>
          <button
            :class="['btn-layout-mode', { active: viewLayout === 'table' }]"
            :title="t('codex.viewTable')"
            @click="viewLayout = 'table'"
          >
            ☰
          </button>
        </div>

        <select v-model="sortBy" class="select-sort">
          <option value="impact">{{ t("codex.sort.impact") }}</option>
          <option value="title">{{ t("codex.sort.title") }}</option>
          <option value="revision">{{ t("codex.sort.revision") }}</option>
        </select>

        <div class="search-input-capsule">
          <span>⌕</span>
          <input v-model="searchQuery" :placeholder="t('codex.searchPlaceholder')" />
        </div>
      </div>
    </div>

    <div v-if="viewLayout === 'grid'" class="codex-entities-grid">
      <div
        v-for="item in filteredEntities"
        :key="item.id"
        class="codex-holo-card"
        :style="{ '--category-accent': getCategoryColor(item.category) }"
        @click="openDossier(item)"
      >
        <div class="card-head-strip">
          <span class="card-category-tag">{{ item.category }}</span>
          <span class="card-metrics-pills">
            {{ t("codex.mentionsCount", { count: getEntityMetrics(item.id).mentions }) }} •
            {{ t("codex.linksCount", { count: getEntityMetrics(item.id).links }) }}
          </span>
        </div>

        <h3 class="card-entity-title">{{ item.title }}</h3>
        <div
          class="card-markdown-preview markdown-rendered"
          v-html="renderMarkdown(item.content.slice(0, 150))"
        ></div>

        <div class="card-footer-dock">
          <div class="resonance-bar-track">
            <div
              class="resonance-bar-fill"
              :style="{ width: `${getImpactPercent(item.id)}%` }"
            ></div>
          </div>
          <span class="card-inspect-hint">{{ t("codex.inspectHint") }}</span>
        </div>
      </div>

      <div v-if="filteredEntities.length === 0" class="codex-empty-card">
        <span class="empty-astral-sigil">✦</span>
        <h3>{{ t("codex.emptyTitle") }}</h3>
        <p>{{ t("codex.emptySubtitle") }}</p>
        <button class="btn-create-inline" @click="showCreatorDrawer = true">
          {{ t("codex.emptyAction") }}
        </button>
      </div>
    </div>

    <div v-else class="codex-table-view">
      <table class="codex-table">
        <thead>
          <tr>
            <th>{{ t("codex.table.entity") }}</th>
            <th>{{ t("codex.table.category") }}</th>
            <th>{{ t("codex.table.impact") }}</th>
            <th>{{ t("codex.table.revision") }}</th>
            <th>{{ t("codex.table.actions") }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="item in filteredEntities" :key="item.id" @click="openDossier(item)">
            <td class="col-name"><b>{{ item.title }}</b></td>
            <td><span class="table-cat-badge">{{ item.category }}</span></td>
            <td>
              <span class="table-res-mono">
                {{ getEntityMetrics(item.id).mentions }} / {{ getEntityMetrics(item.id).links }}
              </span>
            </td>
            <td>r{{ item.revision }}</td>
            <td>
              <button class="btn-table-open" @click.stop="openDossier(item)">
                {{ t("codex.table.openDossier") }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <CodexCreatorDrawer
      :is-open="showCreatorDrawer"
      :project-id="projectId"
      @close="showCreatorDrawer = false"
      @create="handleCreateEntity"
    />

    <CodexDossierModal
      :entity="selectedEntity"
      :project-id="projectId"
      :stories="stories"
      :operator="operator"
      @close="selectedEntity = null"
      @open-in-studio="emit('select', $event); selectedEntity = null"
      @save-entity="emit('saveEntity', $event)"
      @relations-changed="loadProjectRelations"
      @notify="(msg, kind) => emit('notify', msg, kind)"
    />
  </section>
</template>

<style scoped>
.quantum-codex-stage {
  flex: 1;
  overflow-y: auto;
  padding: 24px max(24px, var(--safe-right)) calc(var(--safe-bottom) + 32px) max(24px, var(--safe-left));
  max-width: 1400px;
  margin: 0 auto;
  width: 100%;
  -webkit-overflow-scrolling: touch;
}

.codex-master-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  gap: 14px;
  flex-wrap: wrap;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.codex-sigil {
  font-size: 24px;
  color: var(--accent-cyan);
}

.header-left h2 {
  font-size: clamp(20px, 4vw, 24px);
  font-weight: 900;
  color: var(--text-pure);
}

.header-left p {
  font-size: 13px;
  color: var(--text-muted);
}

.btn-primary-creator {
  background: linear-gradient(135deg, var(--accent-indigo) 0%, #4338ca 100%);
  color: #fff;
  border: none;
  border-radius: 10px;
  min-height: 40px;
  padding: 10px 18px;
  font-size: 13px;
  font-weight: 800;
  cursor: pointer;
}

.codex-control-panel {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.taxonomy-rail {
  display: flex;
  gap: 6px;
  overflow-x: auto;
  max-width: 100%;
  padding-bottom: 2px;
}

.taxonomy-pill {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-muted);
  border-radius: 999px;
  min-height: 36px;
  padding: 6px 14px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
}

.taxonomy-pill.active {
  background: var(--accent-indigo);
  color: #fff;
  border-color: var(--accent-indigo);
}

.filter-controls-group {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  flex: 1;
  justify-content: flex-end;
}

.layout-switch-pills {
  display: flex;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 2px;
}

.btn-layout-mode {
  background: transparent;
  border: none;
  color: var(--text-muted);
  width: 32px;
  height: 32px;
  border-radius: 6px;
  cursor: pointer;
}

.btn-layout-mode.active {
  background: var(--accent-indigo);
  color: #fff;
}

.select-sort {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-pure);
  min-height: 38px;
  padding: 6px 10px;
  border-radius: 8px;
  font-size: 12px;
  outline: none;
}

.search-input-capsule {
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  min-height: 38px;
  padding: 6px 12px;
  min-width: 150px;
}

.search-input-capsule input {
  background: transparent;
  border: none;
  color: var(--text-pure);
  font-size: 12px;
  outline: none;
  width: 100%;
}

.codex-entities-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(100%, 290px), 1fr));
  gap: 16px;
}

.codex-holo-card {
  position: relative;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
  overflow: hidden;
}

.codex-holo-card:hover {
  border-color: var(--category-accent);
  transform: translateY(-2px);
}

.card-head-strip {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}

.card-category-tag {
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--category-accent);
}

.card-metrics-pills {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 700;
  color: var(--text-dim);
}

.card-entity-title {
  font-size: 16px;
  font-weight: 800;
  color: var(--text-pure);
}

.card-markdown-preview {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.55;
  max-height: 60px;
  overflow: hidden;
}

.card-footer-dock {
  display: flex;
  align-items: center;
  gap: 12px;
  padding-top: 8px;
  border-top: 1px solid var(--border-subtle);
}

.resonance-bar-track {
  flex: 1;
  height: 4px;
  background: var(--border-subtle);
  border-radius: 999px;
  overflow: hidden;
}

.resonance-bar-fill {
  height: 100%;
  background: var(--category-accent);
}

.card-inspect-hint {
  font-size: 10px;
  font-weight: 700;
  color: var(--accent-cyan);
}

.codex-empty-card {
  grid-column: 1 / -1;
  text-align: center;
  padding: 50px 18px;
  background: var(--bg-card);
  border: 1px dashed var(--border);
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.empty-astral-sigil {
  font-size: 28px;
  color: var(--accent-cyan);
}

.btn-create-inline {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  border-radius: 8px;
  min-height: 40px;
  padding: 8px 16px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.codex-table-view {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 14px;
  overflow-x: auto;
}

.codex-table {
  width: 100%;
  border-collapse: collapse;
  min-width: 520px;
}

.codex-table th,
.codex-table td {
  padding: 12px 16px;
  text-align: left;
  border-bottom: 1px solid var(--border-subtle);
  font-size: 13px;
}

.codex-table th {
  background: var(--bg-surface-solid);
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--text-dim);
}

.table-cat-badge {
  font-size: 10px;
  font-weight: 700;
  background: var(--border-subtle);
  padding: 2px 8px;
  border-radius: 4px;
  color: var(--accent-cyan);
}

.table-res-mono {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--accent-gold);
}

.btn-table-open {
  background: transparent;
  border: 1px solid var(--border);
  color: var(--accent-cyan);
  border-radius: 6px;
  padding: 5px 10px;
  font-size: 11px;
  cursor: pointer;
}

@media (max-width: 900px) {
  .quantum-codex-stage {
    padding: 14px max(14px, var(--safe-right)) 24px max(14px, var(--safe-left));
  }

  .filter-controls-group {
    justify-content: flex-start;
    width: 100%;
  }

  .search-input-capsule {
    flex: 1;
  }
}
</style>