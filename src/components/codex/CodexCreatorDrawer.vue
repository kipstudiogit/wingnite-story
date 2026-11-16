<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { ref, watch } from "vue";
import type { EntityKind, StoryItem } from "../../types";
import { useI18n } from "../../i18n";

const { isOpen, projectId } = defineProps<{
  isOpen: boolean;
  projectId: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "create", entity: StoryItem): void;
}>();

const { t, messages } = useI18n();

const entityName = ref("");
const entityType = ref<EntityKind>("character");
const entityArchetype = ref(t("codex.creator.archetypeDefault"));
const entityStatus = ref(t("codex.creator.statusDefault"));
const entityDesc = ref("");

watch(
  () => isOpen,
  (open) => {
    if (open) {
      entityArchetype.value = t("codex.creator.archetypeDefault");
      entityStatus.value = t("codex.creator.statusDefault");
    }
  }
);

function handleCreate() {
  if (!entityName.value.trim()) return;

  const cats = messages.value.codex.creator.categories;
  const tpl = messages.value.codex.creator.templates;
  const categoryName = cats[entityType.value];

  const cleanName = entityName.value.trim();
  const body = entityDesc.value.trim() || tpl.fallbackBody;
  const content = `### ${tpl.dossierHeading}: ${cleanName}\n* **${tpl.archetypeField}:** ${entityArchetype.value}\n* **${tpl.statusField}:** ${entityStatus.value}\n\n${body}`;

  const draft: StoryItem = {
    id: "",
    projectId,
    title: cleanName,
    content,
    category: categoryName,
    revision: 1,
    sortOrder: 0,
    contentHash: "",
    updatedAt: 0,
  };

  emit("create", draft);
  entityName.value = "";
  entityDesc.value = "";
}
</script>

<template>
  <div v-if="isOpen" class="creator-drawer-backdrop" @click.self="emit('close')">
    <aside class="creator-drawer-surface">
      <header class="drawer-header">
        <h3>{{ t("codex.creator.title") }}</h3>
        <button class="btn-drawer-close" @click="emit('close')">✕</button>
      </header>

      <div class="creator-fields-form">
        <div class="field-item">
          <label>{{ t("codex.creator.nameLabel") }}</label>
          <input
            v-model="entityName"
            class="modal-input"
            :placeholder="t('codex.creator.namePlaceholder')"
          />
        </div>

        <div class="field-item">
          <label>{{ t("codex.creator.categoryLabel") }}</label>
          <select v-model="entityType" class="modal-input">
            <option value="character">{{ t("codex.creator.categories.character") }}</option>
            <option value="artifact">{{ t("codex.creator.categories.artifact") }}</option>
            <option value="pantheon">{{ t("codex.creator.categories.pantheon") }}</option>
            <option value="sanctuary">{{ t("codex.creator.categories.sanctuary") }}</option>
          </select>
        </div>

        <div class="field-row">
          <div class="field-item">
            <label>{{ t("codex.creator.archetypeLabel") }}</label>
            <input v-model="entityArchetype" class="modal-input" />
          </div>
          <div class="field-item">
            <label>{{ t("codex.creator.statusLabel") }}</label>
            <input v-model="entityStatus" class="modal-input" />
          </div>
        </div>

        <div class="field-item">
          <label>{{ t("codex.creator.descLabel") }}</label>
          <textarea
            v-model="entityDesc"
            class="modal-textarea"
            :placeholder="t('codex.creator.descPlaceholder')"
          ></textarea>
        </div>
      </div>

      <div class="drawer-footer-row">
        <button class="btn-submit-entity" @click="handleCreate">
          {{ t("codex.creator.submitBtn") }}
        </button>
      </div>
    </aside>
  </div>
</template>

<style scoped>
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

.btn-drawer-close {
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 16px;
  cursor: pointer;
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

.modal-input {
  background: var(--bg-input);
  border: 1px solid var(--border);
  color: var(--text-pure);
  padding: 8px 12px;
  border-radius: 8px;
  font-size: 12px;
  outline: none;
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