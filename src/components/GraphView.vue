<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { EnterpriseStoryItem, EntityRelationItem } from "../types";

interface GraphNodePacked {
  id: string;
  index: number;
  label: string;
  category: string;
  isDivine: boolean;
  mass: number;
}

interface GraphEdgePacked {
  sourceIdx: number;
  targetIdx: number;
  relationType: string;
  weight: number;
}

interface GraphTopologyPayload {
  nodes: GraphNodePacked[];
  edges: GraphEdgePacked[];
  revision: number;
}

const GPU_BUFFER_USAGE_VERTEX = 0x0020;
const GPU_BUFFER_USAGE_COPY_DST = 0x0008;

const { stories, operator = "Cosmos-Architect" } = defineProps<{
  stories: EnterpriseStoryItem[];
  operator?: string;
}>();

const emit = defineEmits<{
  (e: "select", story: EnterpriseStoryItem): void;
}>();

const canvasRef = ref<HTMLCanvasElement | null>(null);
const viewportRef = ref<HTMLDivElement | null>(null);
const selectedNode = ref<GraphNodePacked | null>(null);

const showCreateRelationModal = ref(false);
const newRelationTargetId = ref("");
const newRelationType = ref("Союзник");
const newRelationDesc = ref("");

let animationId: number | null = null;
let resizeObserver: ResizeObserver | null = null;

let rawNodes: GraphNodePacked[] = [];
let rawEdges: GraphEdgePacked[] = [];

let gpuDevice: GPUDevice | null = null;
let gpuContext: GPUCanvasContext | null = null;
let renderPipeline: GPURenderPipeline | null = null;
let vertexBuffer: GPUBuffer | null = null;

let positions: Float32Array = new Float32Array(0);
let velocities: Float32Array = new Float32Array(0);
let isSimulating = false;

const WGSL_SHADERS = `
struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) @interpolate(flat) isDivine: f32,
};

@vertex
fn vs_main(
  @location(0) quadPos: vec2<f32>,
  @location(1) nodePos: vec2<f32>,
  @location(2) isDivine: f32,
  @location(3) radius: f32
) -> VertexOutput {
  var out: VertexOutput;
  let worldPos = nodePos + quadPos * radius;
  out.position = vec4<f32>(worldPos, 0.0, 1.0);
  out.uv = quadPos;
  out.isDivine = isDivine;
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let dist = length(in.uv);
  if (dist > 1.0) {
    discard;
  }
  let alpha = 1.0 - smoothstep(0.85, 1.0, dist);
  if (in.isDivine > 0.5) {
    return vec4<f32>(0.96, 0.62, 0.04, alpha);
  }
  return vec4<f32>(0.02, 0.71, 0.83, alpha);
}
`;

