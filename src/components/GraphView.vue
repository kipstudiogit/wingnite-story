<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  EnterpriseStoryItem,
  EntityRelationItem,
  GraphNodePacked,
  GraphEdgePacked,
  GraphTopologyPayload,
} from "../types";
import { useHaptics } from "../composables/useHaptics";

const { stories, operator = "Cosmos-Architect" } = defineProps<{
  stories: EnterpriseStoryItem[];
  operator?: string;
}>();

const emit = defineEmits<{
  (e: "select", story: EnterpriseStoryItem): void;
}>();

const { vibrateSelection, vibrateSuccess } = useHaptics();

const canvasRef = ref<HTMLCanvasElement | null>(null);
const minimapRef = ref<HTMLCanvasElement | null>(null);
const viewportRef = ref<HTMLDivElement | null>(null);
const selectedNode = ref<GraphNodePacked | null>(null);
const searchQuery = ref("");

const showCreateRelationModal = ref(false);
const newRelationTargetId = ref("");
const newRelationType = ref("Союзник");
const newRelationDesc = ref("");

const panX = ref(0);
const panY = ref(0);
const zoom = ref(1.0);

let animationId: number | null = null;
let resizeObserver: ResizeObserver | null = null;

let rawNodes: GraphNodePacked[] = [];
let rawEdges: GraphEdgePacked[] = [];

let positions: Float32Array = new Float32Array(0);
let velocities: Float32Array = new Float32Array(0);
let isSimulating = false;
let pulsePhase = 0;

let isDraggingCanvas = false;
let dragStartX = 0;
let dragStartY = 0;
let initialPanX = 0;
let initialPanY = 0;
let initialTouchDistance = 0;
let initialZoom = 1.0;

async function loadGraphData() {
  try {
    const payload = await invoke<GraphTopologyPayload>("fetch_packed_topology");
    rawNodes = payload.nodes;
    rawEdges = payload.edges;

    const count = rawNodes.length;
    positions = new Float32Array(count * 2);
    velocities = new Float32Array(count * 2);

    for (let i = 0; i < count; i++) {
      const angle = (i / count) * Math.PI * 2;
      const radius = 0.45 + (i % 3) * 0.18;
      positions[i * 2] = Math.cos(angle) * radius;
      positions[i * 2 + 1] = Math.sin(angle) * radius;
    }

    isSimulating = true;
    requestRenderLoop();
  } catch {
    rawNodes = [];
    rawEdges = [];
  }
}

function updatePhysicsStep(): number {
  let totalEnergy = 0;
  const count = rawNodes.length;

  for (let i = 0; i < count; i++) {
    const i2 = i * 2;
    velocities[i2] -= positions[i2] * 0.0003;
    velocities[i2 + 1] -= positions[i2 + 1] * 0.0003;

    for (let j = i + 1; j < count; j++) {
      const j2 = j * 2;
      const dx = positions[j2] - positions[i2];
      const dy = positions[j2 + 1] - positions[i2 + 1];
      const distSq = dx * dx + dy * dy + 0.001;
      if (distSq < 0.3) {
        const force = 0.00025 / distSq;
        velocities[i2] -= dx * force;
        velocities[i2 + 1] -= dy * force;
        velocities[j2] += dx * force;
        velocities[j2 + 1] += dy * force;
      }
    }
  }

  for (let i = 0; i < rawEdges.length; i++) {
    const e = rawEdges[i];
    const s2 = e.sourceIdx * 2;
    const t2 = e.targetIdx * 2;
    const dx = positions[t2] - positions[s2];
    const dy = positions[t2 + 1] - positions[s2 + 1];
    const force = 0.0018 * e.weight;
    velocities[s2] += dx * force;
    velocities[s2 + 1] += dy * force;
    velocities[t2] -= dx * force;
    velocities[t2 + 1] -= dy * force;
  }

  for (let i = 0; i < count * 2; i++) {
    velocities[i] *= 0.89;
    positions[i] += velocities[i];
    totalEnergy += Math.abs(velocities[i]);
  }

  return totalEnergy;
}

