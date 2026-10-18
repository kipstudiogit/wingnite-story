import { ref, shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { EnterpriseStoryItem } from "../types";

export interface ViewportSlice {
  lines: string[];
  startLine: number;
  totalLines: number;
  totalChars: number;
  totalWords: number;
  revisionHash: string;
}

export interface RopeEditPayload {
  storyId: string;
  charIdx: number;
  insertText?: string;
  deleteLen?: number;
}

export function useRopeEditor(storyIdGetter: () => string) {
  const visibleLines = shallowRef<string[]>([]);
  const startLine = ref<number>(0);
  const lineCount = ref<number>(60);
  const totalLines = ref<number>(0);
  const totalChars = ref<number>(0);
  const totalWords = ref<number>(0);
  const revisionHash = ref<string>("");
  const isSyncing = ref<boolean>(false);
  const editorError = ref<string | null>(null);

  async function initSession(initialContent: string): Promise<void> {
    const storyId = storyIdGetter();
    if (!storyId) return;

    isSyncing.value = true;
    editorError.value = null;

    try {
      const slice = await invoke<ViewportSlice>("rope_init_session", {
        storyId,
        initialContent,
      });

      visibleLines.value = slice.lines;
      startLine.value = slice.startLine;
      totalLines.value = slice.totalLines;
      totalChars.value = slice.totalChars;
      totalWords.value = slice.totalWords;
      revisionHash.value = slice.revisionHash;
    } catch (err) {
      editorError.value = String(err);
    } finally {
      isSyncing.value = false;
    }
  }

  async function requestViewport(newStartLine: number, newLineCount = 60): Promise<void> {
    const storyId = storyIdGetter();
    if (!storyId) return;

    try {
      const slice = await invoke<ViewportSlice>("rope_get_viewport", {
        storyId,
        startLine: newStartLine,
        lineCount: newLineCount,
      });

      visibleLines.value = slice.lines;
      startLine.value = slice.startLine;
      totalLines.value = slice.totalLines;
      totalChars.value = slice.totalChars;
      totalWords.value = slice.totalWords;
      revisionHash.value = slice.revisionHash;
    } catch (err) {
      editorError.value = String(err);
    }
  }

  async function applyInsert(charIdx: number, insertText: string): Promise<void> {
    const storyId = storyIdGetter();
    if (!storyId) return;

    const payload: RopeEditPayload = {
      storyId,
      charIdx,
      insertText,
    };

    try {
      const slice = await invoke<ViewportSlice>("rope_apply_delta", { payload });
      visibleLines.value = slice.lines;
      startLine.value = slice.startLine;
      totalLines.value = slice.totalLines;
      totalChars.value = slice.totalChars;
      totalWords.value = slice.totalWords;
      revisionHash.value = slice.revisionHash;
    } catch (err) {
      editorError.value = String(err);
    }
  }

  async function applyDelete(charIdx: number, deleteLen: number): Promise<void> {
    const storyId = storyIdGetter();
    if (!storyId) return;

    const payload: RopeEditPayload = {
      storyId,
      charIdx,
      deleteLen,
    };

    try {
      const slice = await invoke<ViewportSlice>("rope_apply_delta", { payload });
      visibleLines.value = slice.lines;
      startLine.value = slice.startLine;
      totalLines.value = slice.totalLines;
      totalChars.value = slice.totalChars;
      totalWords.value = slice.totalWords;
      revisionHash.value = slice.revisionHash;
    } catch (err) {
      editorError.value = String(err);
    }
  }

  async function flushToStorage(
    title: string,
    category: string,
    operator = "Lead-Editor"
  ): Promise<EnterpriseStoryItem | null> {
    const storyId = storyIdGetter();
    if (!storyId) return null;

    isSyncing.value = true;
    editorError.value = null;

    try {
      const saved = await invoke<EnterpriseStoryItem>("rope_flush_to_storage", {
        storyId,
        title,
        category,
        operator,
      });
      revisionHash.value = saved.contentHash;
      return saved;
    } catch (err) {
      editorError.value = String(err);
      return null;
    } finally {
      isSyncing.value = false;
    }
  }

  return {
    visibleLines,
    startLine,
    lineCount,
    totalLines,
    totalChars,
    totalWords,
    revisionHash,
    isSyncing,
    editorError,
    initSession,
    requestViewport,
    applyInsert,
    applyDelete,
    flushToStorage,
  };
}