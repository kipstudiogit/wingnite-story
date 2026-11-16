<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { renderMarkdown } from "../../utils/markdown";
import { useHaptics } from "../../composables/useHaptics";
import { useI18n } from "../../i18n";
import type { EntityRelationItem, StoryItem } from "../../types";

const { entity, projectId, stories, operator = "Codex-Custodian" } = defineProps<{
  entity: StoryItem | null;
  projectId: string;
  stories: StoryItem[];
  operator?: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "openInStudio", entity: StoryItem): void;
  (e: "saveEntity", entity: StoryItem): void;
  (e: "relationsChanged"): void;
  (e: "notify", msg: string, kind: "success" | "error"): void;
}>();

const { vibrateSuccess } = useHaptics();
const { t } = useI18n();

const activeTab = ref<"dossier" | "relations" | "editor">("dossier");
const relationsList = ref<EntityRelationItem[]>([]);
const editableContent = ref("");
const newRelTargetId = ref("");
const newRelType = ref(t("codex.dossier.relationTypes.ally"));
const newRelDesc = ref("");

async function loadEntityRelations(entityId: string) {
  try {
    const all = await invoke<EntityRelationItem[]>("fetch_entity_relations", {
      projectId,
    });
    relationsList.value = all.filter(
      (r) => r.sourceId === entityId || r.targetId === entityId
    );
  } catch {
    relationsList.value = [];
  }
}

watch(
  () => entity,
  (next) => {
    if (next) {
      activeTab.value = "dossier";
      editableContent.value = next.content;
      newRelTargetId.value = "";
      newRelDesc.value = "";
      newRelType.value = t("codex.dossier.relationTypes.ally");
      loadEntityRelations(next.id);
    } else {
      relationsList.value = [];
    }
  },
  { immediate: true }
);

function getPeerTitle(rel: EntityRelationItem): string {
  if (!entity) return "";
  const peerId = rel.sourceId === entity.id ? rel.targetId : rel.sourceId;
  const peer = stories.find((s) => s.id === peerId);
  return peer ? peer.title : peerId;
}

async function handleAddRelation() {
  if (!entity || !newRelTargetId.value) return;
  const payload: EntityRelationItem = {
    id: "",
    sourceId: entity.id,
    targetId: newRelTargetId.value,
    relationType: newRelType.value,
    description: newRelDesc.value.trim() || t("codex.dossier.defaultRelationDesc"),
  };
  try {
    await invoke("mutate_entity_relation", {
      relation: payload,
      operator,
    });
    vibrateSuccess();
    newRelTargetId.value = "";
    newRelDesc.value = "";
    await loadEntityRelations(entity.id);
    emit("relationsChanged");
  } catch {
  }
}

async function handleDeleteRelation(relationId: string) {
  if (!entity) return;
  try {
    await invoke("delete_entity_relation", {
      id: relationId,
      operator,
    });
    vibrateSuccess();
    await loadEntityRelations(entity.id);
    emit("relationsChanged");
    emit("notify", t("notifications.relationDeletedToast"), "success");
  } catch {
  }
}

function handleSaveInlineEdit() {
  if (!entity) return;
  emit("saveEntity", {
    ...entity,
    content: editableContent.value,
  });
  vibrateSuccess();
}
</script>

