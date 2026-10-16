<script setup lang="ts">
import { ref, watch } from "vue";
import { marked } from "marked";
import { invoke } from "@tauri-apps/api/core";
import type { StudioViewMode, SnapshotItem, TextMetricsResult } from "../types";

const title = defineModel<string>("title", { required: true });
const category = defineModel<string>("category", { required: true });
const content = defineModel<string>("content", { required: true });

const { studioMode, fontSize, hasActive, snapshots } = defineProps<{
  studioMode: StudioViewMode;
  fontSize: number;
  hasActive: boolean;
  snapshots: SnapshotItem[];
}>();

const emit = defineEmits<{
  (e: "save"): void;
  (e: "delete"): void;
  (e: "makeSnapshot"): void;
  (e: "restoreSnapshot", snap: SnapshotItem): void;
}>();

const showTimeline = ref(false);
const showAnalytics = ref(false);
const mobileActiveTab = ref<"editor" | "preview">("editor");
const textareaRef = ref<HTMLTextAreaElement | null>(null);
const renderedMarkdown = ref(marked.parse(content.value) as string);
const metrics = ref<TextMetricsResult>({
  wordCount: 0,
  charCount: 0,
  readingTimeMinutes: 0,
  dialogueRatio: 0,
  sentenceCadence: 0,
  vocabularyRichness: 0,
  repetitionWarnings: [],
});

let parseTimeout: number | null = null;
let metricsTimeout: number | null = null;

watch(content, (newVal) => {
  if (parseTimeout) clearTimeout(parseTimeout);
  parseTimeout = window.setTimeout(() => {
    renderedMarkdown.value = marked.parse(newVal) as string;
  }, 90);

  if (metricsTimeout) clearTimeout(metricsTimeout);
  metricsTimeout = window.setTimeout(async () => {
    try {
      metrics.value = await invoke<TextMetricsResult>("analyze_text_metrics", {
        text: newVal,
      });
    } catch {
      metrics.value.wordCount = newVal.trim() ? newVal.trim().split(/\s+/).length : 0;
    }
  }, 300);
});

function wrapSelection(prefix: string, suffix: string, defaultPlaceholder = "") {
  const el = textareaRef.value;
  if (!el) {
    content.value += prefix + defaultPlaceholder + suffix;
    return;
  }

  const start = el.selectionStart;
  const end = el.selectionEnd;
  const selected = content.value.substring(start, end) || defaultPlaceholder;
  const replacement = prefix + selected + suffix;

  content.value =
    content.value.substring(0, start) + replacement + content.value.substring(end);

  requestAnimationFrame(() => {
    el.focus();
    el.setSelectionRange(start + prefix.length, start + prefix.length + selected.length);
  });
}
</script>

