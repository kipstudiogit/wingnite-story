<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { marked } from "marked";
import { invoke } from "@tauri-apps/api/core";
import type { StudioViewMode, TextMetricsResult } from "../types";
import { useNarrativeLsp } from "../composables/useNarrativeLsp";
import { useLoreGuardian } from "../composables/useLoreGuardian";
import { useEnterpriseVault } from "../composables/useEnterpriseVault";
import AnalyticsDrawer from "./studio/AnalyticsDrawer.vue";
import LoreDrawer from "./studio/LoreDrawer.vue";
import VaultDrawer from "./studio/VaultDrawer.vue";
import LspAutocomplete from "./studio/LspAutocomplete.vue";
import LspTooltip from "./studio/LspTooltip.vue";

const title = defineModel<string>("title", { required: true });
const category = defineModel<string>("category", { required: true });
const content = defineModel<string>("content", { required: true });
const revision = defineModel<number>("revision", { default: 1 });

const {
  studioMode,
  fontSize,
  hasActive,
  operator = "Lead-Editor",
  storyId = "",
  isSyncing = false,
} = defineProps<{
  studioMode: StudioViewMode;
  fontSize: number;
  hasActive: boolean;
  operator?: string;
  storyId?: string;
  isSyncing?: boolean;
}>();

const emit = defineEmits<{
  (e: "save"): void;
  (e: "delete"): void;
  (e: "selectStoryById", id: string): void;
}>();

const {
  isSuggestionsOpen,
  suggestions,
  selectedIndex,
  isHoverActive,
  hoverInfo,
  hoverCoordinates,
  evaluateCursorContext,
  applySuggestion,
  handleKeydownNavigation,
  inspectHoverWord,
  dismissHover,
} = useNarrativeLsp(() => storyId || "");

const {
  isAnalyzing: isLoreAnalyzing,
  consistencyScore,
  anomalies,
  semanticResults,
  triggerLoreAuditDebounced,
  executeSemanticSearch,
  dismissAnomaly,
} = useLoreGuardian();

const {
  vaultStatus,
  isSealing,
  isVerifying,
  verificationResult,
  loadVaultStatus,
  sealProject,
  verifyVaultIntegrity,
} = useEnterpriseVault();

const showAnalytics = ref(false);
const showLore = ref(false);
const showVault = ref(false);
const mobileActiveTab = ref<"editor" | "preview">("editor");
const textareaRef = ref<HTMLTextAreaElement | null>(null);
const renderedMarkdown = ref(marked.parse(content.value) as string);
const saveSuccessFeedback = ref(false);

const metrics = ref<TextMetricsResult>({
  wordCount: 0,
  charCount: 0,
  readingTimeMinutes: 0,
  dialogueRatio: 0,
  sentenceCadence: 0,
  vocabularyRichness: 0,
  repetitionWarnings: [],
});

let parseDebounce: number | null = null;
let metricsDebounce: number | null = null;
let hoverTimer: number | null = null;

watch(content, (newText) => {
  if (parseDebounce) clearTimeout(parseDebounce);
  parseDebounce = window.setTimeout(() => {
    renderedMarkdown.value = marked.parse(newText) as string;
  }, 60);

  if (metricsDebounce) clearTimeout(metricsDebounce);
  metricsDebounce = window.setTimeout(async () => {
    try {
      metrics.value = await invoke<TextMetricsResult>("analyze_text_metrics", {
        text: newText,
      });
    } catch {
      metrics.value.wordCount = newText.trim() ? newText.trim().split(/\s+/).length : 0;
      metrics.value.charCount = newText.length;
    }
  }, 200);

  triggerLoreAuditDebounced(newText);
});

watch(
  () => isSyncing,
  (loading, prev) => {
    if (prev && !loading) {
      saveSuccessFeedback.value = true;
      setTimeout(() => {
        saveSuccessFeedback.value = false;
      }, 1500);
    }
  }
);

function handleTextareaInput(e: Event) {
  const target = e.target as HTMLTextAreaElement;
  evaluateCursorContext(content.value, target.selectionStart);
}

