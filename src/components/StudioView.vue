<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  EnterpriseStoryItem,
  SnapshotItem,
  StudioDrawerType,
  StudioViewMode,
  TextMetricsResult,
} from "../types";
import { renderMarkdown, toggleMarkdownTask } from "../utils/markdown";
import { useNarrativeLsp } from "../composables/useNarrativeLsp";
import { useLoreGuardian } from "../composables/useLoreGuardian";
import { useEnterpriseVault } from "../composables/useEnterpriseVault";
import { useRopeEditor } from "../composables/useRopeEditor";
import { useHaptics } from "../composables/useHaptics";
import { useAppSettings } from "../composables/useAppSettings";
import { useI18n } from "../i18n";
import StudioHeader from "./studio/StudioHeader.vue";
import StudioToolbar from "./studio/StudioToolbar.vue";
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
  projectId,
  studioMode,
  fontSize,
  hasActive,
  compactKeyboardMode = false,
  operator = "Lead-Editor",
  storyId = "",
} = defineProps<{
  projectId: string;
  studioMode: StudioViewMode;
  fontSize: number;
  hasActive: boolean;
  compactKeyboardMode?: boolean;
  operator?: string;
  storyId?: string;
}>();

const emit = defineEmits<{
  (e: "save"): void;
  (e: "delete"): void;
  (e: "selectStoryById", id: string): void;
  (e: "selectEntityByTitle", entityTitle: string): void;
  (e: "entityCreated", story: EnterpriseStoryItem): void;
  (e: "notify", msg: string, kind: "success" | "error"): void;
}>();

const { settings } = useAppSettings();
const { vibrateLight, vibrateSelection, vibrateSuccess } = useHaptics();
const { t, locale } = useI18n();

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
} = useNarrativeLsp(() => projectId, () => storyId || "");

const {
  isAnalyzing: isLoreAnalyzing,
  consistencyScore,
  verifiedClaimsCount,
  anomalies,
  semanticResults,
  triggerLoreAuditDebounced,
  executeSemanticSearch,
  dismissAnomaly,
} = useLoreGuardian(() => projectId);

const {
  vaultStatus,
  snapshots,
  isSealing,
  isUnlocking,
  isVerifying,
  isLoadingSnapshots,
  verificationResult,
  vaultError,
  loadVaultStatus,
  sealProject,
  unlockProject,
  verifyVaultIntegrity,
  loadSnapshots,
  createSnapshot,
  deleteSnapshot,
} = useEnterpriseVault(() => projectId, () => storyId || "");

const {
  totalLines: ropeTotalLines,
  revisionHash: ropeRevisionHash,
  initSession: initRopeSession,
  syncTextDelta,
  closeActiveSession: closeRopeSession,
} = useRopeEditor(() => projectId, () => storyId || "");

const activeDrawer = ref<StudioDrawerType>(null);
const mobileActiveTab = ref<"editor" | "preview">("editor");
const textareaRef = ref<HTMLTextAreaElement | null>(null);
const renderedHtml = ref(renderMarkdown(content.value));

const isVaultLocked = computed(() => Boolean(vaultStatus.value?.isLocked));

const metrics = ref<TextMetricsResult>({
  wordCount: 0,
  charCount: 0,
  paragraphCount: 1,
  readingTimeMinutes: 0,
  dialogueRatio: 0,
  sentenceCadence: 0,
  vocabularyRichness: 0,
  fleschReadingEase: 0,
  sentimentToneKey: "neutral",
  repetitionWarnings: [],
  clicheWarnings: [],
  pacingTimeline: [0, 0, 0, 0, 0, 0, 0, 0],
});

let parseDebounce: number | null = null;
let metricsDebounce: number | null = null;
let hoverTimer: number | null = null;

function toggleDrawer(drawer: Exclude<StudioDrawerType, null>) {
  activeDrawer.value = activeDrawer.value === drawer ? null : drawer;
  if (activeDrawer.value === "vault") {
    loadSnapshots();
  }
}