<template>
  <section :class="['quantum-workbench-studio', `mode-${studioMode}`, `mobile-tab-${mobileActiveTab}`]">
    <div class="mobile-dual-nav">
      <button
        :class="['dual-tab-pill', { active: mobileActiveTab === 'editor' }]"
        @click="mobileActiveTab = 'editor'"
      >
        <svg class="ui-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 20h9"/>
          <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/>
        </svg>
        <span>Терминал</span>
      </button>

      <button
        :class="['dual-tab-pill', { active: mobileActiveTab === 'preview' }]"
        @click="mobileActiveTab = 'preview'"
      >
        <svg class="ui-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
          <circle cx="12" cy="12" r="3"/>
        </svg>
        <span>Предпросмотр</span>
      </button>
    </div>

    <div class="editor-chassis">
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
          <button class="btn-plasma-save" title="Сохранить изменения (Ctrl+S)" @click="emit('save')">
            <svg class="ui-icon" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
              <polyline points="17 21 17 13 7 13 7 21"/>
              <polyline points="7 3 7 8 15 8"/>
            </svg>
            <span>Сохранить</span>
          </button>

          <button
            class="tactile-icon-btn"
            :class="{ active: showAnalytics }"
            title="Нарративный линтер и метрики стиля"
            @click="showAnalytics = !showAnalytics"
          >
            <svg class="ui-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="20" x2="18" y2="10"/>
              <line x1="12" y1="20" x2="12" y2="4"/>
              <line x1="6" y1="20" x2="6" y2="14"/>
            </svg>
          </button>

          <button
            class="tactile-icon-btn"
            :class="{ active: showTimeline }"
            title="История снимков и ревизий"
            @click="showTimeline = !showTimeline"
          >
            <svg class="ui-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10"/>
              <polyline points="12 6 12 12 14 14"/>
            </svg>
          </button>

          <button
            v-if="hasActive"
            class="tactile-icon-btn btn-danger-tone"
            title="Удалить сцену из произведения"
            @click="emit('delete')"
          >
            <svg class="ui-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="3 6 5 6 21 6"/>
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
              <line x1="10" y1="11" x2="10" y2="17"/>
              <line x1="14" y1="11" x2="14" y2="17"/>
            </svg>
          </button>
        </div>
      </div>

      <div class="quantum-tools-strip">
        <div class="tools-horizontal-lane">
          <button class="tool-chip-btn" title="Заголовок H1" @click="wrapSelection('\n# ', '\n', 'Заголовок 1')">
            <span class="tool-label-text">H1</span>
          </button>

          <button class="tool-chip-btn" title="Заголовок H2" @click="wrapSelection('\n## ', '\n', 'Заголовок 2')">
            <span class="tool-label-text">H2</span>
          </button>

          <button class="tool-chip-btn" title="Заголовок H3" @click="wrapSelection('\n### ', '\n', 'Заголовок 3')">
            <span class="tool-label-text">H3</span>
          </button>

          <button class="tool-chip-btn" title="Полужирный текст" @click="wrapSelection('**', '**', 'жирный текст')">
            <svg class="ui-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M6 4h8a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"/>
              <path d="M6 12h9a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"/>
            </svg>
          </button>

          <button class="tool-chip-btn" title="Курсивный текст" @click="wrapSelection('*', '*', 'курсивный текст')">
            <svg class="ui-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="19" y1="4" x2="10" y2="4"/>
              <line x1="14" y1="20" x2="5" y2="20"/>
              <line x1="15" y1="4" x2="9" y2="20"/>
            </svg>
          </button>

          <button class="tool-chip-btn" title="Зачеркнутый текст" @click="wrapSelection('~~', '~~', 'зачеркнуто')">
            <svg class="ui-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M17.5 7.5A6.5 6.5 0 0 0 6.8 5.7M4 12h16m-4.2 6.5A6.5 6.5 0 0 1 6.5 17"/>
            </svg>
          </button>

          <button class="tool-chip-btn" title="Маркерное выделение" @click="wrapSelection('==', '==', 'выделение')">
            <svg class="ui-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="m9 11-6 6v3h3l6-6"/>
              <path d="m22 12-4.6 4.6a2 2 0 0 1-2.8 0l-5.2-5.2a2 2 0 0 1 0-2.8L14 4"/>
            </svg>
          </button>

          <button class="tool-chip-btn" title="Встроенный код" @click="wrapSelection('`', '`', 'код')">
            <svg class="ui-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="16 18 22 12 16 6"/>
              <polyline points="8 6 2 12 8 18"/>
            </svg>
          </button>

          <button class="tool-chip-btn" title="Блок кода" @click="wrapSelection('\n```typescript\n', '\n```\n', '// код')">
            <svg class="ui-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="2" y="4" width="20" height="16" rx="2"/>
              <path d="m10 10-2 2 2 2"/>
              <path d="m14 14 2-2-2-2"/>
            </svg>
          </button>

          <button class="tool-chip-btn" title="Прямая речь и цитата" @click="wrapSelection('\n> «', '»\n', 'Прямая речь')">
            <svg class="ui-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 21c3 0 7-1 7-8V5c0-1.25-.756-2.017-2-2H4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V20c0 1 0 1 1 1z"/>
              <path d="M15 21c3 0 7-1 7-8V5c0-1.25-.757-2.017-2-2h-4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V20c0 1 0 1 1 1z"/>
            </svg>
          </button>

          <button class="tool-chip-btn" title="Интерактивный чек-лист" @click="wrapSelection('\n- [ ] ', '\n', 'Задача')">
            <svg class="ui-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="9 11 12 14 22 4"/>
              <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/>
            </svg>
          </button>

          <button class="tool-chip-btn" title="Таблица" @click="wrapSelection('\n| Параметр | Значение |\n|---|---|\n| Альфа | Омега |\n', '', '')">
            <svg class="ui-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
              <line x1="3" y1="9" x2="21" y2="9"/>
              <line x1="3" y1="15" x2="21" y2="15"/>
              <line x1="12" y1="3" x2="12" y2="21"/>
            </svg>
          </button>

          <button class="tool-chip-btn" title="Гиперссылка" @click="wrapSelection('[', '](https://example.com)', 'ссылка')">
            <svg class="ui-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
              <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
            </svg>
          </button>

          <button class="tool-chip-btn" title="Смена сцены (Разделитель)" @click="wrapSelection('\n\n---\n\n', '', '')">
            <svg class="ui-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="3" y1="12" x2="21" y2="12"/>
              <circle cx="12" cy="12" r="2" fill="currentColor"/>
            </svg>
          </button>
        </div>
      </div>

      <div class="editor-viewport-well">
        <textarea
          ref="textareaRef"
          v-model="content"
          class="quantum-textarea"
          placeholder="Начните писать сагу..."
          autofocus
        ></textarea>

        <aside v-if="showAnalytics" class="analytics-drawer">
          <header class="analytics-header">
            <h4>Линтер прозы</h4>
            <button class="btn-drawer-exit" @click="showAnalytics = false">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"/>
                <line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          </header>

          <div class="analytics-grid">
            <div class="analytic-stat-card">
              <span class="stat-caption">Диалоги</span>
              <span class="stat-value">{{ metrics.dialogueRatio.toFixed(1) }}%</span>
              <div class="stat-mini-bar">
                <div class="stat-mini-fill" :style="{ width: `${metrics.dialogueRatio}%` }"></div>
              </div>
            </div>

            <div class="analytic-stat-card">
              <span class="stat-caption">Лексика</span>
              <span class="stat-value">{{ metrics.vocabularyRichness.toFixed(1) }}%</span>
              <div class="stat-mini-bar">
                <div class="stat-mini-fill" :style="{ width: `${metrics.vocabularyRichness}%` }"></div>
              </div>
            </div>

            <div class="analytic-stat-card">
              <span class="stat-caption">Ритм предложений</span>
              <span class="stat-value">{{ metrics.sentenceCadence.toFixed(1) }}</span>
            </div>

            <div class="analytic-stat-card">
              <span class="stat-caption">Время чтения</span>
              <span class="stat-value">~{{ metrics.readingTimeMinutes.toFixed(1) }} мин</span>
            </div>
          </div>

          <div class="warnings-block">
            <span class="warnings-heading">Частые повторы</span>
            <div class="warning-tags">
              <span v-for="w in metrics.repetitionWarnings" :key="w" class="rep-tag">{{ w }}</span>
              <span v-if="metrics.repetitionWarnings.length === 0" class="rep-empty">Повторы не обнаружены</span>
            </div>
          </div>
        </aside>

        <aside v-if="showTimeline" class="snapshots-drawer">
          <header class="snapshots-header">
            <h4>Хронология ревизий</h4>
            <button class="btn-create-snap" @click="emit('makeSnapshot')">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <line x1="12" y1="5" x2="12" y2="19"/>
                <line x1="5" y1="12" x2="19" y2="12"/>
              </svg>
              <span>Снимок</span>
            </button>
          </header>

          <div class="snapshots-feed">
            <div
              v-for="snap in snapshots"
              :key="snap.id"
              class="snapshot-row-card"
              @click="emit('restoreSnapshot', snap)"
            >
              <div class="snap-title">{{ snap.title }}</div>
              <div class="snap-metrics">
                <span>{{ snap.wordCount }} сл.</span>
                <span>{{ new Date(snap.createdAt * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) }}</span>
              </div>
            </div>
            <div v-if="snapshots.length === 0" class="snapshots-empty">Нет сохраненных снимков</div>
          </div>
        </aside>
      </div>

      <footer class="editor-bottom-hud">
        <div class="hud-goal-track">
          <span class="hud-label">Слов: {{ metrics.wordCount }}</span>
          <span class="hud-label">Символов: {{ metrics.charCount }}</span>
        </div>
        <span class="hud-metric">Движок анализатора: Rust Threadpool</span>
      </footer>
    </div>

    <div v-if="studioMode === 'split'" class="preview-chassis">
      <div
        class="preview-parchment-sheet markdown-rendered"
        :style="{ fontSize: `${fontSize}px` }"
        v-html="renderedMarkdown"
      ></div>
    </div>
  </section>