function renderFrame() {
  if (isSimulating) {
    const energy = updatePhysicsStep();
    if (energy < 0.004) {
      isSimulating = false;
    }
  }

  pulsePhase = (pulsePhase + 0.04) % (Math.PI * 2);

  renderStellarCosmos();
  renderMinimap();

  animationId = requestAnimationFrame(renderFrame);
}

function renderStellarCosmos() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  const w = canvas.width;
  const h = canvas.height;
  ctx.clearRect(0, 0, w, h);

  const cx = w / 2 + panX.value;
  const cy = h / 2 + panY.value;
  const scale = Math.min(w, h) * 0.45 * zoom.value;

  for (let i = 0; i < rawEdges.length; i++) {
    const e = rawEdges[i];
    const sx = cx + positions[e.sourceIdx * 2] * scale;
    const sy = cy + positions[e.sourceIdx * 2 + 1] * scale;
    const tx = cx + positions[e.targetIdx * 2] * scale;
    const ty = cy + positions[e.targetIdx * 2 + 1] * scale;

    const isConflict = e.relationType.includes("Вражда");
    const isPossess = e.relationType.includes("Владение");

    ctx.strokeStyle = isConflict
      ? "rgba(244, 63, 94, 0.4)"
      : isPossess
      ? "rgba(245, 158, 11, 0.45)"
      : "rgba(6, 182, 212, 0.35)";
    ctx.lineWidth = Math.max(1, 1.4 * zoom.value);

    ctx.beginPath();
    ctx.moveTo(sx, sy);
    ctx.lineTo(tx, ty);
    ctx.stroke();

    const t = (Math.sin(pulsePhase + i) + 1) / 2;
    const pulseX = sx + (tx - sx) * t;
    const pulseY = sy + (ty - sy) * t;

    ctx.fillStyle = isConflict ? "#f43f5e" : isPossess ? "#f59e0b" : "#38bdf8";
    ctx.beginPath();
    ctx.arc(pulseX, pulseY, 2.5 * zoom.value, 0, Math.PI * 2);
    ctx.fill();
  }

  for (let i = 0; i < rawNodes.length; i++) {
    const node = rawNodes[i];
    const x = cx + positions[i * 2] * scale;
    const y = cy + positions[i * 2 + 1] * scale;
    const baseRadius = (node.isDivine ? 10 : 6.5) * zoom.value;
    const isSelected = selectedNode.value?.id === node.id;

    if (node.isDivine || isSelected) {
      const glowGrad = ctx.createRadialGradient(x, y, 0, x, y, baseRadius * 3.5);
      glowGrad.addColorStop(0, node.isDivine ? "rgba(245, 158, 11, 0.5)" : "rgba(6, 182, 212, 0.5)");
      glowGrad.addColorStop(1, "transparent");
      ctx.fillStyle = glowGrad;
      ctx.beginPath();
      ctx.arc(x, y, baseRadius * 3.5, 0, Math.PI * 2);
      ctx.fill();
    }

    ctx.fillStyle = node.isDivine ? "#f59e0b" : node.category.includes("Персонаж") ? "#38bdf8" : "#a855f7";
    ctx.beginPath();
    ctx.arc(x, y, baseRadius, 0, Math.PI * 2);
    ctx.fill();

    if (isSelected) {
      ctx.strokeStyle = "#ffffff";
      ctx.lineWidth = 2;
      ctx.stroke();
    }

    ctx.fillStyle = "#ffffff";
    ctx.font = `${Math.round(11 * Math.min(1.4, Math.max(0.7, zoom.value)))}px sans-serif`;
    ctx.fillText(node.label, x + baseRadius + 5, y + 4);
  }
}

