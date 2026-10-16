<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { StoryItem, EntityRelationItem } from "../types";

const { stories } = defineProps<{
  stories: StoryItem[];
}>();

const emit = defineEmits<{
  (e: "select", story: StoryItem): void;
}>();

interface Node {
  id: string;
  label: string;
  category: string;
  x: number;
  y: number;
  vx: number;
  vy: number;
  isDivine: boolean;
  pulsePhase: number;
  story: StoryItem;
}

interface EdgeRef {
  id: string;
  sourceNode: Node;
  targetNode: Node;
  relationType: string;
  description: string;
  flowOffset: number;
}

interface StarParticle {
  x: number;
  y: number;
  size: number;
  alpha: number;
}

const viewportRef = ref<HTMLDivElement | null>(null);
const canvasRef = ref<HTMLCanvasElement | null>(null);
const width = ref(1000);
const height = ref(700);
const hoveredNodeId = ref<string | null>(null);
const selectedNode = ref<Node | null>(null);
const rawRelations = ref<EntityRelationItem[]>([]);

const showCreateRelationModal = ref(false);
const newRelationTargetId = ref("");
const newRelationType = ref("Союзник");
const newRelationDesc = ref("");

let animationFrameId: number | null = null;
let resizeObserver: ResizeObserver | null = null;
let nodes: Node[] = [];
let edges: EdgeRef[] = [];
let stars: StarParticle[] = [];
let isSimulating = false;
let globalTime = 0;

function initStars(w: number, h: number) {
  stars = [];
  const count = Math.floor((w * h) / 12000);
  for (let i = 0; i < count; i++) {
    stars.push({
      x: Math.random() * w,
      y: Math.random() * h,
      size: Math.random() * 1.6 + 0.4,
      alpha: Math.random() * 0.7 + 0.2,
    });
  }
}

async function loadRelations() {
  try {
    const list = await invoke<EntityRelationItem[]>("get_entity_relations");
    if (list.length === 0 && stories.length >= 2) {
      await seedDefaultRelations();
    } else {
      rawRelations.value = list;
    }
  } catch {
    rawRelations.value = [];
  }
}

async function seedDefaultRelations() {
  const s0 = stories[0];
  const s1 = stories[1];
  if (!s0 || !s1) return;

  const defaults: Omit<EntityRelationItem, "id">[] = [
    {
      sourceId: s0.id,
      targetId: s1.id,
      relationType: "Владение артефактом",
      description: "Око Бесконечности извлечено в ходе событий первой главы",
    },
  ];

  if (stories.length >= 3) {
    defaults.push({
      sourceId: s1.id,
      targetId: stories[2].id,
      relationType: "Иерархия",
      description: "Подчиняется первородному закону равновесия",
    });
  }

  for (const r of defaults) {
    await invoke<EntityRelationItem>("save_entity_relation", {
      relation: { ...r, id: "" },
    });
  }
  rawRelations.value = await invoke<EntityRelationItem[]>("get_entity_relations");
}

function buildGraph() {
  if (!stories.length) {
    nodes = [];
    edges = [];
    return;
  }

  const cx = width.value / 2;
  const cy = height.value / 2;

  nodes = stories.map((s, i) => {
    const angle = (i / stories.length) * Math.PI * 2;
    const radius = 190 + (i % 3) * 65;
    const isDivine =
      s.category.includes("Демиург") ||
      s.category.includes("Артефакт") ||
      s.category.includes("Реликвия");
    return {
      id: s.id,
      label: s.title,
      category: s.category,
      x: cx + Math.cos(angle) * radius,
      y: cy + Math.sin(angle) * radius,
      vx: 0,
      vy: 0,
      isDivine,
      pulsePhase: Math.random() * Math.PI * 2,
      story: s,
    };
  });

  const nodeMap = new Map<string, Node>();
  nodes.forEach((n) => nodeMap.set(n.id, n));

  edges = [];
  for (const rel of rawRelations.value) {
    const src = nodeMap.get(rel.sourceId);
    const tgt = nodeMap.get(rel.targetId);
    if (src && tgt) {
      edges.push({
        id: rel.id,
        sourceNode: src,
        targetNode: tgt,
        relationType: rel.relationType,
        description: rel.description,
        flowOffset: Math.random(),
      });
    }
  }

  startSimulation();
}

