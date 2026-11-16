<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { ref, watch } from "vue";
import type { EnterpriseStoryItem, TimelineEventItem } from "../../types";
import { useI18n } from "../../i18n";

const { isOpen, editingEvent = null, stories } = defineProps<{
  isOpen: boolean;
  editingEvent?: TimelineEventItem | null;
  stories: EnterpriseStoryItem[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (
    e: "submit",
    payload: {
      id?: string;
      title: string;
      description: string;
      act: string;
      characterName: string;
      intensity: number;
      storyId: string | null;
      temporalOrder?: number;
    }
  ): void;
}>();

const { t } = useI18n();

const title = ref("");
const description = ref("");
const act = ref(t("timeline.modal.defaultAct"));
const characterName = ref(t("timeline.modal.defaultCharacter"));
const intensity = ref(60);
const storyId = ref("");

watch(
  [() => isOpen, () => editingEvent],
  ([open, ev]) => {
    if (!open) return;
    if (ev) {
      title.value = ev.title;
      description.value = ev.description;
      act.value = ev.act;
      characterName.value = ev.characterName;
      intensity.value = ev.intensity;
      storyId.value = ev.storyId || "";
    } else {
      title.value = "";
      description.value = "";
      act.value = t("timeline.modal.defaultAct");
      characterName.value = t("timeline.modal.defaultCharacter");
      intensity.value = 60;
      storyId.value = "";
    }
  }
);

function handleSubmit() {
  if (!title.value.trim()) return;
  emit("submit", {
    id: editingEvent?.id,
    title: title.value.trim(),
    description: description.value.trim(),
    act: act.value.trim() || t("timeline.modal.defaultAct"),
    characterName: characterName.value.trim() || t("timeline.modal.defaultCharacter"),
    intensity: intensity.value,
    storyId: storyId.value || null,
    temporalOrder: editingEvent?.temporalOrder,
  });
}
</script>

<template>
  <div v-if="isOpen" class="modal-backdrop" @click.self="emit('close')">
    <div class="modal-window">
      <h3>{{ editingEvent ? t("timeline.modal.editTitle") : t("timeline.modal.title") }}</h3>
      <div class="form-inputs">
        <input v-model="title" class="modal-input" :placeholder="t('timeline.modal.namePlaceholder')" />
        <textarea
          v-model="description"
          class="modal-textarea"
          :placeholder="t('timeline.modal.descPlaceholder')"
        ></textarea>
        <div class="modal-row">
          <input v-model="act" class="modal-input" :placeholder="t('timeline.modal.actPlaceholder')" />
          <input v-model="characterName" class="modal-input" :placeholder="t('timeline.modal.characterPlaceholder')" />
        </div>
        <div class="modal-row">
          <label class="slider-label">
            {{ t("timeline.modal.intensityLabel") }} <b>{{ intensity }}%</b>
            <input v-model.number="intensity" type="range" min="10" max="100" class="modal-slider" />
          </label>
          <select v-model="storyId" class="modal-select">
            <option value="">{{ t("timeline.modal.linkSceneOption") }}</option>
            <option v-for="s in stories" :key="s.id" :value="s.id">{{ s.title }}</option>
          </select>
        </div>
      </div>
      <div class="modal-actions">
        <button class="btn-modal-cancel" @click="emit('close')">{{ t("common.cancel") }}</button>
        <button class="btn-modal-submit" @click="handleSubmit">{{ t("common.save") }}</button>
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
  z-index: 210;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-window {
  width: 90%;
  max-width: 500px;
  background: var(--bg-surface-solid);
  border: 1px solid var(--border-strong);
  border-radius: 16px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-inputs {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.modal-input,
.modal-textarea,
.modal-select {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-pure);
  border-radius: 10px;
  padding: 10px 14px;
  font-size: 13px;
  outline: none;
  flex: 1;
}

.modal-textarea {
  resize: vertical;
  min-height: 80px;
}

.modal-row {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.slider-label {
  flex: 1;
  font-size: 11px;
  color: var(--text-muted);
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.modal-slider {
  accent-color: var(--accent-indigo);
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
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