function renderMinimap() {
  const mCanvas = minimapRef.value;
  if (!mCanvas) return;
  const mCtx = mCanvas.getContext("2d");
  if (!mCtx) return;

  const mw = mCanvas.width;
  const mh = mCanvas.height;
  mCtx.clearRect(0, 0, mw, mh);

  mCtx.fillStyle = "rgba(6, 9, 16, 0.85)";
  mCtx.fillRect(0, 0, mw, mh);

  const mcx = mw / 2;
  const mcy = mh / 2;
  const mscale = Math.min(mw, mh) * 0.4;

  for (let i = 0; i < rawNodes.length; i++) {
    const nx = mcx + positions[i * 2] * mscale;
    const ny = mcy + positions[i * 2 + 1] * mscale;
    mCtx.fillStyle = rawNodes[i].isDivine ? "#f59e0b" : "#38bdf8";
    mCtx.beginPath();
    mCtx.arc(nx, ny, 2, 0, Math.PI * 2);
    mCtx.fill();
  }

  const vBoxW = (mw / zoom.value) * 0.45;
  const vBoxH = (mh / zoom.value) * 0.45;
  const vBoxX = mcx - (panX.value / 600) * mscale - vBoxW / 2;
  const vBoxY = mcy - (panY.value / 600) * mscale - vBoxH / 2;

  mCtx.strokeStyle = "rgba(6, 182, 212, 0.7)";
  mCtx.lineWidth = 1;
  mCtx.strokeRect(vBoxX, vBoxY, vBoxW, vBoxH);
}

function handleSearchSpotlight() {
  const q = searchQuery.value.toLowerCase().trim();
  if (!q) return;
  const matchIdx = rawNodes.findIndex((n) => n.label.toLowerCase().includes(q));
  if (matchIdx !== -1) {
    const canvas = canvasRef.value;
    if (!canvas) return;
    const scale = Math.min(canvas.width, canvas.height) * 0.45 * zoom.value;
    panX.value = -positions[matchIdx * 2] * scale;
    panY.value = -positions[matchIdx * 2 + 1] * scale;
    selectedNode.value = rawNodes[matchIdx];
    vibrateSelection();
    requestRenderLoop();
  }
}

function requestRenderLoop() {
  if (!animationId) {
    animationId = requestAnimationFrame(renderFrame);
  }
}

function wakePhysics() {
  if (!isSimulating) {
    isSimulating = true;
    requestRenderLoop();
  }
}

function getNodeAtPoint(mx: number, my: number): GraphNodePacked | null {
  const canvas = canvasRef.value;
  if (!canvas) return null;
  const cx = canvas.width / 2 + panX.value;
  const cy = canvas.height / 2 + panY.value;
  const scale = Math.min(canvas.width, canvas.height) * 0.45 * zoom.value;

  for (let i = 0; i < rawNodes.length; i++) {
    const x = cx + positions[i * 2] * scale;
    const y = cy + positions[i * 2 + 1] * scale;
    const dx = x - mx;
    const dy = y - my;
    const hitRadius = Math.max(18, 14 * zoom.value);
    if (dx * dx + dy * dy < hitRadius * hitRadius) {
      return rawNodes[i];
    }
  }
  return null;
}

function handleMouseDown(e: MouseEvent) {
  isDraggingCanvas = true;
  dragStartX = e.clientX;
  dragStartY = e.clientY;
  initialPanX = panX.value;
  initialPanY = panY.value;
}

function handleMouseMove(e: MouseEvent) {
  if (!isDraggingCanvas) return;
  panX.value = initialPanX + (e.clientX - dragStartX);
  panY.value = initialPanY + (e.clientY - dragStartY);
}

function handleMouseUp() {
  isDraggingCanvas = false;
}

function handleWheel(e: WheelEvent) {
  e.preventDefault();
  const delta = e.deltaY < 0 ? 1.12 : 0.88;
  zoom.value = Math.max(0.3, Math.min(3.5, zoom.value * delta));
}

function handleTouchStart(e: TouchEvent) {
  if (e.touches.length === 1) {
    isDraggingCanvas = true;
    dragStartX = e.touches[0].clientX;
    dragStartY = e.touches[0].clientY;
    initialPanX = panX.value;
    initialPanY = panY.value;
  } else if (e.touches.length === 2) {
    isDraggingCanvas = false;
    const dx = e.touches[0].clientX - e.touches[1].clientX;
    const dy = e.touches[0].clientY - e.touches[1].clientY;
    initialTouchDistance = Math.sqrt(dx * dx + dy * dy);
    initialZoom = zoom.value;
  }
}

