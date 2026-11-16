<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { EnterpriseStoryItem, EntityRelationItem, GraphNodePacked } from "../types";
import { useGraphCanvas } from "../composables/useGraphCanvas";
import { useHaptics } from "../composables/useHaptics";
import { useI18n } from "../i18n";
import GraphNodeInspector, { type InspectorRelationView } from "./graph/GraphNodeInspector.vue";
import GraphRelationModal from "./graph/GraphRelationModal.vue";

const { projectId, stories, operator = "Cosmos-Architect" } = defineProps<{
  projectId: string;
  stories: EnterpriseStoryItem[];
  operator?: string;
}>();

const emit = defineEmits<{
  (e: "select", story: EnterpriseStoryItem): void;
  (e: "notify", msg: string, kind: "success" | "error"): void;
}>();

const { vibrateSuccess } = useHaptics();
const { t } = useI18n();

const canvasRef = ref<HTMLCanvasElement | null>(null);
const minimapRef = ref<HTMLCanvasElement | null>(null);
const viewportRef = ref<HTMLDivElement | null>(null);
const searchQuery = ref("");
const showCreateRelationModal = ref(false);
const relations = ref<EntityRelationItem[]>([]);

function openStoryNode(node: GraphNodePacked) {
  const match = stories.find((s) => s.id === node.id);
  if (match) {
    emit("select", match);
  }
}

const {
  selectedNode,
  filterMode,
  setFilterMode,
  loadGraphData,
  teleportToLabel,
  resetTransform,
  zoomIn,
  zoomOut,
  handleMouseDown,
  handleMouseMove,
  handleMouseUp,
  handleWheel,
  handleTouchStart,
  handleTouchMove,
  handleTouchEnd,
  handleCanvasClick,
  handleCanvasDblClick,
} = useGraphCanvas(() => projectId, canvasRef, minimapRef, viewportRef, openStoryNode);

async function loadRelations() {
  try {
    relations.value = await invoke<EntityRelationItem[]>("fetch_entity_relations", {
      projectId,
    });
  } catch {
    relations.value = [];
  }
}

const selectedNodeRelations = computed<InspectorRelationView[]>(() => {
  const active = selectedNode.value;
  if (!active) return [];
  return relations.value
    .filter((r) => r.sourceId === active.id || r.targetId === active.id)
    .map((r) => {
      const peerId = r.sourceId === active.id ? r.targetId : r.sourceId;
      const peer = stories.find((s) => s.id === peerId);
      return {
        id: r.id,
        peerTitle: peer ? peer.title : peerId,
        relationType: r.relationType,
        description: r.description,
      };
    });
});

async function handleCreateRelation(data: {
  targetId: string;
  relationType: string;
  description: string;
}) {
  if (!selectedNode.value) return;

  const payload: EntityRelationItem = {
    id: "",
    sourceId: selectedNode.value.id,
    targetId: data.targetId,
    relationType: data.relationType,
    description: data.description,
  };

  try {
    await invoke("mutate_entity_relation", { relation: payload, operator });
    showCreateRelationModal.value = false;
    vibrateSuccess();
    await Promise.all([loadGraphData(), loadRelations()]);
  } catch {
  }
}

async function handleUnlink(relationId: string) {
  try {
    await invoke("delete_entity_relation", { id: relationId, operator });
    vibrateSuccess();
    await Promise.all([loadGraphData(), loadRelations()]);
    emit("notify", t("notifications.relationDeletedToast"), "success");
  } catch {
  }
}

watch(
  [() => projectId, () => stories.length],
  () => {
    loadGraphData();
    loadRelations();
  }
);

onMounted(() => {
  loadRelations();
});
</script>

