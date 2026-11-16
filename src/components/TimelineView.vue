<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { EnterpriseStoryItem, TimelineEventItem } from "../types";
import { useI18n } from "../i18n";
import TimelineTensionChart from "./timeline/TimelineTensionChart.vue";
import TimelineMilestoneModal from "./timeline/TimelineMilestoneModal.vue";

const { projectId, stories, operator = "Timeline-Executive" } = defineProps<{
  projectId: string;
  stories: EnterpriseStoryItem[];
  operator?: string;
}>();

const emit = defineEmits<{
  (e: "selectStory", story: EnterpriseStoryItem): void;
}>();

const { t } = useI18n();

const events = ref<TimelineEventItem[]>([]);
const selectedEvent = ref<TimelineEventItem | null>(null);
const showMilestoneModal = ref(false);
const editingEvent = ref<TimelineEventItem | null>(null);

async function loadTimeline() {
  try {
    const list = await invoke<TimelineEventItem[]>("fetch_timeline_events", {
      projectId,
    });
    events.value = list;
    selectedEvent.value = list.length > 0 ? list[0] : null;
  } catch {
    events.value = [];
    selectedEvent.value = null;
  }
}

function openCreateModal() {
  editingEvent.value = null;
  showMilestoneModal.value = true;
}

function openEditModal(ev: TimelineEventItem) {
  editingEvent.value = ev;
  showMilestoneModal.value = true;
}

async function handleSaveEvent(data: {
  id?: string;
  title: string;
  description: string;
  act: string;
  characterName: string;
  intensity: number;
  storyId: string | null;
  temporalOrder?: number;
}) {
  const payload: TimelineEventItem = {
    id: data.id || "",
    projectId,
    storyId: data.storyId,
    title: data.title,
    description: data.description,
    act: data.act,
    characterName: data.characterName,
    temporalOrder: data.temporalOrder ?? events.value.length + 1,
    intensity: data.intensity,
  };

  try {
    const saved = await invoke<TimelineEventItem>("mutate_timeline_event", {
      event: payload,
      operator,
    });
    const idx = events.value.findIndex((e) => e.id === saved.id);
    if (idx !== -1) {
      events.value[idx] = saved;
    } else {
      events.value.push(saved);
    }
    selectedEvent.value = saved;
    showMilestoneModal.value = false;
    editingEvent.value = null;
  } catch {
  }
}

async function handleDeleteEvent(id: string) {
  try {
    await invoke("delete_timeline_event", { id, operator });
    events.value = events.value.filter((e) => e.id !== id);
    if (selectedEvent.value?.id === id) {
      selectedEvent.value = events.value.length > 0 ? events.value[0] : null;
    }
  } catch {
  }
}

const charactersList = computed(() => {
  const set = new Set<string>();
  events.value.forEach((e) => set.add(e.characterName));
  return Array.from(set);
});

function jumpToLinkedStory(storyId: string | null | undefined) {
  if (!storyId) return;
  const match = stories.find((s) => s.id === storyId);
  if (match) {
    emit("selectStory", match);
  }
}

watch(
  () => projectId,
  () => {
    loadTimeline();
  }
);

onMounted(() => {
  loadTimeline();
});
</script>