function handleTextareaKeydown(e: KeyboardEvent) {
  if (isSuggestionsOpen.value) {
    if (e.key === "Enter" || e.key === "Tab") {
      e.preventDefault();
      if (suggestions.value[selectedIndex.value]) {
        confirmSuggestion(suggestions.value[selectedIndex.value]);
      }
      return;
    }
    const handled = handleKeydownNavigation(e);
    if (handled) return;
  }
}

async function confirmSuggestion(sug: typeof suggestions.value[0]) {
  const el = textareaRef.value;
  const cursor = el ? el.selectionStart : content.value.length;
  const { nextText, nextCursor } = await applySuggestion(sug, content.value, cursor, operator);
  content.value = nextText;
  requestAnimationFrame(() => {
    if (el) {
      el.focus();
      el.setSelectionRange(nextCursor, nextCursor);
    }
  });
}

function handleTextareaMousemove(e: MouseEvent) {
  if (hoverTimer) clearTimeout(hoverTimer);
  const clientX = e.clientX;
  const clientY = e.clientY;

  hoverTimer = window.setTimeout(() => {
    const el = textareaRef.value;
    if (!el) return;
    const charPos = el.selectionStart;
    const txt = content.value;
    const left = txt.slice(0, charPos).split(/[\s,.;!?]+/).pop() || "";
    const right = txt.slice(charPos).split(/[\s,.;!?]+/)[0] || "";
    const hoveredWord = left + right;

    if (hoveredWord.length > 2) {
      inspectHoverWord(hoveredWord, clientX, clientY);
    } else {
      dismissHover();
    }
  }, 180);
}

function handleTextareaMouseleave() {
  if (hoverTimer) clearTimeout(hoverTimer);
  dismissHover();
}

async function handleSealVault(passphrase: string) {
  await sealProject(passphrase, operator);
}

function wrapSelection(prefix: string, suffix: string, placeholder = "") {
  const el = textareaRef.value;
  if (!el) {
    content.value += prefix + placeholder + suffix;
    return;
  }

  const start = el.selectionStart;
  const end = el.selectionEnd;
  const selected = content.value.substring(start, end) || placeholder;
  const replacement = prefix + selected + suffix;

  if (typeof el.setRangeText === "function") {
    el.setRangeText(replacement, start, end, "select");
    content.value = el.value;
    el.dispatchEvent(new Event("input", { bubbles: true }));
  } else {
    content.value =
      content.value.substring(0, start) + replacement + content.value.substring(end);
  }

  requestAnimationFrame(() => {
    el.focus();
    el.setSelectionRange(start + prefix.length, start + prefix.length + selected.length);
  });
}

onMounted(() => {
  loadVaultStatus();
});
</script>