async function computeMetrics(text: string) {
  try {
    metrics.value = await invoke<TextMetricsResult>("analyze_text_metrics", {
      text,
      locale: locale.value,
    });
  } catch {
    metrics.value.wordCount = text.trim() ? text.trim().split(/\s+/).length : 0;
    metrics.value.charCount = text.length;
  }
}

watch(
  () => [storyId, projectId],
  async ([nextStoryId]) => {
    await initRopeSession(content.value, nextStoryId || "__draft__");
    await loadVaultStatus();
    await loadSnapshots(nextStoryId || "");
  },
  { immediate: true }
);

watch(
  [content, locale],
  ([newText]) => {
    syncTextDelta(newText);

    if (parseDebounce) clearTimeout(parseDebounce);
    parseDebounce = window.setTimeout(() => {
      renderedHtml.value = renderMarkdown(newText);
    }, 40);

    if (metricsDebounce) clearTimeout(metricsDebounce);
    metricsDebounce = window.setTimeout(() => {
      computeMetrics(newText);
    }, 180);

    if (settings.value.loreAuditOnInput) {
      triggerLoreAuditDebounced(newText);
    }
  },
  { immediate: true }
);

watch(
  [() => projectId, () => settings.value.loreSensitivity],
  () => {
    if (settings.value.loreAuditOnInput) {
      triggerLoreAuditDebounced(content.value, 120);
    }
  }
);

function inspectCursorToken(el: HTMLTextAreaElement) {
  const pos = el.selectionStart;
  const txt = content.value;
  const before = txt.slice(0, pos);
  const openBracket = before.lastIndexOf("[[");
  const closeBracket = before.lastIndexOf("]]");
  if (openBracket !== -1 && openBracket > closeBracket) {
    const endBracket = txt.indexOf("]]", pos);
    const token =
      endBracket !== -1 ? txt.slice(openBracket + 2, endBracket) : txt.slice(openBracket + 2, pos);
    if (token.trim().length >= 2) {
      const rect = el.getBoundingClientRect();
      inspectHoverWord(token, rect.left + 20, rect.top + 30);
      return;
    }
  }
  dismissHover();
}

function handleTextareaInput(e: Event) {
  if (isVaultLocked.value) return;
  const target = e.target as HTMLTextAreaElement;
  evaluateCursorContext(content.value, target.selectionStart);
}

function handleTextareaCursorEvent(e: Event) {
  const target = e.target as HTMLTextAreaElement;
  inspectCursorToken(target);
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
  if (isVaultLocked.value) return;
  const el = textareaRef.value;
  const cursor = el ? el.selectionStart : content.value.length;
  const { nextText, nextCursor, createdStory } = await applySuggestion(
    sug,
    content.value,
    cursor,
    operator
  );
  if (createdStory) {
    emit("entityCreated", createdStory);
  }
  content.value = nextText;
  vibrateLight();
  requestAnimationFrame(() => {
    if (el) {
      el.focus();
      el.setSelectionRange(nextCursor, nextCursor);
    }
  });
}

function handlePreviewMouseover(e: MouseEvent) {
  const target = e.target as HTMLElement;
  const entityNode = target.closest(".md-wikilink, .md-mention") as HTMLElement | null;
  if (!entityNode) return;

  const lookup =
    entityNode.dataset.entity || entityNode.textContent?.replace(/[@◈]/g, "").trim() || "";
  if (lookup.length < 2) return;

  if (hoverTimer) clearTimeout(hoverTimer);
  const cx = e.clientX;
  const cy = e.clientY;
  hoverTimer = window.setTimeout(() => {
    inspectHoverWord(lookup, cx, cy);
  }, 140);
}

function handlePreviewMouseout(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (target.closest(".md-wikilink, .md-mention")) {
    if (hoverTimer) clearTimeout(hoverTimer);
    dismissHover();
  }
}