function updatePhysics(): number {
  const cx = width.value / 2;
  const cy = height.value / 2;
  let totalEnergy = 0;

  for (let i = 0; i < nodes.length; i++) {
    const a = nodes[i];
    a.vx += (cx - a.x) * 0.0008;
    a.vy += (cy - a.y) * 0.0008;

    for (let j = i + 1; j < nodes.length; j++) {
      const b = nodes[j];
      const dx = b.x - a.x;
      const dy = b.y - a.y;
      const dist = Math.sqrt(dx * dx + dy * dy) || 1;
      if (dist < 210) {
        const force = ((210 - dist) / dist) * 0.045;
        a.vx -= dx * force;
        a.vy -= dy * force;
        b.vx += dx * force;
        b.vy += dy * force;
      }
    }
  }

  for (let i = 0; i < edges.length; i++) {
    const a = edges[i].sourceNode;
    const b = edges[i].targetNode;
    const dx = b.x - a.x;
    const dy = b.y - a.y;
    const dist = Math.sqrt(dx * dx + dy * dy) || 1;
    const force = (dist - 140) * 0.003;
    a.vx += dx * force;
    a.vy += dy * force;
    b.vx -= dx * force;
    b.vy -= dy * force;
  }

  for (let i = 0; i < nodes.length; i++) {
    const node = nodes[i];
    node.vx *= 0.88;
    node.vy *= 0.88;
    node.x += node.vx;
    node.y += node.vy;
    totalEnergy += Math.abs(node.vx) + Math.abs(node.vy);
  }

  return totalEnergy;
}

function getRelationColor(type: string): string {
  if (type.includes("Вражда") || type.includes("Конфликт")) return "#f43f5e";
  if (type.includes("Владение") || type.includes("Артефакт")) return "#f59e0b";
  if (type.includes("Союзник") || type.includes("Дружба")) return "#10b981";
  return "#06b6d4";
}

