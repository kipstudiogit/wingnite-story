// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

import { ref, onMounted, onUnmounted, type Ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  classifyEntityCategory,
  isCodexEntityCategory,
  isEnemyRelationType,
} from "../constants/app";
import type { GraphEdgePacked, GraphNodePacked, GraphTopologyPayload } from "../types";
import { useHaptics } from "./useHaptics";

export function useGraphCanvas(
  projectIdGetter: () => string,
  canvasRef: Ref<HTMLCanvasElement | null>,
  minimapRef: Ref<HTMLCanvasElement | null>,
  viewportRef: Ref<HTMLDivElement | null>,
  onOpenNode: (node: GraphNodePacked) => void
) {
  const { vibrateSelection, vibrateSuccess } = useHaptics();

  const selectedNode = ref<GraphNodePacked | null>(null);
  const filterMode = ref<"all" | "entities">("all");
  const panX = ref(0);
  const panY = ref(0);
  const zoom = ref(1.0);

  let animationId: number | null = null;
  let resizeObserver: ResizeObserver | null = null;

  let rawNodes: GraphNodePacked[] = [];
  let rawEdges: GraphEdgePacked[] = [];
  let positions = new Float32Array(0);
  let velocities = new Float32Array(0);
  let isSimulating = false;
  let pulsePhase = 0;

  let isDraggingCanvas = false;
  let draggedNodeIdx: number | null = null;
  let hasMovedDuringDrag = false;
  let dragStartX = 0;
  let dragStartY = 0;
  let initialPanX = 0;
  let initialPanY = 0;
  let initialTouchDistance = 0;
  let initialZoom = 1.0;

  function isNodeVisible(node: GraphNodePacked): boolean {
    if (filterMode.value === "all") return true;
    return isCodexEntityCategory(node.category);
  }

  function requestRenderLoop() {
    if (!animationId) {
      animationId = requestAnimationFrame(renderFrame);
    }
  }

  function wakePhysics() {
    if (!isSimulating) {
      isSimulating = true;
    }
    requestRenderLoop();
  }

  function setFilterMode(mode: "all" | "entities") {
    filterMode.value = mode;
    if (selectedNode.value && !isNodeVisible(selectedNode.value)) {
      selectedNode.value = null;
    }
    wakePhysics();
  }

  async function loadGraphData() {
    try {
      const payload = await invoke<GraphTopologyPayload>("fetch_packed_topology", {
        projectId: projectIdGetter(),
      });
      rawNodes = payload.nodes;
      rawEdges = payload.edges;

      const count = rawNodes.length;
      positions = new Float32Array(count * 2);
      velocities = new Float32Array(count * 2);

      if (count > 0) {
        for (let i = 0; i < count; i++) {
          const angle = (i / count) * Math.PI * 2;
          const radius = 0.45 + (i % 3) * 0.18;
          positions[i * 2] = Math.cos(angle) * radius;
          positions[i * 2 + 1] = Math.sin(angle) * radius;
        }
      }

      if (selectedNode.value) {
        selectedNode.value = rawNodes.find((n) => n.id === selectedNode.value?.id) ?? null;
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
      if (draggedNodeIdx === i || !isNodeVisible(rawNodes[i])) continue;
      const i2 = i * 2;
      velocities[i2] -= positions[i2] * 0.0003;
      velocities[i2 + 1] -= positions[i2 + 1] * 0.0003;

      for (let j = i + 1; j < count; j++) {
        if (!isNodeVisible(rawNodes[j])) continue;
        const j2 = j * 2;
        const dx = positions[j2] - positions[i2];
        const dy = positions[j2 + 1] - positions[i2 + 1];
        const distSq = dx * dx + dy * dy + 0.001;
        if (distSq < 0.3) {
          const force = 0.00025 / distSq;
          velocities[i2] -= dx * force;
          velocities[i2 + 1] -= dy * force;
          if (draggedNodeIdx !== j) {
            velocities[j2] += dx * force;
            velocities[j2 + 1] += dy * force;
          }
        }
      }
    }

    for (let i = 0; i < rawEdges.length; i++) {
      const e = rawEdges[i];
      const srcNode = rawNodes[e.sourceIdx];
      const tgtNode = rawNodes[e.targetIdx];
      if (!srcNode || !tgtNode || !isNodeVisible(srcNode) || !isNodeVisible(tgtNode)) {
        continue;
      }
      const s2 = e.sourceIdx * 2;
      const t2 = e.targetIdx * 2;
      const dx = positions[t2] - positions[s2];
      const dy = positions[t2 + 1] - positions[s2 + 1];
      const force = 0.0018 * e.weight;
      if (draggedNodeIdx !== e.sourceIdx) {
        velocities[s2] += dx * force;
        velocities[s2 + 1] += dy * force;
      }
      if (draggedNodeIdx !== e.targetIdx) {
        velocities[t2] -= dx * force;
        velocities[t2 + 1] -= dy * force;
      }
    }

    for (let i = 0; i < count; i++) {
      const i2 = i * 2;
      if (draggedNodeIdx === i) {
        velocities[i2] = 0;
        velocities[i2 + 1] = 0;
        continue;
      }
      velocities[i2] *= 0.89;
      velocities[i2 + 1] *= 0.89;
      positions[i2] += velocities[i2];
      positions[i2 + 1] += velocities[i2 + 1];
      totalEnergy += Math.abs(velocities[i2]) + Math.abs(velocities[i2 + 1]);
    }

    return totalEnergy;
  }

  function isOwnershipRelation(relType: string): boolean {
    const lower = relType.toLowerCase();
    return (
      lower.includes("владен") ||
      lower.includes("owner") ||
      lower.includes("besitz") ||
      lower.includes("posesión")
    );
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
      const srcNode = rawNodes[e.sourceIdx];
      const tgtNode = rawNodes[e.targetIdx];
      if (!srcNode || !tgtNode || !isNodeVisible(srcNode) || !isNodeVisible(tgtNode)) {
        continue;
      }
      const sx = cx + positions[e.sourceIdx * 2] * scale;
      const sy = cy + positions[e.sourceIdx * 2 + 1] * scale;
      const tx = cx + positions[e.targetIdx * 2] * scale;
      const ty = cy + positions[e.targetIdx * 2 + 1] * scale;

      const isConflict = isEnemyRelationType(e.relationType);
      const isPossess = isOwnershipRelation(e.relationType);

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
      if (!isNodeVisible(node)) continue;

      const x = cx + positions[i * 2] * scale;
      const y = cy + positions[i * 2 + 1] * scale;
      const baseRadius = (node.isDivine ? 10 : 6.5) * zoom.value;
      const isSelected = selectedNode.value?.id === node.id;

      if (node.isDivine || isSelected) {
        const glowGrad = ctx.createRadialGradient(x, y, 0, x, y, baseRadius * 3.5);
        glowGrad.addColorStop(
          0,
          node.isDivine ? "rgba(245, 158, 11, 0.5)" : "rgba(6, 182, 212, 0.5)"
        );
        glowGrad.addColorStop(1, "transparent");
        ctx.fillStyle = glowGrad;
        ctx.beginPath();
        ctx.arc(x, y, baseRadius * 3.5, 0, Math.PI * 2);
        ctx.fill();
      }

      const entityKind = classifyEntityCategory(node.category);
      ctx.fillStyle = node.isDivine
        ? "#f59e0b"
        : entityKind === "character"
        ? "#38bdf8"
        : "#a855f7";
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
      if (!isNodeVisible(rawNodes[i])) continue;
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

  function renderFrame() {
    if (isSimulating || draggedNodeIdx !== null) {
      const energy = updatePhysicsStep();
      if (energy < 0.004 && draggedNodeIdx === null) {
        isSimulating = false;
      }
    }

    pulsePhase = (pulsePhase + 0.04) % (Math.PI * 2);

    renderStellarCosmos();
    renderMinimap();

    animationId = requestAnimationFrame(renderFrame);
  }

  function getNodeIndexAtPoint(mx: number, my: number): number | null {
    const canvas = canvasRef.value;
    if (!canvas) return null;
    const cx = canvas.width / 2 + panX.value;
    const cy = canvas.height / 2 + panY.value;
    const scale = Math.min(canvas.width, canvas.height) * 0.45 * zoom.value;

    for (let i = 0; i < rawNodes.length; i++) {
      if (!isNodeVisible(rawNodes[i])) continue;
      const x = cx + positions[i * 2] * scale;
      const y = cy + positions[i * 2 + 1] * scale;
      const dx = x - mx;
      const dy = y - my;
      const hitRadius = Math.max(18, 14 * zoom.value);
      if (dx * dx + dy * dy < hitRadius * hitRadius) {
        return i;
      }
    }
    return null;
  }

  function teleportToLabel(query: string) {
    const q = query.toLowerCase().trim();
    if (!q) return;
    const matchIdx = rawNodes.findIndex(
      (n) => isNodeVisible(n) && n.label.toLowerCase().includes(q)
    );
    if (matchIdx !== -1 && canvasRef.value) {
      const scale = Math.min(canvasRef.value.width, canvasRef.value.height) * 0.45 * zoom.value;
      panX.value = -positions[matchIdx * 2] * scale;
      panY.value = -positions[matchIdx * 2 + 1] * scale;
      selectedNode.value = rawNodes[matchIdx];
      vibrateSelection();
      requestRenderLoop();
    }
  }

  function moveDraggedNode(clientX: number, clientY: number) {
    if (draggedNodeIdx === null || !canvasRef.value) return;
    const rect = canvasRef.value.getBoundingClientRect();
    const mx = clientX - rect.left;
    const my = clientY - rect.top;
    const cx = canvasRef.value.width / 2 + panX.value;
    const cy = canvasRef.value.height / 2 + panY.value;
    const scale = Math.min(canvasRef.value.width, canvasRef.value.height) * 0.45 * zoom.value;
    if (scale > 0) {
      positions[draggedNodeIdx * 2] = (mx - cx) / scale;
      positions[draggedNodeIdx * 2 + 1] = (my - cy) / scale;
    }
  }

  function handleMouseDown(e: MouseEvent) {
    const canvas = canvasRef.value;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const hitIdx = getNodeIndexAtPoint(e.clientX - rect.left, e.clientY - rect.top);
    hasMovedDuringDrag = false;
    dragStartX = e.clientX;
    dragStartY = e.clientY;

    if (hitIdx !== null) {
      draggedNodeIdx = hitIdx;
      isDraggingCanvas = false;
      wakePhysics();
    } else {
      draggedNodeIdx = null;
      isDraggingCanvas = true;
      initialPanX = panX.value;
      initialPanY = panY.value;
    }
  }

  function handleMouseMove(e: MouseEvent) {
    const dx = e.clientX - dragStartX;
    const dy = e.clientY - dragStartY;
    if (Math.hypot(dx, dy) > 5) {
      hasMovedDuringDrag = true;
    }
    if (draggedNodeIdx !== null) {
      moveDraggedNode(e.clientX, e.clientY);
      return;
    }
    if (!isDraggingCanvas) return;
    panX.value = initialPanX + dx;
    panY.value = initialPanY + dy;
  }

  function handleMouseUp() {
    draggedNodeIdx = null;
    isDraggingCanvas = false;
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    const delta = e.deltaY < 0 ? 1.12 : 0.88;
    zoom.value = Math.max(0.3, Math.min(3.5, zoom.value * delta));
  }

  function handleTouchStart(e: TouchEvent) {
    if (e.touches.length === 1 && canvasRef.value) {
      const rect = canvasRef.value.getBoundingClientRect();
      const hitIdx = getNodeIndexAtPoint(
        e.touches[0].clientX - rect.left,
        e.touches[0].clientY - rect.top
      );
      hasMovedDuringDrag = false;
      dragStartX = e.touches[0].clientX;
      dragStartY = e.touches[0].clientY;
      if (hitIdx !== null) {
        draggedNodeIdx = hitIdx;
        isDraggingCanvas = false;
        wakePhysics();
      } else {
        draggedNodeIdx = null;
        isDraggingCanvas = true;
        initialPanX = panX.value;
        initialPanY = panY.value;
      }
    } else if (e.touches.length === 2) {
      draggedNodeIdx = null;
      isDraggingCanvas = false;
      const dx = e.touches[0].clientX - e.touches[1].clientX;
      const dy = e.touches[0].clientY - e.touches[1].clientY;
      initialTouchDistance = Math.sqrt(dx * dx + dy * dy);
      initialZoom = zoom.value;
    }
  }

  function handleTouchMove(e: TouchEvent) {
    if (e.touches.length === 1) {
      const dx = e.touches[0].clientX - dragStartX;
      const dy = e.touches[0].clientY - dragStartY;
      if (Math.hypot(dx, dy) > 5) {
        hasMovedDuringDrag = true;
      }
      if (draggedNodeIdx !== null) {
        moveDraggedNode(e.touches[0].clientX, e.touches[0].clientY);
        return;
      }
      if (isDraggingCanvas) {
        panX.value = initialPanX + dx;
        panY.value = initialPanY + dy;
      }
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
      draggedNodeIdx = null;
      isDraggingCanvas = false;
    }
  }

  function handleCanvasClick(e: MouseEvent) {
    if (hasMovedDuringDrag) {
      hasMovedDuringDrag = false;
      return;
    }
    const canvas = canvasRef.value;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const hitIdx = getNodeIndexAtPoint(e.clientX - rect.left, e.clientY - rect.top);
    const node = hitIdx !== null ? rawNodes[hitIdx] : null;
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
    const hitIdx = getNodeIndexAtPoint(e.clientX - rect.left, e.clientY - rect.top);
    if (hitIdx !== null) {
      vibrateSuccess();
      onOpenNode(rawNodes[hitIdx]);
    }
  }

  function resetTransform() {
    panX.value = 0;
    panY.value = 0;
    zoom.value = 1.0;
    wakePhysics();
  }

  function zoomIn() {
    zoom.value = Math.min(3.5, zoom.value * 1.2);
  }

  function zoomOut() {
    zoom.value = Math.max(0.3, zoom.value * 0.8);
  }

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

  return {
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
  };
}