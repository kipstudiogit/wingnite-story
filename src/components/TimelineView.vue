<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { EnterpriseStoryItem, TimelineEventItem } from "../types";

const { stories, operator = "Timeline-Executive" } = defineProps<{
  stories: EnterpriseStoryItem[];
  operator?: string;
}>();

const emit = defineEmits<{
  (e: "selectStory", story: EnterpriseStoryItem): void;
}>();

const events = ref<TimelineEventItem[]>([]);
const selectedEvent = ref<TimelineEventItem | null>(null);
const showCreateModal = ref(false);

const newTitle = ref("");
const newDesc = ref("");
const newAct = ref("Акт I");
const newChar = ref("Элион");
const newIntensity = ref(60);
const newStoryId = ref<string>("");

async function loadTimeline() {
  try {
    const list = await invoke<TimelineEventItem[]>("fetch_timeline_events");
    events.value = list;
    if (list.length > 0 && !selectedEvent.value) {
      selectedEvent.value = list[0];
    }
  } catch {
    events.value = [];
  }
}

async function handleCreateEvent() {
  if (!newTitle.value.trim()) return;
  const payload: TimelineEventItem = {
    id: "",
    storyId: newStoryId.value || null,
    title: newTitle.value,
    description: newDesc.value,
    act: newAct.value,
    characterName: newChar.value,
    temporalOrder: events.value.length + 1,
    intensity: newIntensity.value,
  };

  try {
    const saved = await invoke<TimelineEventItem>("mutate_timeline_event", {
      event: payload,
      operator,
    });
    events.value.push(saved);
    selectedEvent.value = saved;
    showCreateModal.value = false;
    newTitle.value = "";
    newDesc.value = "";
  } catch {}
}

async function handleDeleteEvent(id: string) {
  try {
    await invoke("delete_timeline_event", { id, operator });
    events.value = events.value.filter((e) => e.id !== id);
    if (selectedEvent.value?.id === id) {
      selectedEvent.value = events.value.length > 0 ? events.value[0] : null;
    }
  } catch {}
}

const charactersList = computed(() => {
  const set = new Set<string>();
  events.value.forEach((e) => set.add(e.characterName));
  return Array.from(set);
});

const tensionPathData = computed(() => {
  if (events.value.length === 0) return "";
  if (events.value.length === 1) {
    const y = 110 - (events.value[0].intensity / 100) * 90;
    return `M 20 ${y} L 980 ${y}`;
  }
  return events.value.reduce((acc, ev, idx) => {
    const x = (idx / (events.value.length - 1)) * 960 + 20;
    const y = 110 - (ev.intensity / 100) * 90;
    return `${acc} L ${x} ${y}`;
  }, `M 20 ${110 - (events.value[0].intensity / 100) * 90}`);
});

const tensionFillData = computed(() => {
  if (!tensionPathData.value || events.value.length === 0) return "";
  return `${tensionPathData.value} L 980 120 L 20 120 Z`;
});

function jumpToLinkedStory(storyId: string | null | undefined) {
  if (!storyId) return;
  const match = stories.find((s) => s.id === storyId);
  if (match) {
    emit("selectStory", match);
  }
}

onMounted(() => {
  loadTimeline();
});
</script>