function renderFrame() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  const w = width.value;
  const h = height.value;
  const dpr = window.devicePixelRatio || 1;

  ctx.clearRect(0, 0, w * dpr, h * dpr);
  globalTime += 0.02;

  const bgGrad = ctx.createRadialGradient(w / 2, h / 2, 40, w / 2, h / 2, Math.max(w, h) * 0.65);
  bgGrad.addColorStop(0, "rgba(18, 22, 38, 0.45)");
  bgGrad.addColorStop(0.6, "rgba(8, 10, 16, 0.85)");
  bgGrad.addColorStop(1, "rgba(2, 3, 6, 0.98)");
  ctx.fillStyle = bgGrad;
  ctx.fillRect(0, 0, w, h);

  for (let i = 0; i < stars.length; i++) {
    const s = stars[i];
    const twinkle = Math.sin(globalTime * 3 + i) * 0.35 + s.alpha;
    ctx.fillStyle = `rgba(230, 240, 255, ${Math.max(0.1, Math.min(1, twinkle))})`;
    ctx.beginPath();
    ctx.arc(s.x, s.y, s.size, 0, Math.PI * 2);
    ctx.fill();
  }

  const energy = isSimulating ? updatePhysics() : 0;
  if (energy < 0.03 && isSimulating) {
    isSimulating = false;
  }

  for (let i = 0; i < edges.length; i++) {
    const edge = edges[i];
    const a = edge.sourceNode;
    const b = edge.targetNode;
    const isHovered =
      a.id === hoveredNodeId.value ||
      b.id === hoveredNodeId.value ||
      a.id === selectedNode.value?.id ||
      b.id === selectedNode.value?.id;

    const baseColor = getRelationColor(edge.relationType);

    ctx.save();
    if (isHovered) {
      ctx.strokeStyle = baseColor;
      ctx.lineWidth = 2.4;
      ctx.shadowColor = baseColor;
      ctx.shadowBlur = 14;
    } else {
      ctx.strokeStyle = "rgba(255, 255, 255, 0.14)";
      ctx.lineWidth = 1;
      ctx.shadowBlur = 0;
    }

    ctx.beginPath();
    ctx.moveTo(a.x, a.y);
    ctx.lineTo(b.x, b.y);
    ctx.stroke();
    ctx.restore();

    edge.flowOffset = (edge.flowOffset + 0.008) % 1;
    const px = a.x + (b.x - a.x) * edge.flowOffset;
    const py = a.y + (b.y - a.y) * edge.flowOffset;

    ctx.save();
    ctx.fillStyle = baseColor;
    ctx.shadowColor = baseColor;
    ctx.shadowBlur = 10;
    ctx.beginPath();
    ctx.arc(px, py, isHovered ? 2.5 : 1.5, 0, Math.PI * 2);
    ctx.fill();
    ctx.restore();
  }

  for (let i = 0; i < nodes.length; i++) {
    const node = nodes[i];
    const isHovered = node.id === hoveredNodeId.value || node.id === selectedNode.value?.id;
    const baseRadius = node.isDivine ? 12 : 8;
    const radius = isHovered ? baseRadius + 4 : baseRadius;
    const pulse = Math.sin(globalTime * 2 + node.pulsePhase) * 3;

    ctx.save();
    if (node.isDivine) {
      const haloGrad = ctx.createRadialGradient(node.x, node.y, radius * 0.4, node.x, node.y, radius + 14 + pulse);
      haloGrad.addColorStop(0, "rgba(251, 191, 36, 0.6)");
      haloGrad.addColorStop(0.5, "rgba(245, 158, 11, 0.2)");
      haloGrad.addColorStop(1, "rgba(245, 158, 11, 0)");
      ctx.fillStyle = haloGrad;
      ctx.beginPath();
      ctx.arc(node.x, node.y, radius + 14 + pulse, 0, Math.PI * 2);
      ctx.fill();
    } else if (isHovered) {
      const auraGrad = ctx.createRadialGradient(node.x, node.y, radius * 0.5, node.x, node.y, radius + 16);
      auraGrad.addColorStop(0, "rgba(6, 182, 212, 0.5)");
      auraGrad.addColorStop(1, "rgba(99, 102, 241, 0)");
      ctx.fillStyle = auraGrad;
      ctx.beginPath();
      ctx.arc(node.x, node.y, radius + 16, 0, Math.PI * 2);
      ctx.fill();
    }

    const coreGrad = ctx.createRadialGradient(node.x - radius * 0.3, node.y - radius * 0.3, radius * 0.1, node.x, node.y, radius);
    if (node.isDivine) {
      coreGrad.addColorStop(0, "#fffbeb");
      coreGrad.addColorStop(0.8, "#f59e0b");
      coreGrad.addColorStop(1, "#b45309");
      ctx.shadowColor = "#f59e0b";
      ctx.shadowBlur = isHovered ? 24 : 16;
    } else {
      coreGrad.addColorStop(0, "#ffffff");
      coreGrad.addColorStop(0.8, "#06b6d4");
      coreGrad.addColorStop(1, "#4338ca");
      ctx.shadowColor = "#06b6d4";
      ctx.shadowBlur = isHovered ? 20 : 10;
    }

    ctx.fillStyle = coreGrad;
    ctx.beginPath();
    ctx.arc(node.x, node.y, radius, 0, Math.PI * 2);
    ctx.fill();
    ctx.restore();

    ctx.save();
    ctx.font = isHovered ? "bold 13px -apple-system, sans-serif" : "11px -apple-system, sans-serif";
    const textWidth = ctx.measureText(node.label).width;
    const tx = node.x + radius + 10;
    const ty = node.y + 4;

    ctx.fillStyle = "rgba(7, 9, 15, 0.75)";
    ctx.beginPath();
    ctx.roundRect(tx - 6, ty - 13, textWidth + 12, 18, 5);
    ctx.fill();
    ctx.strokeStyle = isHovered ? "rgba(6, 182, 212, 0.8)" : "rgba(255, 255, 255, 0.12)";
    ctx.stroke();

    ctx.fillStyle = isHovered ? "#ffffff" : "rgba(241, 245, 249, 0.88)";
    ctx.fillText(node.label, tx, ty);
    ctx.restore();
  }

  animationFrameId = requestAnimationFrame(renderFrame);
}

function startSimulation() {
  isSimulating = true;
  if (!animationFrameId) {
    animationFrameId = requestAnimationFrame(renderFrame);
  }
}

