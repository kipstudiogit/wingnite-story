<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";

const { isReady } = defineProps<{
  isReady: boolean;
}>();

const emit = defineEmits<{
  (e: "finish"): void;
}>();

const progress = ref(0);
const statusMessage = ref("Инициализация квантового ядра SQLite WAL...");
const canvasRef = ref<HTMLCanvasElement | null>(null);

let animationId: number | null = null;
let progressInterval: number | null = null;

interface Particle {
  x: number;
  y: number;
  radius: number;
  alpha: number;
  speed: number;
  angle: number;
}

let particles: Particle[] = [];

function initParticles(width: number, height: number) {
  particles = [];
  for (let i = 0; i < 70; i++) {
    particles.push({
      x: width / 2,
      y: height / 2,
      radius: Math.random() * 2 + 0.5,
      alpha: Math.random() * 0.8 + 0.2,
      speed: Math.random() * 2 + 0.4,
      angle: Math.random() * Math.PI * 2,
    });
  }
}

function renderCanvas() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  ctx.clearRect(0, 0, canvas.width, canvas.height);

  const cx = canvas.width / 2;
  const cy = canvas.height / 2;

  for (let i = 0; i < particles.length; i++) {
    const p = particles[i];
    p.x += Math.cos(p.angle) * p.speed;
    p.y += Math.sin(p.angle) * p.speed;

    const dx = p.x - cx;
    const dy = p.y - cy;
    const dist = Math.sqrt(dx * dx + dy * dy);

    if (dist > Math.max(canvas.width, canvas.height) * 0.45) {
      p.x = cx + (Math.random() - 0.5) * 40;
      p.y = cy + (Math.random() - 0.5) * 40;
    }

    ctx.fillStyle = `rgba(6, 182, 212, ${p.alpha})`;
    ctx.shadowColor = "#06b6d4";
    ctx.shadowBlur = 8;
    ctx.beginPath();
    ctx.arc(p.x, p.y, p.radius, 0, Math.PI * 2);
    ctx.fill();
    ctx.shadowBlur = 0;
  }

  animationId = requestAnimationFrame(renderCanvas);
}

function handleProgressTick() {
  if (progress.value < 35) {
    statusMessage.value = "Инициализация квантового ядра SQLite WAL...";
  } else if (progress.value < 70) {
    statusMessage.value = "Синхронизация FTS5 и семантической матрицы...";
  } else if (progress.value < 95) {
    statusMessage.value = "Калибровка хронологии и реляционных связей...";
  } else {
    statusMessage.value = "Студия готова к работе.";
  }

  if (progress.value < 90) {
    progress.value += 2;
  } else if (isReady) {
    progress.value = 100;
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
      statusMessage.value = "Студия готова к работе.";
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
    initParticles(canvas.width, canvas.height);
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
      <div class="sigil-orbital-assembly">
        <div class="orbital-ring outer-ring"></div>
        <div class="orbital-ring inner-ring"></div>
        <div class="sigil-halo-glow"></div>

        <div class="sigil-glyph-wrapper">
          <svg class="sigil-feather" width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M20.24 12.24a6 6 0 0 0-8.49-8.49L5 10.5V19h8.5z"/>
            <line x1="16" y1="8" x2="2" y2="22"/>
            <line x1="17.5" y1="15" x2="9" y2="15"/>
          </svg>
        </div>
      </div>

      <div class="brand-typography">
        <h1 class="brand-prime-title">WINGNITE STORY</h1>
        <div class="brand-subline">
          <span class="subline-bullet">✦</span>
          <span>K. I. P. STUDIO — NARRATIVE WORKBENCH</span>
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
          <span class="status-stream-text">{{ statusMessage }}</span>
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
  background: radial-gradient(circle at 50% 45%, #0b1120 0%, #030509 60%, #010204 100%);
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
  gap: 28px;
  max-width: 480px;
  width: 90%;
}

.sigil-orbital-assembly {
  position: relative;
  width: 120px;
  height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.orbital-ring {
  position: absolute;
  border-radius: 50%;
  border: 1px dashed rgba(6, 182, 212, 0.4);
}

.outer-ring {
  width: 120px;
  height: 120px;
  animation: rotate-cw 22s linear infinite;
}

.inner-ring {
  width: 96px;
  height: 96px;
  border-style: solid;
  border-color: rgba(99, 102, 241, 0.3) transparent;
  animation: rotate-ccw 14s linear infinite;
}

.sigil-halo-glow {
  position: absolute;
  width: 80px;
  height: 80px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(6, 182, 212, 0.45) 0%, rgba(99, 102, 241, 0.25) 50%, transparent 75%);
  filter: blur(14px);
  animation: pulse-glow 3s ease-in-out infinite alternate;
}

.sigil-glyph-wrapper {
  position: relative;
  z-index: 2;
  width: 68px;
  height: 68px;
  border-radius: 18px;
  background: rgba(15, 23, 42, 0.85);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.16);
  box-shadow: inset 0 1px 1px rgba(255, 255, 255, 0.4), 0 12px 32px rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
}

.sigil-feather {
  color: #38bdf8;
  filter: drop-shadow(0 0 10px rgba(56, 189, 248, 0.8));
  animation: feather-breathe 3s ease-in-out infinite alternate;
}

.brand-typography {
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.brand-prime-title {
  font-family: var(--font-sans);
  font-size: 24px;
  font-weight: 900;
  letter-spacing: 0.32em;
  background: linear-gradient(135deg, #ffffff 30%, #38bdf8 80%, #818cf8 100%);
  -webkit-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
  text-shadow: 0 0 30px rgba(56, 189, 248, 0.35);
  margin-left: 0.32em;
}

.brand-subline {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.16em;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
}

.subline-bullet {
  color: var(--accent-gold);
  font-size: 8px;
}

.telemetry-deck {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.telemetry-bar-chassis {
  width: 100%;
  height: 4px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 999px;
  overflow: hidden;
  position: relative;
  box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.6);
}

.telemetry-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #6366f1 0%, #06b6d4 70%, #38bdf8 100%);
  border-radius: 999px;
  position: relative;
  transition: width 0.15s ease-out;
  box-shadow: 0 0 14px rgba(6, 182, 212, 0.8);
}

.bar-sparkle-lead {
  position: absolute;
  right: 0;
  top: -2px;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #ffffff;
  box-shadow: 0 0 10px #ffffff, 0 0 16px #38bdf8;
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
  font-weight: 700;
}

@keyframes rotate-cw {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@keyframes rotate-ccw {
  from { transform: rotate(360deg); }
  to { transform: rotate(0deg); }
}

@keyframes pulse-glow {
  0% { transform: scale(0.9); opacity: 0.5; }
  100% { transform: scale(1.25); opacity: 0.9; }
}

@keyframes feather-breathe {
  0% { transform: translateY(0px) scale(1); }
  100% { transform: translateY(-3px) scale(1.05); }
}
</style>