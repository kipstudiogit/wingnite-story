<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { ref, watch } from "vue";
import type { ProjectItem } from "../../types";
import { useI18n } from "../../i18n";

const { isOpen, editingProject = null } = defineProps<{
  isOpen: boolean;
  editingProject?: ProjectItem | null;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "submit", payload: { id?: string; title: string; description: string }): void;
}>();

const { t } = useI18n();
const title = ref("");
const description = ref("");

watch(
  [() => isOpen, () => editingProject],
  ([open, proj]) => {
    if (open) {
      title.value = proj ? proj.title : "";
      description.value = proj ? proj.description : "";
    }
  }
);

function handleConfirm() {
  if (!title.value.trim()) return;
  emit("submit", {
    id: editingProject?.id,
    title: title.value.trim(),
    description: description.value.trim(),
  });
  title.value = "";
  description.value = "";
}
</script>

<template>
  <div v-if="isOpen" class="modal-backdrop" @click.self="emit('close')">
    <div class="modal-window">
      <h3>{{ editingProject ? t("projects.editTitle") : t("projects.createTitle") }}</h3>
      <p class="modal-subtext">{{ t("projects.createSubtitle") }}</p>

      <div class="form-inputs">
        <input
          v-model="title"
          class="modal-input"
          :placeholder="t('projects.titlePlaceholder')"
          autofocus
          @keydown.enter="handleConfirm"
        />
        <textarea
          v-model="description"
          class="modal-textarea"
          :placeholder="t('projects.descPlaceholder')"
        ></textarea>
      </div>

      <div class="modal-actions">
        <button class="btn-modal-cancel" @click="emit('close')">{{ t("common.cancel") }}</button>
        <button class="btn-modal-submit" @click="handleConfirm">
          {{ editingProject ? t("projects.editAction") : t("projects.createAction") }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(14px);
  z-index: 250;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-window {
  width: 90%;
  max-width: 480px;
  background: var(--bg-surface-solid);
  border: 1px solid var(--border-strong);
  border-radius: 16px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.9);
}

.modal-window h3 {
  font-size: 18px;
  font-weight: 800;
  color: var(--text-pure);
}

.modal-subtext {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
}

.form-inputs {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.modal-input,
.modal-textarea {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-pure);
  border-radius: 10px;
  padding: 10px 14px;
  font-size: 13px;
  outline: none;
}

.modal-textarea {
  resize: vertical;
  min-height: 80px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 6px;
}

.btn-modal-cancel {
  background: transparent;
  border: 1px solid var(--border);
  color: var(--text-muted);
  padding: 8px 16px;
  border-radius: 10px;
  cursor: pointer;
}

.btn-modal-submit {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  padding: 8px 20px;
  border-radius: 10px;
  font-weight: 700;
  cursor: pointer;
}
</style>