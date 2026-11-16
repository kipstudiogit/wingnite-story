<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { APP_DISPLAY_VERSION, APP_LICENSE, APP_PUBLISHER } from "../constants/app";
import { useAppSettings } from "../composables/useAppSettings";
import { useHaptics } from "../composables/useHaptics";
import { useI18n, type LocaleCode } from "../i18n";
import type { DbStatsPayload } from "../types";

const emit = defineEmits<{
  (e: "notification", msg: string, kind: "success" | "error"): void;
}>();

const { settings, resetToDefaults } = useAppSettings();
const { vibrateSuccess, vibrateLight } = useHaptics();
const { t, messages, locale, availableLocales, setLocale } = useI18n();

const dbStats = ref<DbStatsPayload | null>(null);
const isOptimizing = ref(false);

async function loadDbStats() {
  try {
    dbStats.value = await invoke<DbStatsPayload>("app_get_db_stats");
  } catch {
  }
}

async function handleWalCheckpoint() {
  isOptimizing.value = true;
  try {
    await invoke("app_checkpoint_wal");
    vibrateSuccess();
    emit("notification", t("settings.database.walDone"), "success");
    await loadDbStats();
  } catch (err) {
    emit("notification", t("settings.database.walError", { error: String(err) }), "error");
  } finally {
    isOptimizing.value = false;
  }
}

async function handleVacuumDb() {
  isOptimizing.value = true;
  try {
    await invoke("app_vacuum_db");
    vibrateSuccess();
    emit("notification", t("settings.database.vacuumDone"), "success");
    await loadDbStats();
  } catch (err) {
    emit("notification", t("settings.database.vacuumError", { error: String(err) }), "error");
  } finally {
    isOptimizing.value = false;
  }
}

