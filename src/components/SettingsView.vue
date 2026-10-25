<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useAppSettings } from "../composables/useAppSettings";
import { useHaptics } from "../composables/useHaptics";
import type { DbStatsPayload } from "../types";

const emit = defineEmits<{
  (e: "notification", msg: string): void;
}>();

const { settings, resetToDefaults } = useAppSettings();
const { vibrateSuccess, vibrateLight } = useHaptics();

const dbStats = ref<DbStatsPayload | null>(null);
const isOptimizing = ref(false);

async function loadDbStats() {
  try {
    dbStats.value = await invoke<DbStatsPayload>("app_get_db_stats");
  } catch {}
}

async function handleWalCheckpoint() {
  isOptimizing.value = true;
  try {
    await invoke("app_checkpoint_wal");
    vibrateSuccess();
    emit("notification", "Сброс SQLite WAL выполнен успешно ✓");
    await loadDbStats();
  } catch (err) {
    emit("notification", `Ошибка WAL: ${String(err)}`);
  } finally {
    isOptimizing.value = false;
  }
}

async function handleVacuumDb() {
  isOptimizing.value = true;
  try {
    await invoke("app_vacuum_db");
    vibrateSuccess();
    emit("notification", "Дефрагментация базы данных завершена ✓");
    await loadDbStats();
  } catch (err) {
    emit("notification", `Ошибка вакуумизации: ${String(err)}`);
  } finally {
    isOptimizing.value = false;
  }
}

function formatBytes(bytes: number) {
  if (bytes === 0) return "0 Б";
  const k = 1024;
  const sizes = ["Б", "КБ", "МБ", "ГБ"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

onMounted(() => {
  loadDbStats();
});
</script>

<template>
  <section class="settings-workspace">
    <div class="settings-scroller">
      <header class="settings-header">
        <div class="header-titles">
          <h2>Параметры и Системное Ядро</h2>
          <p>Архитектурная калибровка студии, баз данных и алгоритмов канона</p>
        </div>
        <button class="btn-reset-settings" @click="resetToDefaults(); vibrateLight()">Сброс настроек</button>
      </header>

      <div class="settings-sections-stack">
        <!-- Секция 1: Редактор и шрифты -->
        <div class="settings-card">
          <div class="card-head">
            <span class="card-icon">✎</span>
            <div>
              <h3>Типографика и Холст Редактора</h3>
              <p>Оформление рабочего пространства в Студии</p>
            </div>
          </div>

          <div class="settings-grid">
            <div class="setting-item">
              <label>Семейство шрифта ввода</label>
              <select v-model="settings.editorFontFamily" class="setting-select">
                <option value="var(--font-sans)">SF Pro / Inter (Гротеск)</option>
                <option value="var(--font-mono)">JetBrains Mono (Моноширинный)</option>
                <option value="var(--font-serif)">Literata / Georgia (Антиква)</option>
              </select>
            </div>

            <div class="setting-item">
              <label>Межстрочный интервал</label>
              <select v-model.number="settings.editorLineHeight" class="setting-select">
                <option :value="1.6">1.6 (Плотный)</option>
                <option :value="1.85">1.85 (Стандартный)</option>
                <option :value="2.1">2.1 (Свободный)</option>
              </select>
            </div>

            <div class="setting-item">
              <label>Ширина рабочего холста</label>
              <select v-model="settings.canvasMaxWidth" class="setting-select">
                <option value="860px">860px (Фокус)</option>
                <option value="1100px">1100px (Широкий)</option>
                <option value="100%">100% (На всю ширину экрана)</option>
              </select>
            </div>

            <div class="setting-item checkbox-item">
              <label class="toggle-switch">
                <input v-model="settings.smoothCaret" type="checkbox" />
                <span class="switch-slider"></span>
              </label>
              <span>Плавное ведение каретки ввода</span>
            </div>
          </div>
        </div>

        <!-- Секция 2: SQLite и Хранилище -->
        <div class="settings-card">
          <div class="card-head">
            <span class="card-icon">🗄️</span>
            <div>
              <h3>Хранилище SQLite WAL Core</h3>
              <p>Телеметрия дискового пространства и оптимизация индексов</p>
            </div>
          </div>

          <div v-if="dbStats" class="db-telemetry-row">
            <div class="db-badge">Сцен: <b>{{ dbStats.totalStories }}</b></div>
            <div class="db-badge">Произведений: <b>{{ dbStats.totalProjects }}</b></div>
            <div class="db-badge">Связей: <b>{{ dbStats.totalRelations }}</b></div>
            <div class="db-badge">Размер базы: <b>{{ formatBytes(dbStats.dbSizeBytes) }}</b></div>
          </div>

          <div class="settings-actions-cluster">
            <button
              class="btn-setting-action"
              :disabled="isOptimizing"
              @click="handleWalCheckpoint"
            >
              Сброс WAL на диск (Checkpoint)
            </button>
            <button
              class="btn-setting-action"
              :disabled="isOptimizing"
              @click="handleVacuumDb"
            >
              Дефрагментация и VACUUM
            </button>
          </div>

          <div class="setting-item mt-3">
            <label>Интервал автосохранения черновиков</label>
            <select v-model.number="settings.autoSaveIntervalMs" class="setting-select">
              <option :value="1000">1 секунда (Мгновенно)</option>
              <option :value="2500">2.5 секунды (Оптимально)</option>
              <option :value="5000">5 секунд</option>
              <option :value="0">Только вручную (Ctrl+S)</option>
            </select>
          </div>
        </div>

        <!-- Секция 3: Страж канона -->
        <div class="settings-card">
          <div class="card-head">
            <span class="card-icon">◈</span>
            <div>
              <h3>Нейроматрица Стража Канона</h3>
              <p>Параметры детектора аномалий и противоречий сюжета</p>
            </div>
          </div>

          <div class="settings-grid">
            <div class="setting-item">
              <label>Чувствительность фильтрации</label>
              <select v-model="settings.loreSensitivity" class="setting-select">
                <option value="STRICT">Строгая (Онтология + Граф + Хронология)</option>
                <option value="BALANCED">Сбалансированная (Рекомендуется)</option>
                <option value="PERMISSIVE">Мягкая (Только критические парадоксы)</option>
              </select>
            </div>

            <div class="setting-item checkbox-item">
              <label class="toggle-switch">
                <input v-model="settings.loreAuditOnInput" type="checkbox" />
                <span class="switch-slider"></span>
              </label>
              <span>Фоновый аудит канона при наборе текста</span>
            </div>
          </div>
        </div>

        <!-- Секция 4: Спринт и цели -->
        <div class="settings-card">
          <div class="card-head">
            <span class="card-icon">⚡</span>
            <div>
              <h3>Спринты и Продуктивность</h3>
              <p>Управление писательскими нормами</p>
            </div>
          </div>

          <div class="settings-grid">
            <div class="setting-item">
              <label>Дневная норма слов</label>
              <input v-model.number="settings.dailyWordGoal" type="number" class="setting-input" />
            </div>

            <div class="setting-item">
              <label>Длительность спринта по умолчанию (мин)</label>
              <input v-model.number="settings.sprintDurationMinutes" type="number" class="setting-input" />
            </div>

            <div class="setting-item checkbox-item">
              <label class="toggle-switch">
                <input v-model="settings.hapticsEnabled" type="checkbox" />
                <span class="switch-slider"></span>
              </label>
              <span>Виброотклик и тактильные щелчки (Haptics)</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.settings-workspace {
  flex: 1;
  display: flex;
  justify-content: center;
  overflow-y: auto;
  background: var(--bg-deep);
  padding: 30px 20px calc(var(--safe-bottom) + 80px) 20px;
}

.settings-scroller {
  max-width: 860px;
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 22px;
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border);
}

.header-titles h2 {
  font-size: 24px;
  font-weight: 900;
  color: var(--text-pure);
}

.header-titles p {
  font-size: 13px;
  color: var(--text-muted);
  margin-top: 2px;
}

.btn-reset-settings {
  background: transparent;
  border: 1px solid var(--border);
  color: var(--text-muted);
  padding: 8px 14px;
  border-radius: 8px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}

.settings-sections-stack {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.settings-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.card-head {
  display: flex;
  align-items: center;
  gap: 12px;
}

.card-icon {
  font-size: 20px;
  color: var(--accent-cyan);
}

.card-head h3 {
  font-size: 15px;
  font-weight: 800;
  color: var(--text-pure);
}

.card-head p {
  font-size: 11px;
  color: var(--text-muted);
}

.settings-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 14px;
}

.setting-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.setting-item label {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-dim);
  text-transform: uppercase;
}