<template>
  <div v-if="entity" class="dossier-modal-backdrop" @click.self="emit('close')">
    <div class="dossier-modal-window">
      <header class="modal-head">
        <div class="modal-title-left">
          <span class="modal-badge">{{ entity.category }}</span>
          <h2>{{ entity.title }}</h2>
        </div>
        <div class="modal-head-buttons">
          <button class="btn-jump-studio" @click="emit('openInStudio', entity)">
            {{ t("codex.dossier.openInStudio") }}
          </button>
          <button class="btn-modal-close" @click="emit('close')">✕</button>
        </div>
      </header>

      <nav class="dossier-nav-pills">
        <button :class="['pill-tab', { active: activeTab === 'dossier' }]" @click="activeTab = 'dossier'">
          {{ t("codex.dossier.tabDossier") }}
        </button>
        <button :class="['pill-tab', { active: activeTab === 'relations' }]" @click="activeTab = 'relations'">
          {{ t("codex.dossier.tabRelations", { count: relationsList.length }) }}
        </button>
        <button :class="['pill-tab', { active: activeTab === 'editor' }]" @click="activeTab = 'editor'">
          {{ t("codex.dossier.tabQuickEdit") }}
        </button>
      </nav>

      <div class="modal-scroll-area">
        <div
          v-if="activeTab === 'dossier'"
          class="dossier-text-view markdown-rendered"
          v-html="renderMarkdown(editableContent)"
        ></div>

        <div v-else-if="activeTab === 'relations'" class="relations-view">
          <div class="add-relation-box">
            <h4>{{ t("codex.dossier.addRelationTitle") }}</h4>
            <div class="add-rel-form">
              <select v-model="newRelTargetId" class="modal-input">
                <option value="">{{ t("codex.dossier.targetPlaceholder") }}</option>
                <option
                  v-for="s in stories.filter((st) => st.id !== entity.id)"
                  :key="s.id"
                  :value="s.id"
                >
                  {{ s.title }} ({{ s.category }})
                </option>
              </select>
              <select v-model="newRelType" class="modal-input">
                <option :value="t('codex.dossier.relationTypes.ally')">{{ t("codex.dossier.relationTypes.ally") }}</option>
                <option :value="t('codex.dossier.relationTypes.enemy')">{{ t("codex.dossier.relationTypes.enemy") }}</option>
                <option :value="t('codex.dossier.relationTypes.ownership')">{{ t("codex.dossier.relationTypes.ownership") }}</option>
                <option :value="t('codex.dossier.relationTypes.hierarchy')">{{ t("codex.dossier.relationTypes.hierarchy") }}</option>
              </select>
              <input
                v-model="newRelDesc"
                class="modal-input"
                :placeholder="t('codex.dossier.relationDescPlaceholder')"
              />
              <button class="btn-add-rel-action" @click="handleAddRelation">
                {{ t("codex.dossier.linkBtn") }}
              </button>
            </div>
          </div>

          <div class="relations-feed">
            <div v-for="rel in relationsList" :key="rel.id" class="rel-feed-item">
              <span class="rel-badge">{{ rel.relationType }}</span>
              <b class="rel-peer">{{ getPeerTitle(rel) }}</b>
              <span class="rel-text">{{ rel.description }}</span>
              <button
                class="btn-rel-delete"
                :title="t('codex.dossier.deleteRelationTooltip')"
                @click="handleDeleteRelation(rel.id)"
              >
                ✕
              </button>
            </div>
            <div v-if="relationsList.length === 0" class="rel-empty-msg">
              {{ t("codex.dossier.emptyRelations") }}
            </div>
          </div>
        </div>

        <div v-else-if="activeTab === 'editor'" class="fast-editor-view">
          <textarea v-model="editableContent" class="fast-textarea"></textarea>
          <div class="fast-editor-footer">
            <button class="btn-fast-save" @click="handleSaveInlineEdit">
              {{ t("codex.dossier.saveCanonBtn") }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
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
  gap: 12px;
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
  flex-wrap: wrap;
}

.rel-badge {
  font-size: 10px;
  font-weight: 800;
  color: var(--accent-cyan);
  background: var(--border-subtle);
  padding: 2px 6px;
  border-radius: 4px;
}

.rel-peer {
  font-size: 12px;
  color: var(--text-pure);
}

.rel-text {
  font-size: 12px;
  color: var(--text-muted);
  flex: 1;
}

.btn-rel-delete {
  background: transparent;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  padding: 2px 6px;
  font-size: 12px;
}

.btn-rel-delete:hover {
  color: var(--accent-rose);
}

.rel-empty-msg {
  font-size: 12px;
  color: var(--text-dim);
  padding: 16px 4px;
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
</style>