<template>
  <section class="enterprise-timeline-stage">
    <header class="timeline-command-bar">
      <div class="command-title-group">
        <h2>Хронологическая матрица</h2>
        <p>Многопоточный реестр вех сюжета и амплитуды драматического напряжения</p>
      </div>

      <div class="command-btn-group">
        <button class="btn-create-milestone" @click="showCreateModal = true">
          <span>+ Новая веха</span>
        </button>
      </div>
    </header>

    <div class="tension-graph-card">
      <div class="graph-legend">
        <span>График драматического напряжения (Telemetry Tensor)</span>
        <span class="legend-scale">0% Покой &nbsp;•&nbsp; 100% Кульминация</span>
      </div>

      <div class="tension-svg-wrap">
        <svg class="tension-svg" viewBox="0 0 1000 120" preserveAspectRatio="none">
          <defs>
            <linearGradient id="tensorGrad" x1="0%" y1="0%" x2="100%" y2="0%">
              <stop offset="0%" stop-color="#6366f1" stop-opacity="0.9" />
              <stop offset="50%" stop-color="#06b6d4" stop-opacity="0.95" />
              <stop offset="100%" stop-color="#f59e0b" stop-opacity="1" />
            </linearGradient>
            <linearGradient id="tensorFill" x1="0%" y1="0%" x2="0%" y2="100%">
              <stop offset="0%" stop-color="#6366f1" stop-opacity="0.35" />
              <stop offset="100%" stop-color="#6366f1" stop-opacity="0" />
            </linearGradient>
          </defs>

          <path
            v-if="tensionFillData"
            :d="tensionFillData"
            fill="url(#tensorFill)"
          />

          <path
            v-if="tensionPathData"
            :d="tensionPathData"
            fill="none"
            stroke="url(#tensorGrad)"
            stroke-width="3"
            stroke-linecap="round"
          />

          <circle
            v-for="(ev, idx) in events"
            :key="ev.id"
            :cx="(idx / (Math.max(1, events.length - 1))) * 960 + 20"
            :cy="110 - (ev.intensity / 100) * 90"
            r="6"
            :class="['tension-pulse-dot', { active: selectedEvent?.id === ev.id }]"
            @click="selectedEvent = ev"
          />
        </svg>
      </div>
    </div>

    <div v-if="selectedEvent" class="selected-milestone-banner">
      <div class="milestone-banner-left">
        <span class="banner-badge">{{ selectedEvent.act }} • {{ selectedEvent.characterName }}</span>
        <h4>{{ selectedEvent.title }}</h4>
        <p>{{ selectedEvent.description }}</p>
      </div>
      <div class="milestone-banner-right">
        <span class="intensity-tag">Напряжение: {{ selectedEvent.intensity }}%</span>
        <button
          v-if="selectedEvent.storyId"
          class="btn-open-linked-scene"
          @click="jumpToLinkedStory(selectedEvent.storyId)"
        >
          Перейти к привязанной сцене →
        </button>
      </div>
    </div>

    <div class="swimlanes-container">
      <div v-for="charName in charactersList" :key="charName" class="swimlane-track">
        <div class="swimlane-actor-badge">
          <span class="actor-orb"></span>
          <span class="actor-name">{{ charName }}</span>
        </div>

        <div class="swimlane-events-row">
          <div
            v-for="ev in events.filter((e) => e.characterName === charName)"
            :key="ev.id"
            :class="['milestone-card', { active: selectedEvent?.id === ev.id }]"
            @click="selectedEvent = ev"
          >
            <div class="milestone-top">
              <span class="milestone-act">{{ ev.act }}</span>
              <span class="milestone-intensity">{{ ev.intensity }}%</span>
            </div>
            <h4 class="milestone-title">{{ ev.title }}</h4>
            <p class="milestone-desc">{{ ev.description }}</p>

            <div class="milestone-footer">
              <button
                v-if="ev.storyId"
                class="btn-open-linked-scene"
                @click.stop="jumpToLinkedStory(ev.storyId)"
              >
                ↗ К сцене
              </button>
              <button class="btn-del-milestone" @click.stop="handleDeleteEvent(ev.id)">✕</button>
            </div>
          </div>
        </div>
      </div>

      <div v-if="events.length === 0" class="timeline-empty-card">
        <p>Хронологическая матрица пуста</p>
        <button class="btn-create-milestone" @click="showCreateModal = true">+ Создать первую сюжетную веху</button>
      </div>
    </div>

    <div v-if="showCreateModal" class="modal-backdrop" @click.self="showCreateModal = false">
      <div class="modal-window">
        <h3>Фиксация сюжетной вехи</h3>
        <div class="form-inputs">
          <input v-model="newTitle" class="modal-input" placeholder="Название вехи..." />
          <textarea
            v-model="newDesc"
            class="modal-textarea"
            placeholder="Описание события и его влияние на мир..."
          ></textarea>
          <div class="modal-row">
            <input v-model="newAct" class="modal-input" placeholder="Акт (напр. Акт I)" />
            <input v-model="newChar" class="modal-input" placeholder="Сущность / Персонаж" />
          </div>
          <div class="modal-row">
            <label class="slider-label">
              Интенсивность: <b>{{ newIntensity }}%</b>
              <input v-model.number="newIntensity" type="range" min="10" max="100" class="modal-slider" />
            </label>
            <select v-model="newStoryId" class="modal-select">
              <option value="">Связать со сценой (опционально)</option>
              <option v-for="s in stories" :key="s.id" :value="s.id">{{ s.title }}</option>
            </select>
          </div>
        </div>
        <div class="modal-actions">
          <button class="btn-modal-cancel" @click="showCreateModal = false">Отмена</button>
          <button class="btn-modal-submit" @click="handleCreateEvent">Сохранить</button>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.enterprise-timeline-stage {
  flex: 1;
  overflow-y: auto;
  padding: 28px 32px calc(var(--safe-bottom) + 80px) 32px;
  display: flex;
  flex-direction: column;
  gap: 22px;
  background: var(--bg-deep);
}

