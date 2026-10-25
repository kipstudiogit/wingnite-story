<script setup lang="ts">
import { ref, computed } from "vue";
import { renderMarkdown } from "../utils/markdown";
import type { EntityRelationItem, StoryItem } from "../types";
import { useHaptics } from "../composables/useHaptics";
import { invoke } from "@tauri-apps/api/core";

const { stories } = defineProps<{
  stories: StoryItem[];
}>();

const emit = defineEmits<{
  (e: "select", story: StoryItem): void;
  (e: "createEntity", type: "artifact" | "pantheon" | "sanctuary" | "character"): void;
  (e: "saveEntity", story: StoryItem): void;
}>();

const { vibrateLight, vibrateSelection, vibrateSuccess } = useHaptics();

const activeFilter = ref<string>("ALL");
const searchQuery = ref<string>("");
const sortBy = ref<"title" | "resonance" | "revision">("resonance");
const viewLayout = ref<"grid" | "table">("grid");

const showCreatorDrawer = ref(false);
const newEntityName = ref("");
const newEntityType = ref<"character" | "artifact" | "pantheon" | "sanctuary">("character");
const newEntityArchetype = ref("Искатель");
const newEntityStatus = ref("Активен");
const newEntityDesc = ref("");

const selectedEntity = ref<StoryItem | null>(null);
const activeModalTab = ref<"dossier" | "relations" | "chronicle" | "editor">("dossier");
const relationsList = ref<EntityRelationItem[]>([]);
const newRelTargetId = ref("");
const newRelType = ref("Союзник");
const newRelDesc = ref("");

const categoriesMap: Record<string, { label: string; icon: string }> = {
  ALL: { label: "Все карточки", icon: "❖" },
  character: { label: "Персонажи", icon: "👤" },
  artifact: { label: "Артефакты", icon: "🔮" },
  pantheon: { label: "Демиурги", icon: "👑" },
  sanctuary: { label: "Святилища", icon: "🏛️" },
};

function getCategoryColor(cat: string) {
  if (cat.includes("Артефакт") || cat.includes("Реликвия")) return "var(--accent-purple)";
  if (cat.includes("Демиург") || cat.includes("Сущность")) return "var(--accent-gold)";
  if (cat.includes("Святилище") || cat.includes("Локация")) return "var(--accent-cyan)";
  return "var(--accent-indigo)";
}

function calculateResonance(id: string, text: string): number {
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = (hash << 5) - hash + id.charCodeAt(i);
    hash |= 0;
  }
  return 70 + (Math.abs(hash + text.length) % 30);
}

const allCodexEntities = computed(() => {
  return stories.filter((item) => {
    return (
      item.category.includes("Персонаж") ||
      item.category.includes("Артефакт") ||
      item.category.includes("Демиург") ||
      item.category.includes("Святилище") ||
      item.category.includes("Локация") ||
      item.category.includes("Реликвия")
    );
  });
});

const filteredEntities = computed(() => {
  const q = searchQuery.value.toLowerCase().trim();
  let list = allCodexEntities.value.filter((item) => {
    if (activeFilter.value === "character") return item.category.includes("Персонаж");
    if (activeFilter.value === "artifact") return item.category.includes("Артефакт") || item.category.includes("Реликвия");
    if (activeFilter.value === "pantheon") return item.category.includes("Демиург") || item.category.includes("Сущность");
    if (activeFilter.value === "sanctuary") return item.category.includes("Святилище") || item.category.includes("Локация");
    return true;
  });

  if (q) {
    list = list.filter((i) => i.title.toLowerCase().includes(q) || i.content.toLowerCase().includes(q));
  }

  if (sortBy.value === "title") {
    list.sort((a, b) => a.title.localeCompare(b.title));
  } else if (sortBy.value === "resonance") {
    list.sort((a, b) => calculateResonance(b.id, b.content) - calculateResonance(a.id, a.content));
  } else {
    list.sort((a, b) => b.revision - a.revision);
  }

  return list;
});