<template>
  <div ref="viewportRef" class="stellar-graph-viewport">
    <div class="cosmos-telemetry-hud">
      <div class="hud-filter-pills">
        <button
          :class="['btn-filter-pill', { active: filterMode === 'all' }]"
          @click="setFilterMode('all')"
        >
          {{ t("graph.filterAll") }}
        </button>
        <button
          :class="['btn-filter-pill', { active: filterMode === 'entities' }]"
          @click="setFilterMode('entities')"
        >
          {{ t("graph.filterEntities") }}
        </button>
      </div>

      <div class="hud-search-capsule">
        <input
          v-model="searchQuery"
          class="hud-search-input"
          :placeholder="t('graph.searchPlaceholder')"
          @keydown.enter="teleportToLabel(searchQuery)"
        />
        <button class="btn-spotlight" @click="teleportToLabel(searchQuery)">
          {{ t("graph.teleportBtn") }}
        </button>
      </div>

      <div class="hud-buttons-cluster">
        <button class="btn-hud-control" :title="t('graph.resetCamera')" @click="resetTransform">⊙</button>
        <button class="btn-hud-control" :title="t('graph.zoomIn')" @click="zoomIn">+</button>
        <button class="btn-hud-control" :title="t('graph.zoomOut')" @click="zoomOut">−</button>
      </div>
    </div>

    <canvas
      ref="canvasRef"
      @mousedown="handleMouseDown"
      @mousemove="handleMouseMove"
      @mouseup="handleMouseUp"
      @wheel="handleWheel"
      @touchstart="handleTouchStart"
      @touchmove="handleTouchMove"
      @touchend="handleTouchEnd"
      @click="handleCanvasClick"
      @dblclick="handleCanvasDblClick"
    ></canvas>

    <div class="minimap-container">
      <canvas ref="minimapRef" width="120" height="90" class="minimap-canvas"></canvas>
      <span class="minimap-caption">{{ t("graph.minimapCaption") }}</span>
    </div>

    <GraphNodeInspector
      v-if="selectedNode"
      :node="selectedNode"
      :node-relations="selectedNodeRelations"
      @close="selectedNode = null"
      @open-story="openStoryNode"
      @open-link-modal="showCreateRelationModal = true"
      @unlink="handleUnlink"
    />

    <GraphRelationModal
      :is-open="showCreateRelationModal"
      :source-node="selectedNode"
      :stories="stories"
      @close="showCreateRelationModal = false"
      @submit="handleCreateRelation"
    />
  </div>
</template>

<style scoped>
.stellar-graph-viewport {
  flex: 1;
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
  background: radial-gradient(circle at 50% 50%, #060914 0%, #020306 100%);
  touch-action: none;
}

canvas {
  display: block;
  width: 100%;
  height: 100%;
  touch-action: none;
}

.cosmos-telemetry-hud {
  position: absolute;
  top: max(12px, var(--safe-top));
  left: max(12px, var(--safe-left));
  right: max(12px, var(--safe-right));
  display: flex;
  gap: 8px;
  align-items: center;
  z-index: 10;
  flex-wrap: wrap;
  justify-content: flex-end;
  pointer-events: none;
}

.cosmos-telemetry-hud > * {
  pointer-events: auto;
}

.hud-filter-pills {
  display: flex;
  background: var(--bg-surface);
  backdrop-filter: blur(20px);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 3px;
  gap: 3px;
}

.btn-filter-pill {
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 10px;
  font-weight: 700;
  padding: 6px 10px;
  border-radius: 7px;
  cursor: pointer;
}

.btn-filter-pill.active {
  background: var(--accent-indigo);
  color: #fff;
}

.hud-search-capsule {
  display: flex;
  background: var(--bg-surface);
  backdrop-filter: blur(20px);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 3px;
  gap: 4px;
}

.hud-search-input {
  background: transparent;
  border: none;
  color: var(--text-pure);
  font-size: 11px;
  padding: 0 8px;
  width: min(36vw, 170px);
  outline: none;
}

.btn-spotlight {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  border-radius: 7px;
  padding: 5px 10px;
  font-size: 10px;
  font-weight: 700;
  cursor: pointer;
}

.hud-buttons-cluster {
  display: flex;
  background: var(--bg-surface);
  backdrop-filter: blur(20px);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 3px;
  gap: 3px;
}

.btn-hud-control {
  background: transparent;
  border: none;
  color: var(--text-pure);
  width: 32px;
  height: 32px;
  border-radius: 7px;
  font-size: 14px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-hud-control:hover {
  background: var(--border-subtle);
}

.minimap-container {
  position: absolute;
  bottom: max(20px, var(--safe-bottom));
  right: max(20px, var(--safe-right));
  background: var(--bg-surface-solid);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 4px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.7);
  z-index: 10;
}

.minimap-canvas {
  border-radius: 8px;
}

.minimap-caption {
  font-size: 8px;
  font-family: var(--font-mono);
  font-weight: 800;
  text-transform: uppercase;
  color: var(--text-dim);
  text-align: center;
}

@media (max-width: 900px) {
  .cosmos-telemetry-hud {
    justify-content: space-between;
  }

  .minimap-container {
    display: none;
  }
}
</style>