import { ref, shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface LspSuggestion {
  id: string;
  title: string;
  category: string;
  preview: string;
  linkType: string;
  matchScore: number;
}

export interface LspHoverInfo {
  id: string;
  title: string;
  category: string;
  summary: string;
  connectedEntities: string[];
  revision: number;
}

export interface LspQueryPayload {
  storyId: string;
  rawToken: string;
  triggerChar: string;
}

export interface WikiLinkResolvePayload {
  sourceStoryId: string;
  targetTitle: string;
  operator: string;
}

export interface WikiLinkResolveResult {
  targetId: string;
  targetTitle: string;
  isNewEntity: boolean;
  relationId: string;
}

export function useNarrativeLsp(sourceStoryIdGetter: () => string) {
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
  ): Promise<{ nextText: string; nextCursor: number }> {
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
      sourceStoryId: sourceStoryIdGetter(),
      targetTitle: suggestion.title,
      operator,
    };

    try {
      await invoke<WikiLinkResolveResult>("lsp_resolve_wikilink", { payload: resolvePayload });
    } catch {}

    return { nextText, nextCursor };
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
      });

      if (data) {
        hoverInfo.value = data;
        hoverCoordinates.value = { x: clientX + 12, y: clientY + 16 };
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