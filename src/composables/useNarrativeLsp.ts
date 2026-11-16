// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

import { ref, shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  EnterpriseStoryItem,
  LspHoverInfo,
  LspQueryPayload,
  LspSuggestion,
  WikiLinkResolvePayload,
  WikiLinkResolveResult,
} from "../types";

export function useNarrativeLsp(
  projectIdGetter: () => string,
  sourceStoryIdGetter: () => string
) {
  const isSuggestionsOpen = ref(false);
  const suggestions = shallowRef<LspSuggestion[]>([]);
  const selectedIndex = ref(0);
  const triggerType = ref<"wikilink" | "mention" | null>(null);
  const tokenStartIndex = ref(-1);
  const currentToken = ref("");

  const isHoverActive = ref(false);
  const hoverInfo = shallowRef<LspHoverInfo | null>(null);
  const hoverCoordinates = ref({ x: 0, y: 0 });

  let lspDebounce: number | null = null;

  function evaluateCursorContext(fullText: string, cursorOffset: number) {
    if (cursorOffset < 1) {
      closeSuggestions();
      return;
    }

    const textBeforeCursor = fullText.slice(0, cursorOffset);
    const lastDoubleBracket = textBeforeCursor.lastIndexOf("[[");
    const lastAtSign = textBeforeCursor.lastIndexOf("@");

    if (lastDoubleBracket !== -1 && lastDoubleBracket > textBeforeCursor.lastIndexOf("]]")) {
      const token = textBeforeCursor.slice(lastDoubleBracket + 2);
      if (!token.includes("\n") && token.length <= 40) {
        triggerType.value = "wikilink";
        tokenStartIndex.value = lastDoubleBracket;
        currentToken.value = token;
        requestAutocomplete(token, "[[");
        return;
      }
    }

    if (lastAtSign !== -1 && lastAtSign > textBeforeCursor.lastIndexOf(" ")) {
      const token = textBeforeCursor.slice(lastAtSign + 1);
      if (!token.includes("\n") && token.length <= 30) {
        triggerType.value = "mention";
        tokenStartIndex.value = lastAtSign;
        currentToken.value = token;
        requestAutocomplete(token, "@");
        return;
      }
    }

    closeSuggestions();
  }

  function requestAutocomplete(rawToken: string, triggerChar: string) {
    if (lspDebounce) clearTimeout(lspDebounce);
    lspDebounce = window.setTimeout(async () => {
      const payload: LspQueryPayload = {
        projectId: projectIdGetter(),
        storyId: sourceStoryIdGetter(),
        rawToken,
        triggerChar,
      };

      try {
        const results = await invoke<LspSuggestion[]>("lsp_complete_wikilink", { payload });
        suggestions.value = results;
        selectedIndex.value = 0;
        isSuggestionsOpen.value = results.length > 0;
      } catch {
        suggestions.value = [];
        isSuggestionsOpen.value = false;
      }
    }, 70);
  }

  function closeSuggestions() {
    isSuggestionsOpen.value = false;
    suggestions.value = [];
    triggerType.value = null;
    tokenStartIndex.value = -1;
    currentToken.value = "";
  }

  async function applySuggestion(
    suggestion: LspSuggestion,
    fullText: string,
    cursorOffset: number,
    operator = "Lsp-Operator"
  ): Promise<{
    nextText: string;
    nextCursor: number;
    createdStory: EnterpriseStoryItem | null;
  }> {
    const start = tokenStartIndex.value;
    const isWiki = triggerType.value === "wikilink";
    const replacement = isWiki
      ? `[[${suggestion.title}]]`
      : `@${suggestion.title} `;

    const before = fullText.slice(0, start);
    const after = fullText.slice(cursorOffset);

    const nextText = `${before}${replacement}${after}`;
    const nextCursor = start + replacement.length;

    closeSuggestions();

    const resolvePayload: WikiLinkResolvePayload = {
      projectId: projectIdGetter(),
      sourceStoryId: sourceStoryIdGetter(),
      targetTitle: suggestion.title,
      operator,
    };

    let createdStory: EnterpriseStoryItem | null = null;
    try {
      const result = await invoke<WikiLinkResolveResult>("lsp_resolve_wikilink", {
        payload: resolvePayload,
      });
      createdStory = result.createdStory ?? null;
    } catch {
    }

    return { nextText, nextCursor, createdStory };
  }

  function handleKeydownNavigation(e: KeyboardEvent): boolean {
    if (!isSuggestionsOpen.value || suggestions.value.length === 0) {
      return false;
    }

    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex.value = (selectedIndex.value + 1) % suggestions.value.length;
      return true;
    }

    if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex.value =
        (selectedIndex.value - 1 + suggestions.value.length) % suggestions.value.length;
      return true;
    }

    if (e.key === "Escape") {
      e.preventDefault();
      closeSuggestions();
      return true;
    }

    return false;
  }

  async function inspectHoverWord(word: string, clientX: number, clientY: number) {
    const cleaned = word.replace(/[[\]@]/g, "").trim();
    if (cleaned.length < 2) {
      dismissHover();
      return;
    }

    try {
      const data = await invoke<LspHoverInfo | null>("lsp_hover_entity", {
        entityTitle: cleaned,
        projectId: projectIdGetter(),
      });

      if (data) {
        const clampedX = Math.min(window.innerWidth - 300, Math.max(12, clientX + 12));
        const clampedY = Math.min(window.innerHeight - 180, Math.max(12, clientY + 16));
        hoverInfo.value = data;
        hoverCoordinates.value = { x: clampedX, y: clampedY };
        isHoverActive.value = true;
      } else {
        dismissHover();
      }
    } catch {
      dismissHover();
    }
  }

  function dismissHover() {
    isHoverActive.value = false;
    hoverInfo.value = null;
  }

  return {
    isSuggestionsOpen,
    suggestions,
    selectedIndex,
    triggerType,
    isHoverActive,
    hoverInfo,
    hoverCoordinates,
    evaluateCursorContext,
    applySuggestion,
    handleKeydownNavigation,
    inspectHoverWord,
    dismissHover,
    closeSuggestions,
  };
}