function handleTouchMove(e: TouchEvent) {
  if (e.touches.length === 1 && isDraggingCanvas) {
    panX.value = initialPanX + (e.touches[0].clientX - dragStartX);
    panY.value = initialPanY + (e.touches[0].clientY - dragStartY);
  } else if (e.touches.length === 2 && initialTouchDistance > 0) {
    const dx = e.touches[0].clientX - e.touches[1].clientX;
    const dy = e.touches[0].clientY - e.touches[1].clientY;
    const currentDist = Math.sqrt(dx * dx + dy * dy);
    const factor = currentDist / initialTouchDistance;
    zoom.value = Math.max(0.3, Math.min(3.5, initialZoom * factor));
  }
}

function handleTouchEnd(e: TouchEvent) {
  if (e.touches.length === 0) {
    isDraggingCanvas = false;
  }
}

function handleCanvasClick(e: MouseEvent) {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  const mx = e.clientX - rect.left;
  const my = e.clientY - rect.top;

  const node = getNodeAtPoint(mx, my);
  selectedNode.value = node;
  if (node) {
    vibrateSelection();
  }
  wakePhysics();
}

function handleCanvasDblClick(e: MouseEvent) {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  const mx = e.clientX - rect.left;
  const my = e.clientY - rect.top;

  const node = getNodeAtPoint(mx, my);
  if (node) {
    vibrateSuccess();
    openStoryNode(node);
  }
}

function closeInspector() {
  selectedNode.value = null;
}

function openStoryNode(node: GraphNodePacked) {
  const match = stories.find((s) => s.id === node.id);
  if (match) {
    emit("select", match);
  }
}

async function handleCreateRelation() {
  if (!selectedNode.value || !newRelationTargetId.value) return;

  const payload: EntityRelationItem = {
    id: "",
    sourceId: selectedNode.value.id,
    targetId: newRelationTargetId.value,
    relationType: newRelationType.value,
    description: newRelationDesc.value,
  };

  try {
    await invoke("mutate_entity_relation", { relation: payload, operator });
    showCreateRelationModal.value = false;
    newRelationDesc.value = "";
    vibrateSuccess();
    await loadGraphData();
  } catch {}
}

function resetTransform() {
  panX.value = 0;
  panY.value = 0;
  zoom.value = 1.0;
  wakePhysics();
}

watch(
  () => stories,
  () => {
    loadGraphData();
  },
  { deep: true }
);

onMounted(async () => {
  const canvas = canvasRef.value;
  if (canvas && viewportRef.value) {
    canvas.width = viewportRef.value.clientWidth;
    canvas.height = viewportRef.value.clientHeight;

    resizeObserver = new ResizeObserver(() => {
      if (viewportRef.value && canvasRef.value) {
        canvasRef.value.width = viewportRef.value.clientWidth;
        canvasRef.value.height = viewportRef.value.clientHeight;
        wakePhysics();
      }
    });
    resizeObserver.observe(viewportRef.value);
  }

  await loadGraphData();
});

onUnmounted(() => {
  if (animationId) {
    cancelAnimationFrame(animationId);
    animationId = null;
  }
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
});
</script>