<template>
  <section :class="['enterprise-workbench-studio', `mode-${studioMode}`, `mobile-tab-${mobileActiveTab}`]">
    <div class="mobile-dual-nav">
      <button
        :class="['dual-tab-pill', { active: mobileActiveTab === 'editor' }]"
        @click="mobileActiveTab = 'editor'"
      >
        <span>Терминал</span>
      </button>
      <button
        :class="['dual-tab-pill', { active: mobileActiveTab === 'preview' }]"
        @click="mobileActiveTab = 'preview'"
      >
        <span>Предпросмотр</span>
      </button>
    </div>

    <div v-show="studioMode !== 'preview'" class="editor-chassis">
      <div class="editor-telemetry-header">
        <div class="meta-input-capsules">
          <input
            v-model="title"
            class="capsule-title-input"
            placeholder="Название сцены..."
          />
          <input
            v-model="category"
            class="capsule-category-input"
            placeholder="Секция / Акт"
          />
        </div>

        <div class="header-action-controls">
          <div class="revision-telemetry-badge">
            <span class="rev-label">REV</span>
            <span class="rev-digit">{{ revision }}</span>
          </div>

          <div
            class="vault-status-indicator"
            :class="{ sealed: vaultStatus?.isSealed }"
            title="Статус криптографического сейфа"
            @click="showVault = !showVault"
          >
            <span class="vault-orb"></span>
            <span class="vault-text">{{ vaultStatus?.isSealed ? 'SEALED' : 'UNSEALED' }}</span>
          </div>

          <div
            class="lore-score-badge"
            :class="{ warning: consistencyScore < 80, danger: consistencyScore < 50 }"
            title="Каноническая согласованность сюжета"
            @click="showLore = !showLore"
          >
            <span>LOR</span>
            <b>{{ Math.round(consistencyScore) }}%</b>
          </div>

          <button
            class="btn-plasma-save"
            :class="{ saving: isSyncing, success: saveSuccessFeedback }"
            :disabled="isSyncing"
            title="Сохранить коммит (Ctrl+S / Cmd+S)"
            @click="emit('save')"
          >
            <svg class="ui-icon" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
              <polyline points="17 21 17 13 7 13 7 21"/>
              <polyline points="7 3 7 8 15 8"/>
            </svg>
            <span v-if="saveSuccessFeedback">Сохранено ✓</span>
            <span v-else-if="isSyncing">Синхронизация...</span>
            <span v-else>Коммит</span>
          </button>

          <button
            class="tactile-icon-btn"
            :class="{ active: showLore }"
            title="Страж лора и поиск смысла"
            @click="showLore = !showLore"
          >
            <svg class="ui-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10"/>
              <path d="m4.93 4.93 4.24 4.24"/>
              <path d="m14.83 9.17 4.24-4.24"/>
              <path d="m14.83 14.83 4.24 4.24"/>
              <path d="m9.17 14.83-4.24 4.24"/>
            </svg>
          </button>

          <button
            class="tactile-icon-btn"
            :class="{ active: showVault }"
            title="Управление криптосейфом"
            @click="showVault = !showVault"
          >
            <svg class="ui-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
              <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
            </svg>
          </button>

          <button
            class="tactile-icon-btn"
            :class="{ active: showAnalytics }"
            title="Лингвистическая телеметрия"
            @click="showAnalytics = !showAnalytics"
          >
            <svg class="ui-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="20" x2="18" y2="10"/>
              <line x1="12" y1="20" x2="12" y2="4"/>
              <line x1="6" y1="20" x2="6" y2="14"/>
            </svg>
          </button>

          <button
            class="tactile-icon-btn btn-danger-tone"
            :title="hasActive ? 'Удалить сцену' : 'Очистить черновик'"
            @click="emit('delete')"
          >
            <svg class="ui-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="3 6 5 6 21 6"/>
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
            </svg>
          </button>
        </div>
      </div>

      <div v-if="anomalies.length > 0" class="lore-alert-strip" @click="showLore = true">
        <span class="alert-icon">⚠️</span>
        <span class="alert-msg">Обнаружено {{ anomalies.length }} противоречий с каноном вселенной.</span>
        <span class="alert-action">Исследовать →</span>
      </div>

      <div class="quantum-tools-strip">
        <div class="tools-horizontal-lane">
          <button class="tool-chip-btn" @mousedown.prevent="wrapSelection('\n# ', '\n', 'Заголовок 1')">
            <span class="tool-label-text">H1</span>
          </button>
          <button class="tool-chip-btn" @mousedown.prevent="wrapSelection('\n## ', '\n', 'Заголовок 2')">
            <span class="tool-label-text">H2</span>
          </button>
          <button class="tool-chip-btn" @mousedown.prevent="wrapSelection('\n### ', '\n', 'Заголовок 3')">
            <span class="tool-label-text">H3</span>
          </button>
          <button class="tool-chip-btn" @mousedown.prevent="wrapSelection('**', '**', 'жирный текст')">
            <span class="tool-label-text">B</span>
          </button>
          <button class="tool-chip-btn" @mousedown.prevent="wrapSelection('*', '*', 'курсивный текст')">
            <span class="tool-label-text">I</span>
          </button>
          <button class="tool-chip-btn" @mousedown.prevent="wrapSelection('~~', '~~', 'зачеркнутый текст')">
            <span class="tool-label-text">S</span>
          </button>
          <button class="tool-chip-btn" @mousedown.prevent="wrapSelection('[[', ']]', 'Сущность')">
            <span class="tool-label-text">[[Wiki]]</span>
          </button>
          <button class="tool-chip-btn" @mousedown.prevent="wrapSelection('@', ' ', 'Персонаж')">
            <span class="tool-label-text">@Персонаж</span>
          </button>
          <button class="tool-chip-btn" @mousedown.prevent="wrapSelection('\n> «', '»\n', 'Прямая речь')">
            <span class="tool-label-text">Цитата</span>
          </button>
          <button class="tool-chip-btn" @mousedown.prevent="wrapSelection('\n- ', '\n', 'Элемент списка')">
            <span class="tool-label-text">• Список</span>
          </button>
          <button class="tool-chip-btn" @mousedown.prevent="wrapSelection('\n1. ', '\n', 'Нумерованный пункт')">
            <span class="tool-label-text">1. Пункт</span>
          </button>
          <button class="tool-chip-btn" @mousedown.prevent="wrapSelection('`', '`', 'код')">
            <span class="tool-label-text">&lt;/&gt;</span>
          </button>
          <button class="tool-chip-btn" @mousedown.prevent="wrapSelection('\n```\n', '\n```\n', 'код блока')">
            <span class="tool-label-text">Блок кода</span>
          </button>
          <button class="tool-chip-btn" @mousedown.prevent="wrapSelection('\n| Параметр | Значение |\n|---|---|\n| Свойство | Данные |\n', '', '')">
            <span class="tool-label-text">Таблица</span>
          </button>
          <button class="tool-chip-btn" @mousedown.prevent="wrapSelection('\n\n---\n\n', '', '')">
            <span class="tool-label-text">Разделитель</span>
          </button>
        </div>
      </div>

      <div class="editor-viewport-well">
        <textarea
          ref="textareaRef"
          v-model="content"
          class="enterprise-textarea"
          placeholder="Формирование зашифрованного драфта. Введите [[ для автокомплита сущностей..."
          autofocus
          @input="handleTextareaInput"
          @keydown="handleTextareaKeydown"
          @mousemove="handleTextareaMousemove"
          @mouseleave="handleTextareaMouseleave"
        ></textarea>

        <LspAutocomplete
          v-if="isSuggestionsOpen"
          :suggestions="suggestions"
          :selected-index="selectedIndex"
          @hover-index="selectedIndex = $event"
          @select="confirmSuggestion"
        />

        <LspTooltip
          v-if="isHoverActive && hoverInfo"
          :hover-info="hoverInfo"
          :coordinates="hoverCoordinates"
        />

        <AnalyticsDrawer
          v-if="showAnalytics"
          :metrics="metrics"
          @close="showAnalytics = false"
        />

        <LoreDrawer
          v-if="showLore"
          :consistency-score="consistencyScore"
          :anomalies="anomalies"
          :semantic-results="semanticResults"
          :is-analyzing="isLoreAnalyzing"
          @close="showLore = false"
          @dismiss-anomaly="dismissAnomaly"
          @search-semantic="executeSemanticSearch"
          @select-story-by-id="emit('selectStoryById', $event)"
        />

        <VaultDrawer
          v-if="showVault"
          :vault-status="vaultStatus"
          :is-sealing="isSealing"
          :is-verifying="isVerifying"
          :verification-result="verificationResult"
          @close="showVault = false"
          @seal="handleSealVault"
          @verify="verifyVaultIntegrity"
        />
      </div>

      <footer class="editor-bottom-hud">
        <div class="hud-goal-track">
          <span class="hud-label">Слов: {{ metrics.wordCount }}</span>
          <span class="hud-label">Символов: {{ metrics.charCount }}</span>
          <span class="hud-label">Канон: {{ Math.round(consistencyScore) }}%</span>
        </div>
        <span class="hud-metric">Сейф: {{ vaultStatus?.isSealed ? 'Защищен' : 'Активен' }} • Оператор: {{ operator }}</span>
      </footer>
    </div>

    <div
      v-if="studioMode === 'split' || studioMode === 'preview' || mobileActiveTab === 'preview'"
      :class="['preview-chassis', { 'full-screen-preview': studioMode === 'preview' }]"
    >
      <div class="preview-toolbar-meta" v-if="studioMode === 'preview'">
        <span class="preview-badge-category">{{ category }}</span>
        <h2 class="preview-title-heading">{{ title }}</h2>
      </div>

      <div
        class="preview-parchment-sheet markdown-rendered"
        :style="{ fontSize: `${fontSize}px` }"
        v-html="renderedMarkdown"
      ></div>
    </div>
  </section>