async function openDossier(item: StoryItem) {
  selectedEntity.value = { ...item };
  activeModalTab.value = "dossier";
  vibrateSelection();
  await loadEntityRelations(item.id);
}

async function loadEntityRelations(entityId: string) {
  try {
    const all = await invoke<EntityRelationItem[]>("fetch_entity_relations");
    relationsList.value = all.filter((r) => r.sourceId === entityId || r.targetId === entityId);
  } catch {
    relationsList.value = [];
  }
}

async function handleAddRelation() {
  if (!selectedEntity.value || !newRelTargetId.value) return;
  const payload: EntityRelationItem = {
    id: "",
    sourceId: selectedEntity.value.id,
    targetId: newRelTargetId.value,
    relationType: newRelType.value,
    description: newRelDesc.value.trim() || "Связь из Кодекса",
  };
  try {
    await invoke("mutate_entity_relation", { relation: payload, operator: "Codex-Custodian" });
    vibrateSuccess();
    newRelDesc.value = "";
    await loadEntityRelations(selectedEntity.value.id);
  } catch {}
}

function handleSaveInlineEdit() {
  if (!selectedEntity.value) return;
  emit("saveEntity", selectedEntity.value);
  vibrateSuccess();
}

function handleCreateFromForm() {
  if (!newEntityName.value.trim()) return;
  let categoryName = "Персонаж";
  if (newEntityType.value === "artifact") categoryName = "Божественный Артефакт";
  if (newEntityType.value === "pantheon") categoryName = "Демиург / Сущность";
  if (newEntityType.value === "sanctuary") categoryName = "Святилище Сил";

  const template = `### Досье: ${newEntityName.value}\n* **Класс / Архетип:** ${newEntityArchetype.value}\n* **Статус:** ${newEntityStatus.value}\n\n${newEntityDesc.value || 'Описание свойств и истории...'}`;

  const dummyItem: StoryItem = {
    id: "",
    projectId: "",
    title: newEntityName.value,
    content: template,
    category: categoryName,
    revision: 1,
    sortOrder: 0,
    contentHash: "",
    updatedAt: 0,
  };

  emit("select", dummyItem);
  showCreatorDrawer.value = false;
  newEntityName.value = "";
  newEntityDesc.value = "";
  vibrateSuccess();
}
</script>