<template>
  <div ref="viewportRef" class="stellar-graph-viewport">
    <div class="cosmos-telemetry-hud">
      <div class="hud-search-capsule">
        <input
          v-model="searchQuery"
          class="hud-search-input"
          placeholder="Найти звезду в космосе..."
          @keydown.enter="handleSearchSpotlight"
        />
        <button class="btn-spotlight" @click="handleSearchSpotlight">Телепорт</button>
      </div>

      <div class="hud-buttons-cluster">
        <button class="btn-hud-control" title="Сброс камеры" @click="resetTransform">⊙</button>
        <button class="btn-hud-control" title="Приблизить" @click="zoom = Math.min(3.5, zoom * 1.2)">+</button>
        <button class="btn-hud-control" title="Отдалить" @click="zoom = Math.max(0.3, zoom * 0.8)">−</button>
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
      <span class="minimap-caption">Радар космоса</span>
    </div>

    <aside v-if="selectedNode" class="node-inspector-drawer">
      <div class="inspector-header">
        <div>
          <span class="inspector-badge">{{ selectedNode.category }}</span>
          <h3>{{ selectedNode.label }}</h3>
        </div>
        <button class="btn-close-inspector" @click="closeInspector">✕</button>
      </div>

      <div class="inspector-stats">
        <span>Связей в системе: <b>{{ selectedNode.degree }}</b></span>
        <span>Масса узла: <b>{{ selectedNode.mass.toFixed(1) }}</b></span>
      </div>

      <div class="inspector-actions">
        <button class="btn-open-story" @click="openStoryNode(selectedNode)">Перейти к сцене →</button>
        <button class="btn-add-rel" @click="showCreateRelationModal = true">+ Связать узел</button>
      </div>
    </aside>

    <div v-if="showCreateRelationModal" class="modal-backdrop" @click.self="showCreateRelationModal = false">
      <div class="modal-card">
        <h3>Новая сюжетная связь</h3>
        <p class="modal-sub">Узел: <b>{{ selectedNode?.label }}</b></p>

        <div class="modal-form">
          <select v-model="newRelationTargetId" class="modal-select">
            <option value="">Выберите целевую сущность...</option>
            <option v-for="s in stories.filter((st) => st.id !== selectedNode?.id)" :key="s.id" :value="s.id">
              {{ s.title }} ({{ s.category }})
            </option>
          </select>

          <select v-model="newRelationType" class="modal-select">
            <option value="Союзник">Союзник / Альянс</option>
            <option value="Вражда">Вражда / Конфликт</option>
            <option value="Владение">Владение артефактом</option>
            <option value="Иерархия">Иерархия / Подчинение</option>
          </select>

          <input v-model="newRelationDesc" class="modal-input" placeholder="Описание сути связи..." />
        </div>

        <div class="modal-btn-row">
          <button class="btn-cancel" @click="showCreateRelationModal = false">Отмена</button>
          <button class="btn-submit" @click="handleCreateRelation">Создать</button>
        </div>
      </div>
    </div>
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
  top: 18px;
  right: 18px;
  display: flex;
  gap: 10px;
  align-items: center;
  z-index: 10;
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
  width: 170px;
  outline: none;
}

.btn-spotlight {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  border-radius: 7px;
  padding: 4px 10px;
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
  width: 30px;
  height: 30px;
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
  bottom: 24px;
  right: 24px;
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

.node-inspector-drawer {
  position: absolute;
  top: 18px;
  left: 18px;
  width: 310px;
  background: var(--bg-surface-solid);
  border: 1px solid var(--border-strong);
  border-radius: 16px;
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  z-index: 10;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.9);
}

.inspector-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.inspector-badge {
  font-size: 10px;
  font-weight: 800;
  color: var(--accent-cyan);
  text-transform: uppercase;
}

.inspector-header h3 {
  font-size: 15px;
  font-weight: 800;
  color: var(--text-pure);
  margin-top: 2px;
}

.btn-close-inspector {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 16px;
}

.inspector-stats {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--font-mono);
  background: var(--bg-card);
  padding: 8px 10px;
  border-radius: 8px;
}

.inspector-stats b {
  color: var(--accent-cyan);
}

.inspector-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.btn-open-story {
  background: linear-gradient(135deg, var(--accent-indigo) 0%, #4338ca 100%);
  border: none;
  color: #fff;
  border-radius: 8px;
  padding: 8px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.btn-add-rel {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-pure);
  border-radius: 8px;
  padding: 8px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

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

@media (max-width: 768px) {
  .minimap-container {
    display: none;
  }
}
</style>