async function handleSealVault(passphrase: string, lockEditing: boolean) {
  const ok = await sealProject(passphrase, lockEditing, operator);
  if (ok) {
    emit("notify", t("notifications.vaultSealedToast"), "success");
  }
}

async function handleUnlockVault(passphrase: string) {
  const ok = await unlockProject(passphrase, operator);
  if (ok) {
    emit("notify", t("notifications.vaultUnlockedToast"), "success");
  } else if (vaultError.value) {
    emit("notify", t("notifications.vaultUnlockError", { error: vaultError.value }), "error");
  }
}

async function handleCreateSnapshot() {
  const snap = await createSnapshot(storyId, title.value, content.value);
  if (snap) {
    emit("notify", t("notifications.snapshotSavedToast"), "success");
  }
}

function handleRestoreSnapshot(snap: SnapshotItem) {
  if (isVaultLocked.value) return;
  title.value = snap.title;
  content.value = snap.content;
  vibrateSuccess();
  emit("notify", t("notifications.snapshotRestoredToast"), "success");
}

function wrapSelection(prefix: string, suffix: string, placeholder = "") {
  if (isVaultLocked.value) return;
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

  vibrateLight();
  requestAnimationFrame(() => {
    el.focus();
    el.setSelectionRange(start + prefix.length, start + prefix.length + selected.length);
  });
}

function handlePreviewClick(e: MouseEvent) {
  const target = e.target as HTMLElement;

  const wikilink = target.closest(".md-wikilink") as HTMLElement | null;
  if (wikilink && wikilink.dataset.entity) {
    e.preventDefault();
    vibrateSelection();
    emit("selectEntityByTitle", wikilink.dataset.entity);
    return;
  }

  if (isVaultLocked.value) return;
  const taskCheck = target.closest(".task-check") as HTMLInputElement | null;
  if (taskCheck && taskCheck.dataset.taskIndex !== undefined) {
    const idx = parseInt(taskCheck.dataset.taskIndex, 10);
    content.value = toggleMarkdownTask(content.value, idx);
    vibrateSuccess();
  }
}

onMounted(() => {
  loadVaultStatus();
});

onUnmounted(() => {
  if (parseDebounce) clearTimeout(parseDebounce);
  if (metricsDebounce) clearTimeout(metricsDebounce);
  if (hoverTimer) clearTimeout(hoverTimer);
  closeRopeSession();
});
</script>