<template>
  <section class="quantum-codex-stage">
    <!-- Шапка реестра -->
    <header class="codex-master-header">
      <div class="header-left">
        <span class="codex-sigil">◈</span>
        <div>
          <h2>Кодекс Мироздания</h2>
          <p>Архив персонажей, реликвий, демиургов и онтологических законов</p>
        </div>
      </div>

      <div class="header-actions">
        <button class="btn-primary-creator" @click="showCreatorDrawer = true; vibrateLight()">
          <span>+ Создать сущность</span>
        </button>
      </div>
    </header>

    <!-- Панель инструментов и таксономия -->
    <div class="codex-control-panel">
      <div class="taxonomy-rail">
        <button
          v-for="(meta, key) in categoriesMap"
          :key="key"
          :class="['taxonomy-pill', { active: activeFilter === key }]"
          @click="activeFilter = key; vibrateLight()"
        >
          <span class="pill-icon">{{ meta.icon }}</span>
          <span>{{ meta.label }}</span>
        </button>
      </div>

      <div class="filter-controls-group">
        <div class="layout-switch-pills">
          <button :class="['btn-layout-mode', { active: viewLayout === 'grid' }]" title="Сетка" @click="viewLayout = 'grid'">⊞</button>
          <button :class="['btn-layout-mode', { active: viewLayout === 'table' }]" title="Таблица" @click="viewLayout = 'table'">☰</button>
        </div>

        <select v-model="sortBy" class="select-sort">
          <option value="resonance">По резонансу</option>
          <option value="title">По алфавиту</option>
          <option value="revision">По ревизиям</option>
        </select>

        <div class="search-input-capsule">
          <span>⌕</span>
          <input v-model="searchQuery" placeholder="Быстрый фильтр реестра..." />
        </div>
      </div>
    </div>

    <!-- Вид 1: Сетка голографических карточек -->
    <div v-if="viewLayout === 'grid'" class="codex-entities-grid">
      <div
        v-for="item in filteredEntities"
        :key="item.id"
        class="codex-holo-card"
        :style="{ '--category-accent': getCategoryColor(item.category) }"
        @click="openDossier(item)"
      >
        <div class="card-glass-lead"></div>
        <div class="card-head-strip">
          <span class="card-category-tag">{{ item.category }}</span>
          <span class="card-resonance-digit">{{ calculateResonance(item.id, item.content) }}%</span>
        </div>

        <h3 class="card-entity-title">{{ item.title }}</h3>
        <div class="card-markdown-preview markdown-rendered" v-html="renderMarkdown(item.content.slice(0, 150))"></div>

        <div class="card-footer-dock">
          <div class="resonance-bar-track">
            <div class="resonance-bar-fill" :style="{ width: `${calculateResonance(item.id, item.content)}%` }"></div>
          </div>
          <span class="card-inspect-hint">Открыть досье →</span>
        </div>
      </div>

      <div v-if="filteredEntities.length === 0" class="codex-empty-card">
        <span class="empty-astral-sigil">✦</span>
        <h3>В реестре пока нет сущностей этой категории</h3>
        <p>Создайте персонажа, божественный артефакт или локацию, чтобы оживить канон</p>
        <button class="btn-create-inline" @click="showCreatorDrawer = true">+ Добавить в реестр</button>
      </div>
    </div>

    <!-- Вид 2: Табличный обзор -->
    <div v-else class="codex-table-view">
      <table class="codex-table">
        <thead>
          <tr>
            <th>Сущность</th>
            <th>Категория</th>
            <th>Резонанс</th>
            <th>Ревизия</th>
            <th>Действия</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="item in filteredEntities" :key="item.id" @click="openDossier(item)">
            <td class="col-name"><b>{{ item.title }}</b></td>
            <td><span class="table-cat-badge">{{ item.category }}</span></td>
            <td><span class="table-res-mono">{{ calculateResonance(item.id, item.content) }}%</span></td>
            <td>r{{ item.revision }}</td>
            <td><button class="btn-table-open" @click.stop="openDossier(item)">Досье ↗</button></td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Боковая панель создания сущности -->
    <div v-if="showCreatorDrawer" class="creator-drawer-backdrop" @click.self="showCreatorDrawer = false">
      <aside class="creator-drawer-surface">
        <header class="drawer-header">
          <h3>Мастерская сущности</h3>
          <button class="btn-drawer-close" @click="showCreatorDrawer = false">✕</button>
        </header>

        <div class="creator-fields-form">
          <div class="field-item">
            <label>Имя или титул сущности</label>
            <input v-model="newEntityName" class="modal-input" placeholder="Напр. Элион, Клинок Зари..." />
          </div>

          <div class="field-item">
            <label>Онтологическая категория</label>
            <select v-model="newEntityType" class="modal-input">
              <option value="character">Персонаж / Искатель</option>
              <option value="artifact">Божественный Артефакт / Реликвия</option>
              <option value="pantheon">Демиург / Высшая Сущность</option>
              <option value="sanctuary">Святилище Сил / Локация</option>
            </select>
          </div>

          <div class="field-row">
            <div class="field-item">
              <label>Архетип / Класс</label>
              <input v-model="newEntityArchetype" class="modal-input" />
            </div>
            <div class="field-item">
              <label>Статус</label>
              <input v-model="newEntityStatus" class="modal-input" />
            </div>
          </div>

          <div class="field-item">
            <label>Описание и ключевые свойства</label>
            <textarea v-model="newEntityDesc" class="modal-textarea" placeholder="Свойства, цели, уязвимости, история обретения..."></textarea>
          </div>
        </div>

        <div class="drawer-footer-row">
          <button class="btn-submit-entity" @click="handleCreateFromForm">Материализовать в каноне</button>
        </div>
      </aside>
    </div>

    <!-- Модальный инспектор досье -->
    <div v-if="selectedEntity" class="dossier-modal-backdrop" @click.self="selectedEntity = null">
      <div class="dossier-modal-window">
        <header class="modal-head">
          <div class="modal-title-left">
            <span class="modal-badge">{{ selectedEntity.category }}</span>
            <h2>{{ selectedEntity.title }}</h2>
          </div>
          <div class="modal-head-buttons">
            <button class="btn-jump-studio" @click="emit('select', selectedEntity)">Открыть в Студии ↗</button>
            <button class="btn-modal-close" @click="selectedEntity = null">✕</button>
          </div>
        </header>

        <nav class="dossier-nav-pills">
          <button :class="['pill-tab', { active: activeModalTab === 'dossier' }]" @click="activeModalTab = 'dossier'">Досье</button>
          <button :class="['pill-tab', { active: activeModalTab === 'relations' }]" @click="activeModalTab = 'relations'">Связи ({{ relationsList.length }})</button>
          <button :class="['pill-tab', { active: activeModalTab === 'editor' }]" @click="activeModalTab = 'editor'">Быстрая правка</button>
        </nav>

        <div class="modal-scroll-area">
          <div v-if="activeModalTab === 'dossier'" class="dossier-text-view markdown-rendered" v-html="renderMarkdown(selectedEntity.content)"></div>

          <div v-else-if="activeModalTab === 'relations'" class="relations-view">
            <div class="add-relation-box">
              <h4>Добавить связь в реляционный граф</h4>
              <div class="add-rel-form">
                <select v-model="newRelTargetId" class="modal-input">
                  <option value="">Выберите целевую сущность...</option>
                  <option v-for="s in stories.filter((st) => st.id !== selectedEntity?.id)" :key="s.id" :value="s.id">
                    {{ s.title }} ({{ s.category }})
                  </option>
                </select>
                <select v-model="newRelType" class="modal-input">
                  <option value="Союзник">Союзник</option>
                  <option value="Вражда">Вражда</option>
                  <option value="Владение">Владение</option>
                  <option value="Иерархия">Иерархия</option>
                </select>
                <input v-model="newRelDesc" class="modal-input" placeholder="Описание связи..." />
                <button class="btn-add-rel-action" @click="handleAddRelation">Связать</button>
              </div>
            </div>

            <div class="relations-feed">
              <div v-for="rel in relationsList" :key="rel.id" class="rel-feed-item">
                <span class="rel-badge">{{ rel.relationType }}</span>
                <span class="rel-text">{{ rel.description }}</span>
              </div>
              <div v-if="relationsList.length === 0" class="rel-empty-msg">Связей с другими сущностями пока не зафиксировано</div>
            </div>
          </div>

          <div v-else-if="activeModalTab === 'editor'" class="fast-editor-view">
            <textarea v-model="selectedEntity.content" class="fast-textarea"></textarea>
            <div class="fast-editor-footer">
              <button class="btn-fast-save" @click="handleSaveInlineEdit">Сохранить изменения в канон</button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.quantum-codex-stage {
  flex: 1;
  overflow-y: auto;
  padding: 30px 36px calc(var(--safe-bottom) + 80px) 36px;
  max-width: 1400px;
  margin: 0 auto;
  width: 100%;
}