</template>

<style scoped>
.quantum-workbench-studio {
  flex: 1;
  display: flex;
  overflow: hidden;
  height: 100%;
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
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  cursor: pointer;
}

.dual-tab-pill.active {
  background: var(--bg-card);
  border-color: var(--border);
  color: var(--text-pure);
}

.quantum-workbench-studio.mode-split .editor-chassis {
  flex: 1;
  border-right: 1px solid var(--border);
}

.quantum-workbench-studio.mode-split .preview-chassis {
  flex: 1;
}

.quantum-workbench-studio.mode-edit .editor-chassis {
  flex: 1;
  max-width: 1000px;
  margin: 0 auto;
}

.quantum-workbench-studio.mode-zen .editor-chassis {
  flex: 1;
  max-width: 820px;
  margin: 0 auto;
}

.editor-chassis {
  display: flex;
  flex-direction: column;
  padding: 18px 24px;
  gap: 12px;
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
  box-shadow: inset 0 2px 6px rgba(0, 0, 0, 0.5), var(--specular-high);
  color: var(--text-pure);
  padding: 0 16px;
  height: 42px;
  border-radius: 12px;
  font-size: 15px;
  font-weight: 700;
  outline: none;
  transition: border-color 0.2s ease;
}

.capsule-title-input:focus {
  border-color: var(--border-accent);
}