async function initWebGpu(): Promise<boolean> {
  const canvas = canvasRef.value;
  if (!canvas || !navigator.gpu) return false;

  try {
    const adapter = await navigator.gpu.requestAdapter();
    if (!adapter) return false;

    gpuDevice = await adapter.requestDevice();
    gpuContext = canvas.getContext("webgpu") as unknown as GPUCanvasContext | null;
    if (!gpuContext) return false;

    const presentationFormat = navigator.gpu.getPreferredCanvasFormat();
    gpuContext.configure({
      device: gpuDevice,
      format: presentationFormat,
      alphaMode: "premultiplied",
    });

    const shaderModule = gpuDevice.createShaderModule({
      code: WGSL_SHADERS,
    });

    renderPipeline = gpuDevice.createRenderPipeline({
      layout: "auto",
      vertex: {
        module: shaderModule,
        entryPoint: "vs_main",
        buffers: [
          {
            arrayStride: 6 * 4,
            stepMode: "instance",
            attributes: [
              { shaderLocation: 1, offset: 0, format: "float32x2" },
              { shaderLocation: 2, offset: 8, format: "float32" },
              { shaderLocation: 3, offset: 12, format: "float32" },
            ],
          },
        ],
      },
      fragment: {
        module: shaderModule,
        entryPoint: "fs_main",
        targets: [
          {
            format: presentationFormat,
            blend: {
              color: {
                srcFactor: "src-alpha",
                dstFactor: "one-minus-src-alpha",
                operation: "add",
              },
              alpha: {
                srcFactor: "one",
                dstFactor: "one-minus-src-alpha",
                operation: "add",
              },
            },
          },
        ],
      },
      primitive: {
        topology: "triangle-strip",
      },
    });

    return true;
  } catch {
    return false;
  }
}

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
      const radius = 0.5 + (i % 3) * 0.15;
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
    velocities[i2] -= positions[i2] * 0.0004;
    velocities[i2 + 1] -= positions[i2 + 1] * 0.0004;

    for (let j = i + 1; j < count; j++) {
      const j2 = j * 2;
      const dx = positions[j2] - positions[i2];
      const dy = positions[j2 + 1] - positions[i2 + 1];
      const distSq = dx * dx + dy * dy + 0.001;
      if (distSq < 0.25) {
        const force = 0.0002 / distSq;
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
    const force = 0.002 * e.weight;
    velocities[s2] += dx * force;
    velocities[s2 + 1] += dy * force;
    velocities[t2] -= dx * force;
    velocities[t2 + 1] -= dy * force;
  }

  for (let i = 0; i < count * 2; i++) {
    velocities[i] *= 0.88;
    positions[i] += velocities[i];
    totalEnergy += Math.abs(velocities[i]);
  }

  return totalEnergy;
}

function renderFrame() {
  if (isSimulating) {
    const energy = updatePhysicsStep();
    if (energy < 0.005) {
      isSimulating = false;
    }
  }

  if (gpuDevice && gpuContext && renderPipeline) {
    renderWebGpuFrame();
  } else {
    renderCanvas2dFallback();
  }

  if (isSimulating) {
    animationId = requestAnimationFrame(renderFrame);
  } else {
    animationId = null;
  }
}

function renderWebGpuFrame() {
  if (!gpuDevice || !gpuContext || !renderPipeline) return;

  const count = rawNodes.length;
  if (count === 0) return;

  const instanceData = new Float32Array(count * 6);
  for (let i = 0; i < count; i++) {
    const idx = i * 6;
    instanceData[idx] = positions[i * 2];
    instanceData[idx + 1] = positions[i * 2 + 1];
    instanceData[idx + 2] = rawNodes[i].isDivine ? 1.0 : 0.0;
    instanceData[idx + 3] = rawNodes[i].isDivine ? 0.04 : 0.025;
  }

  if (!vertexBuffer || vertexBuffer.size < instanceData.byteLength) {
    if (vertexBuffer) vertexBuffer.destroy();
    vertexBuffer = gpuDevice.createBuffer({
      size: Math.max(instanceData.byteLength, 1024),
      usage: GPU_BUFFER_USAGE_VERTEX | GPU_BUFFER_USAGE_COPY_DST,
    });
  }

  gpuDevice.queue.writeBuffer(vertexBuffer, 0, instanceData);

  const commandEncoder = gpuDevice.createCommandEncoder();
  const textureView = gpuContext.getCurrentTexture().createView();

  const renderPass = commandEncoder.beginRenderPass({
    colorAttachments: [
      {
        view: textureView,
        clearValue: { r: 0.01, g: 0.015, b: 0.03, a: 1.0 },
        loadOp: "clear",
        storeOp: "store",
      },
    ],
  });

  renderPass.setPipeline(renderPipeline);
  renderPass.setVertexBuffer(0, vertexBuffer);
  renderPass.draw(4, count, 0, 0);
  renderPass.end();

  gpuDevice.queue.submit([commandEncoder.finish()]);
}