.codex-master-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  gap: 16px;
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
  font-size: 24px;
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
  padding: 10px 18px;
  font-size: 13px;
  font-weight: 800;
  cursor: pointer;
}

.codex-control-panel {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  margin-bottom: 24px;
  flex-wrap: wrap;
}

.taxonomy-rail {
  display: flex;
  gap: 6px;
  overflow-x: auto;
}

.taxonomy-pill {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-muted);
  border-radius: 999px;
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
  width: 28px;
  height: 28px;
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
  padding: 6px 12px;
}

.search-input-capsule input {
  background: transparent;
  border: none;
  color: var(--text-pure);
  font-size: 12px;
  outline: none;
  width: 170px;
}

.codex-entities-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 20px;
}

.codex-holo-card {
  position: relative;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 20px;
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
}

.card-category-tag {
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--category-accent);
}

.card-resonance-digit {
  font-family: var(--font-mono);
  font-size: 11px;
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
  padding: 60px 20px;
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
  padding: 8px 16px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

/* Табличный вид */
.codex-table-view {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 14px;
  overflow: hidden;
}

.codex-table {
  width: 100%;
  border-collapse: collapse;
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
  padding: 4px 10px;
  font-size: 11px;
  cursor: pointer;
}

/* Модальный инспектор досье */
.dossier-modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(14px);
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
}