</template>

<style scoped>
.enterprise-workbench-studio {
  flex: 1;
  display: flex;
  overflow: hidden;
  height: 100%;
  background: var(--bg-deep);
  position: relative;
}

.mobile-dual-nav {
  display: none;
  background: var(--bg-surface-solid);
  padding: 8px;
  gap: 8px;
  border-bottom: 1px solid var(--border);
}

.dual-tab-pill {
  flex: 1;
  padding: 10px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 700;
  color: var(--text-muted);
  cursor: pointer;
}

.dual-tab-pill.active {
  background: var(--bg-card);
  border-color: var(--border);
  color: var(--text-pure);
}

.enterprise-workbench-studio.mode-split .editor-chassis {
  flex: 1;
  border-right: 1px solid var(--border);
}

.enterprise-workbench-studio.mode-split .preview-chassis {
  flex: 1;
}

.enterprise-workbench-studio.mode-preview .editor-chassis {
  display: none;
}

.enterprise-workbench-studio.mode-preview .preview-chassis {
  flex: 1;
  max-width: 900px;
  margin: 0 auto;
}

.enterprise-workbench-studio.mode-edit .editor-chassis {
  flex: 1;
  max-width: 1080px;
  margin: 0 auto;
}

.enterprise-workbench-studio.mode-zen .editor-chassis {
  flex: 1;
  max-width: 840px;
  margin: 0 auto;
}