function renderCanvas2dFallback() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  const w = canvas.width;
  const h = canvas.height;
  ctx.clearRect(0, 0, w, h);

  const cx = w / 2;
  const cy = h / 2;
  const scale = Math.min(w, h) * 0.45;

  ctx.strokeStyle = "rgba(255, 255, 255, 0.12)";
  ctx.lineWidth = 1;
  for (let i = 0; i < rawEdges.length; i++) {
    const e = rawEdges[i];
    const sx = cx + positions[e.sourceIdx * 2] * scale;
    const sy = cy + positions[e.sourceIdx * 2 + 1] * scale;
    const tx = cx + positions[e.targetIdx * 2] * scale;
    const ty = cy + positions[e.targetIdx * 2 + 1] * scale;

    ctx.beginPath();
    ctx.moveTo(sx, sy);
    ctx.lineTo(tx, ty);
    ctx.stroke();
  }

  for (let i = 0; i < rawNodes.length; i++) {
    const node = rawNodes[i];
    const x = cx + positions[i * 2] * scale;
    const y = cy + positions[i * 2 + 1] * scale;
    const radius = node.isDivine ? 8 : 5;

    ctx.fillStyle = node.isDivine ? "#f59e0b" : "#06b6d4";
    ctx.beginPath();
    ctx.arc(x, y, radius, 0, Math.PI * 2);
    ctx.fill();

    ctx.fillStyle = "rgba(255, 255, 255, 0.85)";
    ctx.font = "11px sans-serif";
    ctx.fillText(node.label, x + radius + 4, y + 4);
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
  const cx = canvas.width / 2;
  const cy = canvas.height / 2;
  const scale = Math.min(canvas.width, canvas.height) * 0.45;

  for (let i = 0; i < rawNodes.length; i++) {
    const x = cx + positions[i * 2] * scale;
    const y = cy + positions[i * 2 + 1] * scale;
    const dx = x - mx;
    const dy = y - my;
    if (dx * dx + dy * dy < 256) {
      return rawNodes[i];
    }
  }
  return null;
}

function handleCanvasClick(e: MouseEvent) {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  const mx = e.clientX - rect.left;
  const my = e.clientY - rect.top;

  selectedNode.value = getNodeAtPoint(mx, my);
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
    openStoryNode(node);
  }
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
    await loadGraphData();
  } catch {}
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

  await initWebGpu();
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
  if (vertexBuffer) {
    vertexBuffer.destroy();
    vertexBuffer = null;
  }
});
</script>

<template>
  <div ref="viewportRef" class="webgpu-graph-viewport">
    <div class="engine-tag-badge">WebGPU Acceleration Active</div>

    <canvas
      ref="canvasRef"
      @click="handleCanvasClick"
      @dblclick="handleCanvasDblClick"
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
        <button class="btn-open-story" @click="openStoryNode(selectedNode)">
          Перейти к сцене →
        </button>
        <button class="btn-add-rel" @click="showCreateRelationModal = true">
          + Связать узел
        </button>
      </div>
    </aside>

    <div v-if="showCreateRelationModal" class="modal-backdrop" @click.self="showCreateRelationModal = false">
      <div class="modal-card">
        <h3>Новая сюжетная связь</h3>
        <p class="modal-sub">Узел: <b>{{ selectedNode?.label }}</b></p>

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
            <option value="Союзник">Союзник / Альянс</option>
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
          <button class="btn-submit" @click="handleCreateRelation">Создать</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.webgpu-graph-viewport {
  flex: 1;
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
  background: var(--bg-deep);
}

canvas {
  display: block;
  width: 100%;
  height: 100%;
}

.engine-tag-badge {
  position: absolute;
  top: 18px;
  right: 18px;
  background: rgba(0, 0, 0, 0.7);
  border: 1px solid var(--accent-cyan);
  color: var(--accent-cyan);
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 700;
  padding: 4px 10px;
  border-radius: 8px;
  pointer-events: none;
}

.node-inspector-drawer {
  position: absolute;
  top: 18px;
  left: 18px;
  width: 300px;
  background: var(--bg-surface-solid);
  border: 1px solid var(--border-strong);
  border-radius: 14px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
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
  font-size: 15px;
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
  flex-direction: column;
  gap: 8px;
}

.btn-open-story {
  width: 100%;
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
  width: 100%;
  background: rgba(255, 255, 255, 0.05);
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
</style>