<template>
  <section class="enterprise-timeline-stage">
    <header class="timeline-command-bar">
      <div class="command-title-group">
        <h2>{{ t("timeline.title") }}</h2>
        <p>{{ t("timeline.subtitle") }}</p>
      </div>

      <div class="command-btn-group">
        <button class="btn-create-milestone" @click="openCreateModal">
          <span>{{ t("timeline.newMilestoneBtn") }}</span>
        </button>
      </div>
    </header>

    <TimelineTensionChart
      :events="events"
      :selected-event-id="selectedEvent?.id || null"
      @select="selectedEvent = $event"
    />

    <div v-if="selectedEvent" class="selected-milestone-banner">
      <div class="milestone-banner-left">
        <span class="banner-badge">{{ selectedEvent.act }} • {{ selectedEvent.characterName }}</span>
        <h4>{{ selectedEvent.title }}</h4>
        <p>{{ selectedEvent.description }}</p>
      </div>
      <div class="milestone-banner-right">
        <span class="intensity-tag">{{ t("timeline.intensityLabel", { value: selectedEvent.intensity }) }}</span>
        <div class="banner-btns-row">
          <button class="btn-edit-banner" @click="openEditModal(selectedEvent)">
            ✎ {{ t("common.edit") }}
          </button>
          <button
            v-if="selectedEvent.storyId"
            class="btn-open-linked-scene"
            @click="jumpToLinkedStory(selectedEvent.storyId)"
          >
            {{ t("timeline.jumpToScene") }}
          </button>
        </div>
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
                {{ t("timeline.jumpToSceneShort") }}
              </button>
              <div class="milestone-card-actions">
                <button
                  class="btn-edit-milestone"
                  :title="t('timeline.editMilestoneTooltip')"
                  @click.stop="openEditModal(ev)"
                >
                  ✎
                </button>
                <button class="btn-del-milestone" @click.stop="handleDeleteEvent(ev.id)">✕</button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div v-if="events.length === 0" class="timeline-empty-card">
        <p>{{ t("timeline.emptyState") }}</p>
        <button class="btn-create-milestone" @click="openCreateModal">
          {{ t("timeline.createFirstBtn") }}
        </button>
      </div>
    </div>

    <TimelineMilestoneModal
      :is-open="showMilestoneModal"
      :editing-event="editingEvent"
      :stories="stories"
      @close="showMilestoneModal = false; editingEvent = null"
      @submit="handleSaveEvent"
    />
  </section>
</template>

<style scoped>
.enterprise-timeline-stage {
  flex: 1;
  overflow-y: auto;
  padding: 24px max(24px, var(--safe-right)) calc(var(--safe-bottom) + 32px) max(24px, var(--safe-left));
  display: flex;
  flex-direction: column;
  gap: 20px;
  background: var(--bg-deep);
  -webkit-overflow-scrolling: touch;
}

.timeline-command-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.command-title-group h2 {
  font-size: clamp(20px, 4vw, 24px);
  font-weight: 900;
  letter-spacing: -0.02em;
  color: var(--text-pure);
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
  min-height: 40px;
  padding: 10px 20px;
  font-size: 13px;
  font-weight: 800;
  cursor: pointer;
}

.selected-milestone-banner {
  background: var(--bg-card);
  border: 1px solid var(--accent-indigo);
  border-radius: 14px;
  padding: 16px 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
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
  color: var(--text-pure);
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

.banner-btns-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.btn-edit-banner {
  background: var(--bg-input);
  border: 1px solid var(--border);
  color: var(--text-pure);
  font-size: 11px;
  font-weight: 700;
  padding: 6px 10px;
  border-radius: 6px;
  cursor: pointer;
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
  padding: 16px 18px;
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
  scroll-snap-type: x mandatory;
  -webkit-overflow-scrolling: touch;
}

.milestone-card {
  width: min(78vw, 280px);
  flex-shrink: 0;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  cursor: pointer;
  scroll-snap-align: start;
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

.milestone-card-actions {
  display: flex;
  gap: 6px;
  margin-left: auto;
}

.btn-open-linked-scene {
  background: rgba(99, 102, 241, 0.16);
  border: 1px solid var(--accent-indigo);
  color: var(--text-pure);
  font-size: 11px;
  font-weight: 700;
  padding: 5px 10px;
  border-radius: 6px;
  cursor: pointer;
}

.btn-edit-milestone,
.btn-del-milestone {
  background: transparent;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  padding: 4px 6px;
}

.btn-edit-milestone:hover {
  color: var(--accent-cyan);
}

.btn-del-milestone:hover {
  color: var(--accent-rose);
}

.timeline-empty-card {
  padding: 50px 20px;
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

@media (max-width: 900px) {
  .enterprise-timeline-stage {
    padding: 14px max(12px, var(--safe-right)) 24px max(12px, var(--safe-left));
  }

  .milestone-banner-right {
    align-items: flex-start;
  }
}
</style>