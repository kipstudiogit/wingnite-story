<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";

const { isReady } = defineProps<{
  isReady: boolean;
}>();

const emit = defineEmits<{
  (e: "finish"): void;
}>();

const progress = ref(0);
const telemetryIndex = ref(0);
const canvasRef = ref<HTMLCanvasElement | null>(null);

const telemetryLogs = [
  "Инициализация квантового ядра SQLite WAL...",
  "Синхронизация FTS5 и семантической матрицы лора...",
  "Калибровка хронологии и тензора напряжения...",
  "Верификация криптографического сейфа Merkle Tree...",
  "Полномочия подтверждены. Студия готова к работе.",
];

let animationId: number | null = null;
let progressInterval: number | null = null;

interface StarParticle {
  x: number;
  y: number;
  z: number;
  baseRadius: number;
  alpha: number;
  speed: number;
  angle: number;
}

let stars: StarParticle[] = [];

function initStarfield(width: number, height: number) {
  stars = [];
  const count = Math.min(100, Math.floor((width * height) / 12000));
  for (let i = 0; i < count; i++) {
    stars.push({
      x: (Math.random() - 0.5) * width,
      y: (Math.random() - 0.5) * height,
      z: Math.random() * 800 + 200,
      baseRadius: Math.random() * 1.8 + 0.6,
      alpha: Math.random() * 0.7 + 0.3,
      speed: Math.random() * 0.8 + 0.3,
      angle: Math.random() * Math.PI * 2,
    });
  }
}

function renderCanvas() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  const w = canvas.width;
  const h = canvas.height;
  const cx = w / 2;
  const cy = h / 2;

  ctx.clearRect(0, 0, w, h);

  const radialGradient = ctx.createRadialGradient(cx, cy, 10, cx, cy, Math.max(w, h) * 0.65);
  radialGradient.addColorStop(0, "rgba(15, 23, 42, 0.45)");
  radialGradient.addColorStop(0.5, "rgba(6, 9, 16, 0.75)");
  radialGradient.addColorStop(1, "rgba(2, 3, 6, 0.98)");
  ctx.fillStyle = radialGradient;
  ctx.fillRect(0, 0, w, h);

  const projectedStars: { px: number; py: number; alpha: number; radius: number }[] = [];

  for (let i = 0; i < stars.length; i++) {
    const s = stars[i];
    s.z -= s.speed * 2.2;
    if (s.z <= 10) {
      s.z = 800;
      s.x = (Math.random() - 0.5) * w;
      s.y = (Math.random() - 0.5) * h;
    }

    const k = 450 / s.z;
    const px = cx + s.x * k;
    const py = cy + s.y * k;

    if (px >= 0 && px <= w && py >= 0 && py <= h) {
      const radius = s.baseRadius * k * 0.8;
      projectedStars.push({ px, py, alpha: s.alpha, radius });

      ctx.beginPath();
      ctx.arc(px, py, radius, 0, Math.PI * 2);
      ctx.fillStyle = `rgba(56, 189, 248, ${s.alpha * Math.min(1, k)})`;
      ctx.shadowColor = "#38bdf8";
      ctx.shadowBlur = 6;
      ctx.fill();
      ctx.shadowBlur = 0;
    }
  }

  ctx.lineWidth = 0.5;
  for (let i = 0; i < projectedStars.length; i++) {
    for (let j = i + 1; j < projectedStars.length; j++) {
      const p1 = projectedStars[i];
      const p2 = projectedStars[j];
      const dx = p1.px - p2.px;
      const dy = p1.py - p2.py;
      const dist = Math.sqrt(dx * dx + dy * dy);

      if (dist < 85) {
        const lineAlpha = (1 - dist / 85) * 0.18 * Math.min(p1.alpha, p2.alpha);
        ctx.strokeStyle = `rgba(99, 102, 241, ${lineAlpha})`;
        ctx.beginPath();
        ctx.moveTo(p1.px, p1.py);
        ctx.lineTo(p2.px, p2.py);
        ctx.stroke();
      }
    }
  }

  animationId = requestAnimationFrame(renderCanvas);
}

function handleProgressTick() {
  if (progress.value < 25) {
    telemetryIndex.value = 0;
  } else if (progress.value < 50) {
    telemetryIndex.value = 1;
  } else if (progress.value < 75) {
    telemetryIndex.value = 2;
  } else if (progress.value < 95) {
    telemetryIndex.value = 3;
  } else {
    telemetryIndex.value = 4;
  }

  if (progress.value < 90) {
    progress.value += 2;
  } else if (isReady) {
    progress.value = 100;
    telemetryIndex.value = 4;
    if (progressInterval) clearInterval(progressInterval);
    setTimeout(() => {
      emit("finish");
    }, 450);
  }
}

watch(
  () => isReady,
  (ready) => {
    if (ready && progress.value >= 90) {
      progress.value = 100;
      telemetryIndex.value = 4;
      if (progressInterval) clearInterval(progressInterval);
      setTimeout(() => {
        emit("finish");
      }, 450);
    }
  }
);

onMounted(() => {
  const canvas = canvasRef.value;
  if (canvas) {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    initStarfield(canvas.width, canvas.height);
    renderCanvas();
  }
  progressInterval = window.setInterval(handleProgressTick, 35);
});

onUnmounted(() => {
  if (animationId) cancelAnimationFrame(animationId);
  if (progressInterval) clearInterval(progressInterval);
});
</script>