.setting-select,
.setting-input {
  background: var(--bg-input);
  border: 1px solid var(--border);
  color: var(--text-pure);
  border-radius: 8px;
  padding: 8px 12px;
  font-size: 13px;
  outline: none;
}

.checkbox-item {
  flex-direction: row;
  align-items: center;
  gap: 10px;
  margin-top: 10px;
}

.checkbox-item span {
  font-size: 13px;
  color: var(--text-main);
  font-weight: 600;
}

.toggle-switch {
  position: relative;
  display: inline-block;
  width: 40px;
  height: 22px;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.switch-slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background-color: var(--border-strong);
  transition: 0.25s;
  border-radius: 22px;
}

.switch-slider:before {
  position: absolute;
  content: "";
  height: 16px;
  width: 16px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.25s;
  border-radius: 50%;
}

input:checked + .switch-slider {
  background-color: var(--accent-indigo);
}

input:checked + .switch-slider:before {
  transform: translateX(18px);
}

.db-telemetry-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.db-badge {
  background: var(--bg-input);
  border: 1px solid var(--border-subtle);
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 11px;
  color: var(--text-muted);
}

.db-badge b {
  color: var(--accent-cyan);
  font-family: var(--font-mono);
}

.settings-actions-cluster {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.btn-setting-action {
  background: var(--bg-input);
  border: 1px solid var(--border);
  color: var(--text-main);
  padding: 8px 16px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-setting-action:hover {
  background: var(--accent-indigo);
  color: #fff;
}

.mt-3 {
  margin-top: 8px;
}
</style>