.editor-chassis {
  display: flex;
  flex-direction: column;
  padding: 16px 22px;
  gap: 10px;
  height: 100%;
}

.editor-telemetry-header {
  display: flex;
  gap: 12px;
  align-items: center;
}

.meta-input-capsules {
  display: flex;
  gap: 10px;
  flex: 1;
}

.capsule-title-input {
  flex: 3;
  background: var(--bg-card);
  border: 1px solid var(--border);
  box-shadow: inset 0 2px 6px rgba(0, 0, 0, 0.5);
  color: var(--text-pure);
  padding: 0 16px;
  height: 42px;
  border-radius: 12px;
  font-size: 15px;
  font-weight: 700;
  outline: none;
}

.capsule-category-input {
  flex: 1;
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-main);
  padding: 0 14px;
  height: 42px;
  border-radius: 12px;
  font-size: 13px;
  font-weight: 600;
  outline: none;
}

.header-action-controls {
  display: flex;
  gap: 8px;
  align-items: center;
}

.revision-telemetry-badge {
  display: flex;
  align-items: center;
  background: rgba(0, 0, 0, 0.6);
  border: 1px solid var(--border-strong);
  border-radius: 10px;
  height: 42px;
  padding: 0 10px;
  gap: 6px;
  font-family: var(--font-mono);
}

.rev-label {
  font-size: 10px;
  color: var(--accent-cyan);
  font-weight: 800;
}

.rev-digit {
  font-size: 13px;
  font-weight: 800;
  color: var(--text-pure);
}

.vault-status-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  background: rgba(0, 0, 0, 0.5);
  border: 1px solid var(--border);
  border-radius: 10px;
  height: 42px;
  padding: 0 12px;
  cursor: pointer;
}

.vault-orb {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--accent-rose);
}

.vault-status-indicator.sealed .vault-orb {
  background: var(--accent-emerald);
}

.vault-text {
  font-size: 11px;
  font-weight: 800;
  font-family: var(--font-mono);
  color: var(--text-muted);
}

.lore-score-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  background: rgba(16, 185, 129, 0.12);
  border: 1px solid rgba(16, 185, 129, 0.3);
  color: var(--accent-emerald);
  height: 42px;
  padding: 0 12px;
  border-radius: 10px;
  font-size: 11px;
  font-family: var(--font-mono);
  cursor: pointer;
}

