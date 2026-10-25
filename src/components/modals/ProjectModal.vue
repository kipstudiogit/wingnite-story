<script setup lang="ts">
import { ref } from "vue";

const { isOpen } = defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "submit", payload: { title: string; description: string }): void;
}>();

const title = ref("");
const description = ref("");

function handleConfirm() {
  if (!title.value.trim()) return;
  emit("submit", {
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
      <h3>Новое литературное произведение</h3>
      <p class="modal-subtext">Создание изолированного пространства для глав, сцен, канбана и реестра канона.</p>
      
      <div class="form-inputs">
        <input
          v-model="title"
          class="modal-input"
          placeholder="Название книги или саги..."
          autofocus
          @keydown.enter="handleConfirm"
        />
        <textarea
          v-model="description"
          class="modal-textarea"
          placeholder="Краткое описание сеттинга, синопсис или логлайн..."
        ></textarea>
      </div>

      <div class="modal-actions">
        <button class="btn-modal-cancel" @click="emit('close')">Отмена</button>
        <button class="btn-modal-submit" @click="handleConfirm">Создать произведение</button>
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