function syncCanvasScale() {
  const canvas = canvasRef.value;
  if (!canvas || !viewportRef.value) return;

  const rect = viewportRef.value.getBoundingClientRect();
  const dpr = window.devicePixelRatio || 1;

  width.value = rect.width;
  height.value = rect.height;

  canvas.width = rect.width * dpr;
  canvas.height = rect.height * dpr;
  canvas.style.width = `${rect.width}px`;
  canvas.style.height = `${rect.height}px`;

  const ctx = canvas.getContext("2d");
  if (ctx) {
    ctx.scale(dpr, dpr);
  }

  initStars(rect.width, rect.height);
}

function handleMouseMove(e: MouseEvent) {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  const mx = e.clientX - rect.left;
  const my = e.clientY - rect.top;

  let found: string | null = null;
  for (let i = 0; i < nodes.length; i++) {
    const node = nodes[i];
    const dx = node.x - mx;
    const dy = node.y - my;
    const hitRadius = node.isDivine ? 24 : 18;
    if (dx * dx + dy * dy < hitRadius * hitRadius) {
      found = node.id;
      break;
    }
  }

  if (hoveredNodeId.value !== found) {
    hoveredNodeId.value = found;
    canvas.style.cursor = found ? "pointer" : "default";
    startSimulation();
  }
}

function handleClick() {
  if (hoveredNodeId.value) {
    const target = nodes.find((n) => n.id === hoveredNodeId.value);
    if (target) {
      selectedNode.value = target;
    }
  } else {
    selectedNode.value = null;
  }
}

const activeNodeRelations = computed(() => {
  if (!selectedNode.value) return [];
  const id = selectedNode.value.id;
  return edges.filter((e) => e.sourceNode.id === id || e.targetNode.id === id);
});

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
    const saved = await invoke<EntityRelationItem>("save_entity_relation", { relation: payload });
    rawRelations.value.push(saved);
    showCreateRelationModal.value = false;
    newRelationDesc.value = "";
    buildGraph();
  } catch {}
}

async function handleDeleteRelation(relId: string) {
  try {
    await invoke("delete_entity_relation", { id: relId });
    rawRelations.value = rawRelations.value.filter((r) => r.id !== relId);
    buildGraph();
  } catch {}
}

watch(
  () => stories,
  () => {
    buildGraph();
  },
  { deep: true }
);

onMounted(async () => {
  if (viewportRef.value) {
    syncCanvasScale();
    resizeObserver = new ResizeObserver(() => {
      syncCanvasScale();
      startSimulation();
    });
    resizeObserver.observe(viewportRef.value);
  }
  await loadRelations();
  buildGraph();
  animationFrameId = requestAnimationFrame(renderFrame);
});

onUnmounted(() => {
  if (animationFrameId) cancelAnimationFrame(animationFrameId);
  if (resizeObserver) resizeObserver.disconnect();
});
</script>