.capsule-category-input {
  flex: 1;
  background: var(--bg-card);
  border: 1px solid var(--border);
  box-shadow: var(--specular-high);
  color: var(--text-main);
  padding: 0 14px;
  height: 42px;
  border-radius: 12px;
  font-size: 13px;
  font-weight: 600;
  outline: none;
  transition: border-color 0.2s ease;
}

.capsule-category-input:focus {
  border-color: var(--border-accent);
}

.header-action-controls {
  display: flex;
  gap: 6px;
  align-items: center;
}

.ui-icon {
  flex-shrink: 0;
}

.btn-plasma-save {
  background: linear-gradient(135deg, var(--accent-indigo) 0%, #4338ca 100%);
  color: #fff;
  border: none;
  box-shadow: 0 4px 18px var(--accent-indigo-glow), var(--specular-prismatic);
  border-radius: 12px;
  padding: 0 16px;
  height: 42px;
  font-size: 13px;
  font-weight: 800;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

.btn-plasma-save:hover {
  transform: translateY(-1.5px);
  filter: brightness(1.1);
}

.btn-plasma-save:active {
  transform: scale(0.97);
}

.tactile-icon-btn {
  background: var(--bg-card);
  border: 1px solid var(--border);
  box-shadow: var(--specular-high);
  color: var(--text-muted);
  border-radius: 12px;
  width: 42px;
  height: 42px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

.tactile-icon-btn:hover {
  color: var(--text-pure);
  background: var(--bg-card-hover);
  border-color: var(--border-accent);
}

.tactile-icon-btn:active {
  transform: scale(0.95);
}

.tactile-icon-btn.active {
  background: var(--accent-indigo);
  border-color: var(--accent-indigo);
  color: #fff;
}

.btn-danger-tone:hover {
  background: rgba(244, 63, 94, 0.18);
  border-color: var(--accent-rose);
  color: var(--accent-rose);
}

.quantum-tools-strip {
  width: 100%;
  overflow-x: auto;
  padding-bottom: 2px;
}

.tools-horizontal-lane {
  display: flex;
  gap: 5px;
  width: max-content;
}

.tool-chip-btn {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--border);
  box-shadow: var(--specular-high);
  color: var(--text-muted);
  padding: 0 12px;
  height: 34px;
  border-radius: 9px;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  transition: all 0.15s ease;
}

.tool-chip-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-pure);
  border-color: var(--border-accent);
}

.tool-chip-btn:active {
  transform: scale(0.95);
}

.tool-label-text {
  font-size: 11px;
  font-weight: 800;
  letter-spacing: -0.01em;
}

