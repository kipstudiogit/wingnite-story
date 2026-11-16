// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

import { ref, shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { EnterpriseStoryItem, RopeEditPayload, ViewportSlice } from "../types";

function isHighSurrogate(code: number): boolean {
  return code >= 0xd800 && code <= 0xdbff;
}

function isLowSurrogate(code: number): boolean {
  return code >= 0xdc00 && code <= 0xdfff;
}

function countCodePoints(str: string): number {
  let count = 0;
  for (let i = 0; i < str.length; i++) {
    if (
      isHighSurrogate(str.charCodeAt(i)) &&
      i + 1 < str.length &&
      isLowSurrogate(str.charCodeAt(i + 1))
    ) {
      i++;
    }
    count++;
  }
  return count;
}

function computeMinimalDelta(prevText: string, nextText: string) {
  let prefix = 0;
  const minLen = Math.min(prevText.length, nextText.length);
  while (prefix < minLen && prevText.charCodeAt(prefix) === nextText.charCodeAt(prefix)) {
    prefix++;
  }
  if (prefix > 0 && isHighSurrogate(prevText.charCodeAt(prefix - 1))) {
    prefix--;
  }

  let suffix = 0;
  while (
    suffix < minLen - prefix &&
    prevText.charCodeAt(prevText.length - 1 - suffix) ===
      nextText.charCodeAt(nextText.length - 1 - suffix)
  ) {
    suffix++;
  }
  if (suffix > 0 && isLowSurrogate(prevText.charCodeAt(prevText.length - suffix))) {
    suffix--;
  }

  const charIdx = countCodePoints(prevText.slice(0, prefix));
  const deleteLen = countCodePoints(prevText.slice(prefix, prevText.length - suffix));
  const insertText = nextText.slice(prefix, nextText.length - suffix);

  return { charIdx, deleteLen, insertText };
}

export function useRopeEditor(
  projectIdGetter: () => string,
  storyIdGetter: () => string
) {
  const visibleLines = shallowRef<string[]>([]);
  const startLine = ref<number>(0);
  const totalLines = ref<number>(1);
  const totalChars = ref<number>(0);
  const totalWords = ref<number>(0);
  const revisionHash = ref<string>("");
  const isSyncing = ref<boolean>(false);
  const editorError = ref<string | null>(null);

  const activeSessionKey = ref<string>("__draft__");
  let syncedText = "";
  let syncQueue: Promise<void> = Promise.resolve();

  function applySliceState(slice: ViewportSlice) {
    visibleLines.value = slice.lines;
    startLine.value = slice.startLine;
    totalLines.value = slice.totalLines;
    totalChars.value = slice.totalChars;
    totalWords.value = slice.totalWords;
    revisionHash.value = slice.revisionHash;
  }

  async function initSession(initialContent: string, explicitKey?: string): Promise<void> {
    const nextKey = explicitKey ?? (storyIdGetter() || "__draft__");
    const prevKey = activeSessionKey.value;
    activeSessionKey.value = nextKey;
    syncedText = initialContent;
    isSyncing.value = true;
    editorError.value = null;

    syncQueue = syncQueue.then(async () => {
      try {
        if (prevKey && prevKey !== nextKey) {
          await invoke("rope_close_session", { sessionKey: prevKey }).catch(() => {});
        }
        const slice = await invoke<ViewportSlice>("rope_init_session", {
          sessionKey: nextKey,
          initialContent,
        });
        applySliceState(slice);
      } catch (err) {
        editorError.value = String(err);
      } finally {
        isSyncing.value = false;
      }
    });

    return syncQueue;
  }

  function syncTextDelta(nextText: string): Promise<void> {
    if (nextText === syncedText) {
      return syncQueue;
    }

    const sessionKey = activeSessionKey.value;
    const prevSnapshot = syncedText;
    syncedText = nextText;

    syncQueue = syncQueue.then(async () => {
      const { charIdx, deleteLen, insertText } = computeMinimalDelta(prevSnapshot, nextText);
      const payload: RopeEditPayload = {
        sessionKey,
        charIdx,
        deleteLen: deleteLen > 0 ? deleteLen : undefined,
        insertText: insertText.length > 0 ? insertText : undefined,
      };

      try {
        const slice = await invoke<ViewportSlice>("rope_apply_delta", { payload });
        applySliceState(slice);
      } catch {
        try {
          const slice = await invoke<ViewportSlice>("rope_init_session", {
            sessionKey,
            initialContent: nextText,
          });
          applySliceState(slice);
        } catch (err) {
          editorError.value = String(err);
        }
      }
    });

    return syncQueue;
  }

  async function requestViewport(newStartLine: number, newLineCount = 60): Promise<void> {
    try {
      const slice = await invoke<ViewportSlice>("rope_get_viewport", {
        sessionKey: activeSessionKey.value,
        startLine: newStartLine,
        lineCount: newLineCount,
      });
      applySliceState(slice);
    } catch (err) {
      editorError.value = String(err);
    }
  }

  async function flushToStorage(
    title: string,
    category: string,
    operator = "Lead-Editor"
  ): Promise<EnterpriseStoryItem | null> {
    isSyncing.value = true;
    editorError.value = null;

    await syncQueue;

    try {
      const saved = await invoke<EnterpriseStoryItem>("rope_flush_to_storage", {
        sessionKey: activeSessionKey.value,
        storyId: storyIdGetter(),
        projectId: projectIdGetter(),
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

  async function closeActiveSession(): Promise<void> {
    try {
      await invoke("rope_close_session", { sessionKey: activeSessionKey.value });
    } catch {
    }
  }

  return {
    visibleLines,
    startLine,
    totalLines,
    totalChars,
    totalWords,
    revisionHash,
    activeSessionKey,
    isSyncing,
    editorError,
    initSession,
    syncTextDelta,
    requestViewport,
    flushToStorage,
    closeActiveSession,
  };
}