.lore-score-badge.warning {
  background: rgba(245, 158, 11, 0.12);
  border-color: rgba(245, 158, 11, 0.3);
  color: var(--accent-gold);
}

.lore-score-badge.danger {
  background: rgba(244, 63, 94, 0.15);
  border-color: rgba(244, 63, 94, 0.35);
  color: var(--accent-rose);
}

.btn-plasma-save {
  background: linear-gradient(135deg, var(--accent-indigo) 0%, #4338ca 100%);
  color: #fff;
  border: none;
  border-radius: 12px;
  padding: 0 18px;
  height: 42px;
  font-size: 13px;
  font-weight: 800;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.2s ease;
}

.btn-plasma-save.saving {
  opacity: 0.7;
}

.btn-plasma-save.success {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
}

.tactile-icon-btn {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-muted);
  border-radius: 12px;
  width: 42px;
  height: 42px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.tactile-icon-btn.active {
  background: var(--accent-indigo);
  color: #fff;
}

.btn-danger-tone:hover {
  background: rgba(244, 63, 94, 0.2);
  color: var(--accent-rose);
}

.lore-alert-strip {
  display: flex;
  align-items: center;
  gap: 10px;
  background: rgba(244, 63, 94, 0.15);
  border: 1px solid var(--accent-rose);
  border-radius: 10px;
  padding: 8px 14px;
  font-size: 12px;
  cursor: pointer;
}

.alert-msg {
  flex: 1;
  color: var(--text-pure);
}

.alert-action {
  color: var(--accent-rose);
  font-weight: 700;
}

.quantum-tools-strip {
  width: 100%;
  overflow-x: auto;
}

.tools-horizontal-lane {
  display: flex;
  gap: 6px;
  width: max-content;
}

.tool-chip-btn {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--border);
  color: var(--text-muted);
  padding: 0 12px;
  height: 32px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.tool-chip-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-pure);
}

.tool-label-text {
  font-size: 11px;
  font-weight: 800;
}

.editor-viewport-well {
  flex: 1;
  position: relative;
  display: flex;
  overflow: hidden;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 20px;
}

.enterprise-textarea {
  flex: 1;
  width: 100%;
  background: transparent;
  border: none;
  color: var(--text-pure);
  font-size: 16px;
  line-height: 1.85;
  outline: none;
  resize: none;
  user-select: text;
}

.editor-bottom-hud {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px;
  font-size: 11px;
}

.hud-goal-track {
  display: flex;
  gap: 14px;
}

.hud-label {
  color: var(--text-dim);
  font-weight: 600;
}

.hud-metric {
  font-family: var(--font-mono);
  color: var(--text-muted);
}

.preview-chassis {
  height: 100%;
  overflow-y: auto;
  padding: 32px 42px;
  background: rgba(0, 0, 0, 0.35);
  user-select: text;
}

.preview-toolbar-meta {
  margin-bottom: 24px;
}

.preview-badge-category {
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--accent-cyan);
}

.preview-title-heading {
  font-size: 26px;
  font-weight: 800;
  color: var(--text-pure);
  margin-top: 4px;
}

.preview-parchment-sheet {
  max-width: 800px;
  margin: 0 auto;
}

@media (max-width: 768px) {
  .mobile-dual-nav {
    display: flex;
  }
  .editor-chassis {
    padding: 10px;
  }
  .editor-telemetry-header {
    flex-direction: column;
    align-items: stretch;
  }
  .header-action-controls {
    justify-content: space-between;
  }
  .enterprise-workbench-studio.mobile-tab-editor .editor-chassis {
    display: flex;
  }
  .enterprise-workbench-studio.mobile-tab-editor .preview-chassis {
    display: none !important;
  }
  .enterprise-workbench-studio.mobile-tab-preview .editor-chassis {
    display: none !important;
  }
  .enterprise-workbench-studio.mobile-tab-preview .preview-chassis {
    display: block !important;
    flex: 1;
    padding: 16px;
  }
}
</style>