.editor-viewport-well {
  flex: 1;
  position: relative;
  display: flex;
  overflow: hidden;
  background: var(--bg-card);
  border: 1px solid var(--border);
  box-shadow: inset 0 2px 10px rgba(0, 0, 0, 0.6), var(--specular-high);
  border-radius: 16px;
  padding: 20px;
}

.quantum-textarea {
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

.analytics-drawer {
  width: 300px;
  background: var(--bg-surface-solid);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  padding: 16px;
  border-radius: 0 16px 16px 0;
  gap: 16px;
}

.analytics-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.analytics-header h4 {
  font-size: 13px;
  font-weight: 800;
  color: var(--text-pure);
}

.btn-drawer-exit {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
}

.btn-drawer-exit:hover {
  color: var(--text-pure);
}

.analytics-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.analytic-stat-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  padding: 10px;
  border-radius: 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-caption {
  font-size: 10px;
  font-weight: 700;
  color: var(--text-dim);
  text-transform: uppercase;
}

.stat-value {
  font-family: var(--font-mono);
  font-size: 15px;
  font-weight: 800;
  color: var(--accent-cyan);
}

.stat-mini-bar {
  height: 3px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 999px;
  overflow: hidden;
}

.stat-mini-fill {
  height: 100%;
  background: var(--accent-cyan);
}

.warnings-block {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.warnings-heading {
  font-size: 11px;
  font-weight: 800;
  color: var(--text-muted);
  text-transform: uppercase;
}

.warning-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.rep-tag {
  background: rgba(244, 63, 94, 0.15);
  border: 1px solid rgba(244, 63, 94, 0.3);
  color: var(--accent-rose);
  font-size: 11px;
  font-family: var(--font-mono);
  padding: 3px 8px;
  border-radius: 6px;
}

.rep-empty {
  font-size: 11px;
  color: var(--text-dim);
}

.snapshots-drawer {
  width: 290px;
  background: var(--bg-surface-solid);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  padding: 16px;
  border-radius: 0 16px 16px 0;
}

.snapshots-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border);
}

.snapshots-header h4 {
  font-size: 13px;
  font-weight: 800;
  color: var(--text-pure);
}

.btn-create-snap {
  background: var(--accent-indigo);
  color: #fff;
  border: none;
  border-radius: 8px;
  padding: 6px 10px;
  font-size: 11px;
  font-weight: 800;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
}

.snapshots-feed {
  flex: 1;
  overflow-y: auto;
  padding: 10px 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.snapshot-row-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 10px 12px;
  cursor: pointer;
}

.snapshot-row-card:hover {
  border-color: var(--border-accent);
}

.snap-title {
  font-size: 12px;
  font-weight: 700;
  color: var(--text-pure);
}

.snap-metrics {
  display: flex;
  justify-content: space-between;
  font-size: 10px;
  color: var(--text-muted);
  font-family: var(--font-mono);
  margin-top: 4px;
}

.snapshots-empty {
  font-size: 12px;
  color: var(--text-dim);
  text-align: center;
  padding: 30px 0;
}

.editor-bottom-hud {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 4px 0 4px;
  font-size: 11px;
}

.hud-goal-track {
  display: flex;
  align-items: center;
  gap: 14px;
}

.hud-label {
  color: var(--text-dim);
  font-weight: 600;
}

.hud-metric {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
}

.preview-chassis {
  height: 100%;
  overflow-y: auto;
  padding: 32px 42px;
  background: rgba(0, 0, 0, 0.35);
  user-select: text;
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

  .btn-plasma-save {
    flex: 1;
    justify-content: center;
  }

  .analytics-drawer,
  .snapshots-drawer {
    position: absolute;
    inset: 0;
    width: 100%;
    z-index: 30;
    border-radius: 16px;
  }

  .quantum-workbench-studio.mobile-tab-editor .preview-chassis {
    display: none;
  }

  .quantum-workbench-studio.mobile-tab-editor .editor-chassis {
    display: flex;
  }

  .quantum-workbench-studio.mobile-tab-preview .editor-chassis {
    display: none;
  }

  .quantum-workbench-studio.mobile-tab-preview .preview-chassis {
    display: block;
    width: 100%;
    padding: 18px 12px;
  }
}
</style>