.dossier-modal-window {
  width: 90%;
  max-width: 800px;
  max-height: 85vh;
  background: var(--bg-surface-solid);
  border: 1px solid var(--border-strong);
  border-radius: 20px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.modal-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-badge {
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--accent-cyan);
}

.modal-title-left h2 {
  font-size: 22px;
  font-weight: 900;
  color: var(--text-pure);
}

.modal-head-buttons {
  display: flex;
  gap: 8px;
}

.btn-jump-studio {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--accent-cyan);
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}

.btn-modal-close {
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 16px;
  cursor: pointer;
}

.dossier-nav-pills {
  display: flex;
  gap: 6px;
  border-bottom: 1px solid var(--border);
  padding-bottom: 8px;
}

.pill-tab {
  background: transparent;
  border: none;
  color: var(--text-muted);
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.pill-tab.active {
  background: var(--accent-indigo);
  color: #fff;
}

.modal-scroll-area {
  flex: 1;
  overflow-y: auto;
  max-height: 55vh;
}

.add-relation-box {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 12px;
  margin-bottom: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.add-rel-form {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.modal-input {
  background: var(--bg-input);
  border: 1px solid var(--border);
  color: var(--text-pure);
  padding: 6px 10px;
  border-radius: 8px;
  font-size: 11px;
  outline: none;
}

.btn-add-rel-action {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  border-radius: 8px;
  padding: 6px 14px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}

.relations-feed {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.rel-feed-item {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px 12px;
  display: flex;
  align-items: center;
  gap: 10px;
}

.rel-badge {
  font-size: 10px;
  font-weight: 800;
  color: var(--accent-cyan);
  background: var(--border-subtle);
  padding: 2px 6px;
  border-radius: 4px;
}

.rel-text {
  font-size: 12px;
  color: var(--text-main);
}

.fast-editor-view {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.fast-textarea {
  width: 100%;
  height: 280px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-main);
  padding: 12px;
  border-radius: 10px;
  font-family: var(--font-mono);
  font-size: 13px;
  outline: none;
  resize: vertical;
}

.btn-fast-save {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  border-radius: 8px;
  padding: 8px 16px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

/* Боковая панель мастера создания */
.creator-drawer-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  z-index: 220;
  display: flex;
  justify-content: flex-end;
}

.creator-drawer-surface {
  width: 100%;
  max-width: 440px;
  height: 100%;
  background: var(--bg-surface-solid);
  border-left: 1px solid var(--border);
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  box-shadow: -10px 0 40px rgba(0, 0, 0, 0.9);
}

.drawer-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.creator-fields-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
  flex: 1;
  overflow-y: auto;
}

.field-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.field-item label {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-dim);
  text-transform: uppercase;
}

.field-row {
  display: flex;
  gap: 10px;
}

.field-row .field-item {
  flex: 1;
}

.modal-textarea {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-pure);
  border-radius: 8px;
  padding: 10px;
  font-size: 12px;
  min-height: 120px;
  outline: none;
}

.btn-submit-entity {
  width: 100%;
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  border-radius: 10px;
  padding: 12px;
  font-size: 13px;
  font-weight: 800;
  cursor: pointer;
}
</style>