<template>
  <div ref="viewportRef" class="graph-viewport">
    <div class="graph-legend">
      <div class="legend-item"><span class="legend-dot dot-alliance"></span> Союзник / Дружба</div>
      <div class="legend-item"><span class="legend-dot dot-enmity"></span> Вражда / Конфликт</div>
      <div class="legend-item"><span class="legend-dot dot-relic"></span> Владение артефактом</div>
      <div class="legend-item"><span class="legend-dot dot-hierarchy"></span> Иерархия / Подчинение</div>
    </div>

    <canvas
      ref="canvasRef"
      @mousemove="handleMouseMove"
      @click="handleClick"
    ></canvas>

    <aside v-if="selectedNode" class="node-inspector-drawer">
      <div class="inspector-header">
        <div>
          <span class="inspector-badge">{{ selectedNode.category }}</span>
          <h3>{{ selectedNode.label }}</h3>
        </div>
        <button class="btn-close-inspector" @click="selectedNode = null">✕</button>
      </div>

      <div class="inspector-actions">
        <button class="btn-jump-editor" @click="emit('select', selectedNode.story)">
          ✏️ Открыть в студии
        </button>
        <button class="btn-add-rel" @click="showCreateRelationModal = true">
          + Новая связь
        </button>
      </div>

      <div class="inspector-relations-feed">
        <h4>Семантические связи ({{ activeNodeRelations.length }})</h4>
        <div v-for="edge in activeNodeRelations" :key="edge.id" class="relation-row-card">
          <div class="rel-row-top">
            <span class="rel-type" :style="{ color: getRelationColor(edge.relationType) }">
              {{ edge.relationType }}
            </span>
            <button class="btn-del-rel" @click="handleDeleteRelation(edge.id)">✕</button>
          </div>
          <div class="rel-counterpart">
            {{ edge.sourceNode.id === selectedNode.id ? '→ ' + edge.targetNode.label : '← ' + edge.sourceNode.label }}
          </div>
          <p v-if="edge.description" class="rel-desc">{{ edge.description }}</p>
        </div>
        <div v-if="activeNodeRelations.length === 0" class="rel-empty">Нет зафиксированных связей</div>
      </div>
    </aside>

    <div v-if="showCreateRelationModal" class="modal-backdrop" @click.self="showCreateRelationModal = false">
      <div class="modal-card">
        <h3>Новая сюжетная связь</h3>
        <p class="modal-sub">Связывание узла: <b>{{ selectedNode?.label }}</b></p>

        <div class="modal-form">
          <select v-model="newRelationTargetId" class="modal-select">
            <option value="">Выберите целевую сущность...</option>
            <option
              v-for="s in stories.filter((st) => st.id !== selectedNode?.id)"
              :key="s.id"
              :value="s.id"
            >
              {{ s.title }} ({{ s.category }})
            </option>
          </select>

          <select v-model="newRelationType" class="modal-select">
            <option value="Союзник">Союзник / Дружба</option>
            <option value="Вражда">Вражда / Конфликт</option>
            <option value="Владение артефактом">Владение артефактом</option>
            <option value="Иерархия">Иерархия / Подчинение</option>
          </select>

          <input
            v-model="newRelationDesc"
            class="modal-input"
            placeholder="Описание сути связи..."
          />
        </div>

        <div class="modal-btn-row">
          <button class="btn-cancel" @click="showCreateRelationModal = false">Отмена</button>
          <button class="btn-submit" @click="handleCreateRelation">Создать связь</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.graph-viewport {
  flex: 1;
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
  background: #020306;
}

canvas {
  display: block;
  width: 100%;
  height: 100%;
}

.graph-legend {
  position: absolute;
  top: 24px;
  right: 24px;
  background: rgba(10, 13, 20, 0.7);
  backdrop-filter: blur(28px);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 14px 18px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.6);
  pointer-events: none;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-main);
}

.legend-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.dot-alliance { background: #10b981; box-shadow: 0 0 8px #10b981; }
.dot-enmity { background: #f43f5e; box-shadow: 0 0 8px #f43f5e; }
.dot-relic { background: #f59e0b; box-shadow: 0 0 8px #f59e0b; }
.dot-hierarchy { background: #06b6d4; box-shadow: 0 0 8px #06b6d4; }

.node-inspector-drawer {
  position: absolute;
  top: 24px;
  left: 24px;
  bottom: 24px;
  width: 320px;
  background: var(--bg-surface-solid);
  border: 1px solid var(--border-strong);
  border-radius: 18px;
  box-shadow: var(--shadow-quantum);
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  z-index: 10;
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
  font-size: 17px;
  font-weight: 800;
  margin-top: 2px;
}

.btn-close-inspector {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 16px;
}

.inspector-actions {
  display: flex;
  gap: 8px;
}

.btn-jump-editor {
  flex: 1;
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  border-radius: 10px;
  padding: 8px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.btn-add-rel {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-pure);
  border-radius: 10px;
  padding: 8px 12px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.inspector-relations-feed {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.inspector-relations-feed h4 {
  font-size: 12px;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--text-dim);
}

.relation-row-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.rel-row-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.rel-type {
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
}

.btn-del-rel {
  background: transparent;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  font-size: 12px;
}

.rel-counterpart {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-pure);
}

.rel-desc {
  font-size: 11px;
  color: var(--text-muted);
}

.rel-empty {
  font-size: 12px;
  color: var(--text-dim);
  text-align: center;
  padding: 30px 0;
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
  max-width: 440px;
  background: var(--bg-surface-solid);
  border: 1px solid var(--border-strong);
  border-radius: 18px;
  box-shadow: var(--shadow-quantum);
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
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
  .node-inspector-drawer {
    left: 10px;
    right: 10px;
    bottom: calc(var(--safe-bottom) + 84px);
    width: auto;
    max-height: 40vh;
    top: auto;
  }
}
</style>