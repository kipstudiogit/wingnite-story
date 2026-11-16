<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { ref, watch } from "vue";
import type { EnterpriseStoryItem, GraphNodePacked } from "../../types";
import { useI18n } from "../../i18n";

const { isOpen, sourceNode, stories } = defineProps<{
  isOpen: boolean;
  sourceNode: GraphNodePacked | null;
  stories: EnterpriseStoryItem[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "submit", payload: { targetId: string; relationType: string; description: string }): void;
}>();

const { t } = useI18n();

const targetId = ref("");
const relationType = ref(t("graph.modal.types.ally"));
const description = ref("");

watch(
  () => isOpen,
  (open) => {
    if (open) {
      targetId.value = "";
      relationType.value = t("graph.modal.types.ally");
      description.value = "";
    }
  }
);

function handleSubmit() {
  if (!targetId.value) return;
  emit("submit", {
    targetId: targetId.value,
    relationType: relationType.value,
    description: description.value.trim(),
  });
}
</script>

<template>
  <div v-if="isOpen" class="modal-backdrop" @click.self="emit('close')">
    <div class="modal-card">
      <h3>{{ t("graph.modal.title") }}</h3>
      <p class="modal-sub">{{ t("graph.modal.nodePrefix") }} <b>{{ sourceNode?.label }}</b></p>

      <div class="modal-form">
        <select v-model="targetId" class="modal-select">
          <option value="">{{ t("graph.modal.targetPlaceholder") }}</option>
          <option
            v-for="s in stories.filter((st) => st.id !== sourceNode?.id)"
            :key="s.id"
            :value="s.id"
          >
            {{ s.title }} ({{ s.category }})
          </option>
        </select>

        <select v-model="relationType" class="modal-select">
          <option :value="t('graph.modal.types.ally')">{{ t("graph.modal.types.ally") }}</option>
          <option :value="t('graph.modal.types.enemy')">{{ t("graph.modal.types.enemy") }}</option>
          <option :value="t('graph.modal.types.ownership')">{{ t("graph.modal.types.ownership") }}</option>
          <option :value="t('graph.modal.types.hierarchy')">{{ t("graph.modal.types.hierarchy") }}</option>
        </select>

        <input
          v-model="description"
          class="modal-input"
          :placeholder="t('graph.modal.descPlaceholder')"
        />
      </div>

      <div class="modal-btn-row">
        <button class="btn-cancel" @click="emit('close')">{{ t("common.cancel") }}</button>
        <button class="btn-submit" @click="handleSubmit">{{ t("graph.modal.submitBtn") }}</button>
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
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-card {
  width: 90%;
  max-width: 420px;
  background: var(--bg-surface-solid);
  border: 1px solid var(--border-strong);
  border-radius: 16px;
  padding: 22px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.modal-sub {
  font-size: 13px;
  color: var(--text-muted);
}

.modal-form {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.modal-select,
.modal-input {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-pure);
  border-radius: 10px;
  padding: 10px 12px;
  font-size: 13px;
  outline: none;
}

.modal-btn-row {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 4px;
}

.btn-cancel {
  background: transparent;
  border: 1px solid var(--border);
  color: var(--text-muted);
  padding: 8px 16px;
  border-radius: 10px;
  cursor: pointer;
}

.btn-submit {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  padding: 8px 18px;
  border-radius: 10px;
  font-weight: 700;
  cursor: pointer;
}
</style>