<template>
  <section :class="['enterprise-workbench-studio', `mode-${studioMode}`]">
    <div v-show="!compactKeyboardMode" class="mobile-dual-nav">
      <button
        :class="['dual-tab-pill', { active: mobileActiveTab === 'editor' }]"
        @click="mobileActiveTab = 'editor'"
      >
        <span>{{ t("studio.editorTab") }}</span>
      </button>
      <button
        :class="['dual-tab-pill', { active: mobileActiveTab === 'preview' }]"
        @click="mobileActiveTab = 'preview'"
      >
        <span>{{ t("studio.previewTab") }}</span>
      </button>
    </div>

    <div
      v-show="studioMode !== 'preview' && (mobileActiveTab !== 'preview' || compactKeyboardMode)"
      class="editor-chassis"
      :style="{ maxWidth: settings.canvasMaxWidth }"
    >
      <StudioHeader
        v-show="!compactKeyboardMode"
        v-model:title="title"
        v-model:category="category"
        :revision="revision"
        :vault-sealed="!!vaultStatus?.isSealed"
        :vault-locked="isVaultLocked"
        :consistency-score="consistencyScore"
        :active-drawer="activeDrawer"
        :has-active="hasActive"
        @toggle-drawer="toggleDrawer"
        @delete="emit('delete')"
      />

      <div v-if="isVaultLocked && !compactKeyboardMode" class="vault-locked-banner" @click="activeDrawer = 'vault'">
        <span>🔒</span>
        <span>{{ t("studio.vaultLockedBanner") }}</span>
      </div>

      <div v-if="anomalies.length > 0 && !compactKeyboardMode" class="lore-alert-strip" @click="activeDrawer = 'lore'">
        <span class="alert-icon">⚠️</span>
        <span class="alert-msg">{{ t("studio.loreAlert", { count: anomalies.length }) }}</span>
        <span class="alert-action">{{ t("studio.loreInspect") }}</span>
      </div>

      <StudioToolbar v-if="!isVaultLocked && !compactKeyboardMode" @insert="wrapSelection" />

      <div class="editor-viewport-well">
        <textarea
          ref="textareaRef"
          v-model="content"
          :readonly="isVaultLocked"
          :class="['enterprise-textarea', { 'smooth-caret': settings.smoothCaret }]"
          :style="{
            fontFamily: settings.editorFontFamily,
            lineHeight: settings.editorLineHeight,
            fontSize: `${fontSize}px`,
          }"
          :placeholder="t('studio.editorPlaceholder')"
          autofocus
          @input="handleTextareaInput"
          @keydown="handleTextareaKeydown"
          @click="handleTextareaCursorEvent"
          @keyup="handleTextareaCursorEvent"
          @blur="dismissHover"
        ></textarea>

        <LspAutocomplete
          v-if="isSuggestionsOpen && !isVaultLocked"
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
          v-if="activeDrawer === 'analytics'"
          :metrics="metrics"
          @close="activeDrawer = null"
        />

        <LoreDrawer
          v-if="activeDrawer === 'lore'"
          :consistency-score="consistencyScore"
          :verified-claims-count="verifiedClaimsCount"
          :anomalies="anomalies"
          :semantic-results="semanticResults"
          :is-analyzing="isLoreAnalyzing"
          @close="activeDrawer = null"
          @dismiss-anomaly="dismissAnomaly"
          @search-semantic="executeSemanticSearch"
          @select-story-by-id="emit('selectStoryById', $event)"
        />

        <VaultDrawer
          v-if="activeDrawer === 'vault'"
          :vault-status="vaultStatus"
          :snapshots="snapshots"
          :is-sealing="isSealing"
          :is-unlocking="isUnlocking"
          :is-verifying="isVerifying"
          :is-loading-snapshots="isLoadingSnapshots"
          :verification-result="verificationResult"
          :vault-error="vaultError"
          :has-active-story="!!storyId"
          @close="activeDrawer = null"
          @seal="handleSealVault"
          @unlock="handleUnlockVault"
          @verify="verifyVaultIntegrity"
          @create-snapshot="handleCreateSnapshot"
          @restore-snapshot="handleRestoreSnapshot"
          @delete-snapshot="deleteSnapshot"
        />
      </div>

      <footer v-show="!compactKeyboardMode" class="editor-bottom-hud">
        <div class="hud-goal-track">
          <span class="hud-label">{{ t("studio.wordsStat", { count: metrics.wordCount }) }}</span>
          <span class="hud-label">{{ t("studio.charsStat", { count: metrics.charCount }) }}</span>
          <span class="hud-label">{{ t("studio.ropeLinesStat", { count: ropeTotalLines }) }}</span>
          <span class="hud-label">{{ t("studio.canonStat", { score: Math.round(consistencyScore) }) }}</span>
        </div>
        <span class="hud-metric">
          {{ t("studio.ropeHashStat", { hash: ropeRevisionHash ? ropeRevisionHash.slice(0, 8) : "00000000" }) }} •
          {{ t("studio.operatorLabel", { status: vaultStatus?.isSealed ? t("studio.vaultProtected") : t("studio.vaultActive"), operator }) }}
        </span>
      </footer>
    </div>

    <div
      v-if="(studioMode === 'split' || studioMode === 'preview' || mobileActiveTab === 'preview') && !compactKeyboardMode"
      v-show="mobileActiveTab === 'preview' || studioMode === 'split' || studioMode === 'preview'"
      :class="['preview-chassis', { 'full-screen-preview': studioMode === 'preview' || mobileActiveTab === 'preview' }]"
    >
      <div class="preview-toolbar-meta" v-if="studioMode === 'preview' || mobileActiveTab === 'preview'">
        <span class="preview-badge-category">{{ category }}</span>
        <h2 class="preview-title-heading">{{ title }}</h2>
      </div>

      <div
        class="preview-parchment-sheet markdown-rendered"
        :style="{ fontSize: `${fontSize}px` }"
        v-html="renderedHtml"
        @click="handlePreviewClick"
        @mouseover="handlePreviewMouseover"
        @mouseout="handlePreviewMouseout"
      ></div>
    </div>
  </section>