.timeline-command-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.command-title-group h2 {
  font-size: 24px;
  font-weight: 900;
  letter-spacing: -0.02em;
  background: linear-gradient(135deg, #fff 50%, var(--accent-cyan) 100%);
  -webkit-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
}

.command-title-group p {
  color: var(--text-muted);
  font-size: 13px;
  margin-top: 4px;
}

.btn-create-milestone {
  background: linear-gradient(135deg, var(--accent-indigo) 0%, #4338ca 100%);
  color: #fff;
  border: none;
  border-radius: 12px;
  padding: 10px 20px;
  font-size: 13px;
  font-weight: 800;
  cursor: pointer;
}

.tension-graph-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 18px 24px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.graph-legend {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--text-muted);
}

.legend-scale {
  color: var(--text-dim);
  font-family: var(--font-mono);
}

.tension-svg-wrap {
  width: 100%;
  height: 120px;
}

.tension-svg {
  width: 100%;
  height: 100%;
  overflow: visible;
}

.tension-pulse-dot {
  fill: #fff;
  stroke: var(--accent-indigo);
  stroke-width: 3px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tension-pulse-dot:hover,
.tension-pulse-dot.active {
  r: 8;
  stroke: var(--accent-cyan);
  fill: var(--accent-cyan);
}

.selected-milestone-banner {
  background: var(--bg-card);
  border: 1px solid var(--accent-indigo);
  border-radius: 14px;
  padding: 16px 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 20px;
}

.banner-badge {
  font-size: 10px;
  font-weight: 800;
  color: var(--accent-cyan);
  text-transform: uppercase;
}

.milestone-banner-left h4 {
  font-size: 16px;
  font-weight: 800;
  margin: 4px 0;
}

.milestone-banner-left p {
  font-size: 12px;
  color: var(--text-muted);
}

.milestone-banner-right {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
}

.intensity-tag {
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 700;
  color: var(--accent-gold);
}

.swimlanes-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.swimlane-track {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.swimlane-actor-badge {
  display: flex;
  align-items: center;
  gap: 10px;
}

.actor-orb {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent-cyan);
}

.actor-name {
  font-size: 12px;
  font-weight: 800;
  color: var(--text-pure);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.swimlane-events-row {
  display: flex;
  gap: 14px;
  overflow-x: auto;
  padding-bottom: 6px;
}

.milestone-card {
  width: 280px;
  flex-shrink: 0;
  background: rgba(0, 0, 0, 0.45);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  cursor: pointer;
}

.milestone-card.active {
  background: var(--bg-card-hover);
  border-color: var(--border-accent);
}

.milestone-top {
  display: flex;
  justify-content: space-between;
  font-size: 10px;
  font-weight: 700;
}

.milestone-act {
  color: var(--accent-cyan);
  text-transform: uppercase;
}

.milestone-intensity {
  color: var(--accent-gold);
  font-family: var(--font-mono);
}

.milestone-title {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-pure);
}

.milestone-desc {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
}

.milestone-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 6px;
}

.btn-open-linked-scene {
  background: rgba(99, 102, 241, 0.15);
  border: 1px solid var(--accent-indigo);
  color: var(--text-pure);
  font-size: 11px;
  font-weight: 700;
  padding: 4px 8px;
  border-radius: 6px;
  cursor: pointer;
}

.btn-del-milestone {
  background: transparent;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
}

.timeline-empty-card {
  padding: 60px 20px;
  text-align: center;
  background: var(--bg-card);
  border: 1px dashed var(--border);
  border-radius: 14px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  color: var(--text-dim);
}

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
}

.modal-textarea {
  resize: vertical;
  min-height: 80px;
}

.modal-row {
  display: flex;
  gap: 10px;
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

@media (max-width: 768px) {
  .enterprise-timeline-stage {
    padding: 16px 14px calc(var(--safe-bottom) + 84px) 14px;
  }
}
</style>