<template>
  <div class="splash-curtain">
    <canvas ref="canvasRef" class="splash-particle-canvas"></canvas>

    <div class="splash-central-core">
      <div class="armillary-sphere-assembly">
        <div class="gyro-ring ring-latitude"></div>
        <div class="gyro-ring ring-longitude"></div>
        <div class="gyro-ring ring-oblique"></div>
        <div class="singularity-glow"></div>

        <div class="sigil-capsule">
          <svg class="sigil-feather" width="52" height="52" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M20.24 12.24a6 6 0 0 0-8.49-8.49L5 10.5V19h8.5z"/>
            <line x1="16" y1="8" x2="2" y2="22"/>
            <line x1="17.5" y1="15" x2="9" y2="15"/>
          </svg>
        </div>
      </div>

      <div class="brand-monolith-block">
        <h1 class="brand-prime-title">WINGNITE STORY</h1>
        <div class="brand-subline">
          <span class="subline-bullet">✦</span>
          <span>NARRATIVE WORKBENCH ENTERPRISE</span>
          <span class="subline-bullet">✦</span>
        </div>
      </div>

      <div class="telemetry-deck">
        <div class="telemetry-bar-chassis">
          <div class="telemetry-bar-fill" :style="{ width: `${progress}%` }">
            <span class="bar-sparkle-lead"></span>
          </div>
        </div>

        <div class="telemetry-status-row">
          <span class="status-stream-text">{{ telemetryLogs[telemetryIndex] }}</span>
          <span class="status-counter-text">{{ progress }}%</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.splash-curtain {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: #020306;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  user-select: none;
}

.splash-particle-canvas {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.splash-central-core {
  position: relative;
  z-index: 10;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 32px;
  max-width: 500px;
  width: 90%;
}

.armillary-sphere-assembly {
  position: relative;
  width: 130px;
  height: 130px;
  display: flex;
  align-items: center;
  justify-content: center;
  perspective: 800px;
}

.gyro-ring {
  position: absolute;
  border-radius: 50%;
  border: 1px dashed rgba(56, 189, 248, 0.45);
  box-shadow: 0 0 14px rgba(6, 182, 212, 0.2);
}

.ring-latitude {
  width: 130px;
  height: 130px;
  animation: gyro-rotate-x 16s linear infinite;
}

.ring-longitude {
  width: 114px;
  height: 114px;
  border-style: solid;
  border-color: rgba(99, 102, 241, 0.5) transparent;
  animation: gyro-rotate-y 11s linear infinite;
}

.ring-oblique {
  width: 98px;
  height: 98px;
  border-style: dashed;
  border-color: rgba(245, 158, 11, 0.45);
  animation: gyro-rotate-z 14s linear infinite reverse;
}

.singularity-glow {
  position: absolute;
  width: 86px;
  height: 86px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(56, 189, 248, 0.5) 0%, rgba(99, 102, 241, 0.25) 50%, transparent 75%);
  filter: blur(16px);
  animation: pulse-singularity 2.8s ease-in-out infinite alternate;
}

.sigil-capsule {
  position: relative;
  z-index: 3;
  width: 72px;
  height: 72px;
  border-radius: 20px;
  background: rgba(10, 15, 30, 0.88);
  backdrop-filter: blur(24px);
  border: 1px solid rgba(255, 255, 255, 0.2);
  box-shadow: inset 0 1px 2px rgba(255, 255, 255, 0.4), 0 14px 40px rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
}

.sigil-feather {
  color: #38bdf8;
  filter: drop-shadow(0 0 12px rgba(56, 189, 248, 0.85));
  animation: feather-levitate 3s ease-in-out infinite alternate;
}

.brand-monolith-block {
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.brand-prime-title {
  font-family: var(--font-sans);
  font-size: 26px;
  font-weight: 900;
  letter-spacing: 0.35em;
  background: linear-gradient(135deg, #ffffff 25%, #38bdf8 75%, #818cf8 100%);
  -webkit-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
  text-shadow: 0 0 35px rgba(56, 189, 248, 0.4);
  margin-left: 0.35em;
}

.brand-subline {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.2em;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.subline-bullet {
  color: var(--accent-gold);
  font-size: 8px;
}

.telemetry-deck {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.telemetry-bar-chassis {
  width: 100%;
  height: 5px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 999px;
  overflow: hidden;
  position: relative;
  box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.7);
}

.telemetry-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #6366f1 0%, #06b6d4 65%, #38bdf8 100%);
  border-radius: 999px;
  position: relative;
  transition: width 0.15s ease-out;
  box-shadow: 0 0 16px rgba(6, 182, 212, 0.9);
}

.bar-sparkle-lead {
  position: absolute;
  right: 0;
  top: -2.5px;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #ffffff;
  box-shadow: 0 0 12px #ffffff, 0 0 20px #38bdf8;
}

.telemetry-status-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-family: var(--font-mono);
  font-size: 11px;
}

.status-stream-text {
  color: var(--text-muted);
}

.status-counter-text {
  color: #38bdf8;
  font-weight: 800;
}

@keyframes gyro-rotate-x {
  from { transform: rotate3d(1, 0, 0, 0deg); }
  to { transform: rotate3d(1, 0, 0, 360deg); }
}

@keyframes gyro-rotate-y {
  from { transform: rotate3d(0, 1, 0, 0deg); }
  to { transform: rotate3d(0, 1, 0, 360deg); }
}

@keyframes gyro-rotate-z {
  from { transform: rotate3d(1, 1, 1, 0deg); }
  to { transform: rotate3d(1, 1, 1, 360deg); }
}

@keyframes pulse-singularity {
  0% { transform: scale(0.85); opacity: 0.5; }
  100% { transform: scale(1.3); opacity: 0.95; }
}

@keyframes feather-levitate {
  0% { transform: translateY(0px) scale(1); }
  100% { transform: translateY(-4px) scale(1.06); }
}
</style>