</template>

<style scoped>
.enterprise-workbench-studio {
  flex: 1;
  display: flex;
  flex-direction: row;
  overflow: hidden;
  height: 100%;
  background: var(--bg-deep);
  position: relative;
}

.mobile-dual-nav {
  display: none;
}

.dual-tab-pill {
  flex: 1;
  padding: 9px 12px;
  min-height: 38px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  font-size: 13px;
  font-weight: 700;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.2s ease;
}

.dual-tab-pill.active {
  background: var(--accent-indigo);
  border-color: var(--accent-indigo);
  color: #fff;
  box-shadow: 0 2px 10px var(--accent-indigo-glow);
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

.editor-chassis {
  display: flex;
  flex-direction: column;
  padding: 16px max(22px, var(--safe-right)) 16px max(22px, var(--safe-left));
  gap: 10px;
  height: 100%;
  flex: 1;
  min-width: 0;
  margin: 0 auto;
  transition: max-width 0.2s ease;
}

.vault-locked-banner {
  display: flex;
  align-items: center;
  gap: 10px;
  background: rgba(245, 158, 11, 0.14);
  border: 1px solid var(--accent-gold);
  color: var(--text-pure);
  border-radius: 10px;
  padding: 8px 14px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.lore-alert-strip {
  display: flex;
  align-items: center;
  gap: 10px;
  background: rgba(244, 63, 94, 0.12);
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

.editor-viewport-well {
  flex: 1;
  position: relative;
  display: flex;
  overflow: hidden;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 18px;
  min-height: 0;
}

.enterprise-textarea {
  flex: 1;
  width: 100%;
  background: transparent;
  border: none;
  color: var(--text-main);
  outline: none;
  resize: none;
  user-select: text;
  -webkit-overflow-scrolling: touch;
}

.enterprise-textarea.smooth-caret {
  caret-color: var(--accent-cyan);
}

.editor-bottom-hud {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px;
  font-size: 11px;
  gap: 8px;
  flex-wrap: wrap;
}

.hud-goal-track {
  display: flex;
  gap: 14px;
  flex-wrap: wrap;
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
  padding: 28px max(36px, var(--safe-right)) 28px max(36px, var(--safe-left));
  background: var(--bg-card);
  user-select: text;
  -webkit-overflow-scrolling: touch;
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

@media (max-width: 900px) {
  .enterprise-workbench-studio {
    flex-direction: column !important;
  }

  .mobile-dual-nav {
    display: flex !important;
    width: 100% !important;
    padding: 8px max(12px, var(--safe-right)) 8px max(12px, var(--safe-left));
    background: var(--bg-surface-solid);
    border-bottom: 1px solid var(--border);
    gap: 8px;
    flex-shrink: 0;
  }

  .editor-chassis {
    padding: 8px max(10px, var(--safe-right)) 8px max(10px, var(--safe-left));
    width: 100% !important;
    border-right: none !important;
  }

  .editor-viewport-well {
    padding: 12px;
    border-radius: 14px;
  }

  .preview-chassis {
    padding: 14px max(14px, var(--safe-right)) 24px max(14px, var(--safe-left));
    width: 100% !important;
  }
}
</style>