function formatBytes(bytes: number) {
  const sizes = messages.value.settings.database.bytesUnits;
  if (bytes === 0) return `0 ${sizes[0]}`;
  const k = 1024;
  const i = Math.min(sizes.length - 1, Math.floor(Math.log(bytes) / Math.log(k)));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
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
          <h2>{{ t("settings.title") }}</h2>
          <p>{{ t("settings.subtitle") }}</p>
        </div>
        <button class="btn-reset-settings" @click="resetToDefaults(); vibrateLight()">
          {{ t("settings.resetBtn") }}
        </button>
      </header>

      <div class="settings-sections-stack">
        <div class="settings-card">
          <div class="card-head">
            <span class="card-icon">🌐</span>
            <div>
              <h3>{{ t("settings.localization.title") }}</h3>
              <p>{{ t("settings.localization.subtitle") }}</p>
            </div>
          </div>

          <div class="settings-grid">
            <div class="setting-item">
              <label>{{ t("settings.localization.languageLabel") }}</label>
              <select
                class="setting-select"
                :value="locale"
                @change="setLocale(($event.target as HTMLSelectElement).value as LocaleCode); vibrateLight()"
              >
                <option v-for="loc in availableLocales" :key="loc.code" :value="loc.code">
                  {{ loc.nativeName }} ({{ loc.name }})
                </option>
              </select>
            </div>
          </div>
        </div>

        <div class="settings-card">
          <div class="card-head">
            <span class="card-icon">✎</span>
            <div>
              <h3>{{ t("settings.typography.title") }}</h3>
              <p>{{ t("settings.typography.subtitle") }}</p>
            </div>
          </div>

          <div class="settings-grid">
            <div class="setting-item">
              <label>{{ t("settings.typography.fontLabel") }}</label>
              <select v-model="settings.editorFontFamily" class="setting-select">
                <option value="var(--font-sans)">{{ t("settings.typography.fonts.sans") }}</option>
                <option value="var(--font-mono)">{{ t("settings.typography.fonts.mono") }}</option>
                <option value="var(--font-serif)">{{ t("settings.typography.fonts.serif") }}</option>
              </select>
            </div>

            <div class="setting-item">
              <label>{{ t("settings.typography.lineHeightLabel") }}</label>
              <select v-model.number="settings.editorLineHeight" class="setting-select">
                <option :value="1.6">{{ t("settings.typography.lineHeights.compact") }}</option>
                <option :value="1.85">{{ t("settings.typography.lineHeights.standard") }}</option>
                <option :value="2.1">{{ t("settings.typography.lineHeights.loose") }}</option>
              </select>
            </div>

            <div class="setting-item">
              <label>{{ t("settings.typography.canvasWidthLabel") }}</label>
              <select v-model="settings.canvasMaxWidth" class="setting-select">
                <option value="860px">{{ t("settings.typography.canvasWidths.focus") }}</option>
                <option value="1100px">{{ t("settings.typography.canvasWidths.wide") }}</option>
                <option value="100%">{{ t("settings.typography.canvasWidths.full") }}</option>
              </select>
            </div>

            <div class="setting-item checkbox-item">
              <label class="toggle-switch">
                <input v-model="settings.smoothCaret" type="checkbox" />
                <span class="switch-slider"></span>
              </label>
              <span>{{ t("settings.typography.smoothCaretLabel") }}</span>
            </div>
          </div>
        </div>

        <div class="settings-card">
          <div class="card-head">
            <span class="card-icon">🗄️</span>
            <div>
              <h3>{{ t("settings.database.title") }}</h3>
              <p>{{ t("settings.database.subtitle") }}</p>
            </div>
          </div>

          <div v-if="dbStats" class="db-telemetry-row">
            <div class="db-badge">{{ t("settings.database.scenes") }} <b>{{ dbStats.totalStories }}</b></div>
            <div class="db-badge">{{ t("settings.database.projects") }} <b>{{ dbStats.totalProjects }}</b></div>
            <div class="db-badge">{{ t("settings.database.relations") }} <b>{{ dbStats.totalRelations }}</b></div>
            <div class="db-badge">{{ t("settings.database.events") }} <b>{{ dbStats.totalTimelineEvents }}</b></div>
            <div class="db-badge">{{ t("settings.database.size") }} <b>{{ formatBytes(dbStats.dbSizeBytes) }}</b></div>
          </div>

          <div class="settings-actions-cluster">
            <button
              class="btn-setting-action"
              :disabled="isOptimizing"
              @click="handleWalCheckpoint"
            >
              {{ t("settings.database.walBtn") }}
            </button>
            <button
              class="btn-setting-action"
              :disabled="isOptimizing"
              @click="handleVacuumDb"
            >
              {{ t("settings.database.vacuumBtn") }}
            </button>
          </div>

          <div class="setting-item mt-3">
            <label>{{ t("settings.database.autoSaveLabel") }}</label>
            <select v-model.number="settings.autoSaveIntervalMs" class="setting-select">
              <option :value="1000">{{ t("settings.database.autoSaveOptions.instant") }}</option>
              <option :value="2500">{{ t("settings.database.autoSaveOptions.optimal") }}</option>
              <option :value="5000">{{ t("settings.database.autoSaveOptions.relaxed") }}</option>
              <option :value="0">{{ t("settings.database.autoSaveOptions.manual") }}</option>
            </select>
          </div>
        </div>

        <div class="settings-card">
          <div class="card-head">
            <span class="card-icon">◈</span>
            <div>
              <h3>{{ t("settings.loreSection.title") }}</h3>
              <p>{{ t("settings.loreSection.subtitle") }}</p>
            </div>
          </div>

          <div class="settings-grid">
            <div class="setting-item">
              <label>{{ t("settings.loreSection.sensitivityLabel") }}</label>
              <select v-model="settings.loreSensitivity" class="setting-select">
                <option value="STRICT">{{ t("settings.loreSection.sensitivities.strict") }}</option>
                <option value="BALANCED">{{ t("settings.loreSection.sensitivities.balanced") }}</option>
                <option value="PERMISSIVE">{{ t("settings.loreSection.sensitivities.permissive") }}</option>
              </select>
            </div>

            <div class="setting-item checkbox-item">
              <label class="toggle-switch">
                <input v-model="settings.loreAuditOnInput" type="checkbox" />
                <span class="switch-slider"></span>
              </label>
              <span>{{ t("settings.loreSection.auditOnInputLabel") }}</span>
            </div>
          </div>
        </div>

        <div class="settings-card">
          <div class="card-head">
            <span class="card-icon">⚡</span>
            <div>
              <h3>{{ t("settings.productivity.title") }}</h3>
              <p>{{ t("settings.productivity.subtitle") }}</p>
            </div>
          </div>

          <div class="settings-grid">
            <div class="setting-item">
              <label>{{ t("settings.productivity.dailyGoalLabel") }}</label>
              <input v-model.number="settings.dailyWordGoal" type="number" min="100" class="setting-input" />
            </div>

            <div class="setting-item">
              <label>{{ t("settings.productivity.bookGoalLabel") }}</label>
              <input v-model.number="settings.bookWordGoal" type="number" min="1000" class="setting-input" />
            </div>

            <div class="setting-item">
              <label>{{ t("settings.productivity.sprintDurationLabel") }}</label>
              <input v-model.number="settings.sprintDurationMinutes" type="number" min="1" class="setting-input" />
            </div>

            <div class="setting-item checkbox-item">
              <label class="toggle-switch">
                <input v-model="settings.hapticsEnabled" type="checkbox" />
                <span class="switch-slider"></span>
              </label>
              <span>{{ t("settings.productivity.hapticsLabel") }}</span>
            </div>
          </div>
        </div>

        <div class="settings-card">
          <div class="card-head">
            <span class="card-icon">❖</span>
            <div>
              <h3>{{ t("settings.about.title") }}</h3>
              <p>{{ t("settings.about.subtitle") }}</p>
            </div>
          </div>

          <div class="db-telemetry-row">
            <div class="db-badge">{{ t("settings.about.versionLabel") }}: <b>{{ APP_DISPLAY_VERSION }}</b></div>
            <div class="db-badge">{{ t("settings.about.licenseLabel") }}: <b>{{ APP_LICENSE }}</b></div>
            <div class="db-badge">{{ t("settings.about.studioLabel") }}: <b>{{ APP_PUBLISHER }}</b></div>
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
  padding: 24px max(16px, var(--safe-right)) calc(var(--safe-bottom) + 32px) max(16px, var(--safe-left));
  -webkit-overflow-scrolling: touch;
}

.settings-scroller {
  max-width: 860px;
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--border);
  gap: 12px;
  flex-wrap: wrap;
}

.header-titles h2 {
  font-size: clamp(20px, 4vw, 24px);
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
  min-height: 38px;
  border-radius: 8px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}

.settings-sections-stack {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.settings-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 18px;
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
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 260px), 1fr));
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
  min-height: 40px;
  padding: 8px 12px;
  font-size: 13px;
  outline: none;
  width: 100%;
}

.checkbox-item {
  flex-direction: row;
  align-items: center;
  gap: 10px;
  margin-top: 6px;
}

.checkbox-item span {
  font-size: 13px;
  color: var(--text-main);
  font-weight: 600;
}

.toggle-switch {
  position: relative;
  display: inline-block;
  width: 42px;
  height: 24px;
  flex-shrink: 0;
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
  border-radius: 24px;
}

.switch-slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
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
  padding: 10px 16px;
  min-height: 40px;
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