// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  EnterpriseStoryItem,
  SaveStoryPayload,
  StoryOrderUpdate,
  FtsResultItem,
  ProjectItem,
  EntityKind,
} from "../types";
import { useHaptics } from "./useHaptics";
import { useI18n } from "../i18n";

const stories = ref<EnterpriseStoryItem[]>([]);
const activeId = ref<string>("");
const currentTitle = ref<string>("");
const currentContent = ref<string>("");
const currentCategory = ref<string>("Act I");
const currentRevision = ref<number>(0);
const currentSortOrder = ref<number>(0);
const activeOperator = ref<string>("Lead-Editor-Console");
const isSyncing = ref<boolean>(false);
const syncError = ref<string | null>(null);

const savedBaseline = ref({
  id: "",
  title: "",
  category: "Act I",
  content: "",
});

function calculateWords(text: string): number {
  let count = 0;
  let inWord = false;
  for (let i = 0; i < text.length; i++) {
    if (text.charCodeAt(i) > 32) {
      if (!inWord) {
        inWord = true;
        count++;
      }
    } else {
      inWord = false;
    }
  }
  return count;
}

export function useEnterpriseStory() {
  const { vibrateSuccess, vibrateWarning } = useHaptics();
  const { t } = useI18n();

  const activeStory = computed(() =>
    stories.value.find((s) => s.id === activeId.value)
  );

  const isDirty = computed(() => {
    return (
      activeId.value !== savedBaseline.value.id ||
      currentTitle.value !== savedBaseline.value.title ||
      currentCategory.value !== savedBaseline.value.category ||
      currentContent.value !== savedBaseline.value.content
    );
  });

  const currentWords = computed(() => calculateWords(currentContent.value));

  const totalSystemWords = computed(() => {
    let count = 0;
    for (const s of stories.value) {
      if (s.id === activeId.value) {
        count += currentWords.value;
      } else {
        count += calculateWords(s.content);
      }
    }
    return count;
  });

  function setBaseline(id: string, title: string, category: string, content: string) {
    savedBaseline.value = { id, title, category, content };
  }

  function applyServerSavedStory(updated: EnterpriseStoryItem) {
    activeId.value = updated.id;
    currentTitle.value = updated.title;
    currentContent.value = updated.content;
    currentCategory.value = updated.category;
    currentRevision.value = updated.revision;
    currentSortOrder.value = updated.sortOrder;
    setBaseline(updated.id, updated.title, updated.category, updated.content);

    const idx = stories.value.findIndex((s) => s.id === updated.id);
    if (idx !== -1) {
      stories.value[idx] = updated;
    } else {
      stories.value.push(updated);
    }
  }

  function registerExternalStory(story: EnterpriseStoryItem) {
    const idx = stories.value.findIndex((s) => s.id === story.id);
    if (idx === -1) {
      stories.value.push(story);
    } else {
      stories.value[idx] = story;
    }
  }

  async function loadSystemStories(projectId: string): Promise<void> {
    isSyncing.value = true;
    syncError.value = null;
    try {
      const records = await invoke<EnterpriseStoryItem[]>("fetch_stories", {
        projectId,
      });
      stories.value = records;
      if (records.length > 0) {
        selectEnterpriseStory(records[0]);
      } else {
        initializeBlankDraft();
      }
    } catch (err) {
      syncError.value = String(err);
    } finally {
      isSyncing.value = false;
    }
  }

  function selectEnterpriseStory(story: EnterpriseStoryItem): void {
    activeId.value = story.id;
    currentTitle.value = story.title;
    currentContent.value = story.content;
    currentCategory.value = story.category;
    currentRevision.value = story.revision;
    currentSortOrder.value = story.sortOrder;
    setBaseline(story.id, story.title, story.category, story.content);
  }

  async function persistStoryMutation(projectId: string): Promise<boolean> {
    if (isSyncing.value) return false;
    isSyncing.value = true;
    syncError.value = null;

    const payload: SaveStoryPayload = {
      id: activeId.value,
      projectId,
      title: currentTitle.value.trim() || t("studio.defaultSceneTitle"),
      content: currentContent.value,
      category: currentCategory.value.trim() || t("studio.defaultCategory"),
      expectedRevision: currentRevision.value,
      sortOrder: currentSortOrder.value,
      operator: activeOperator.value,
    };

    try {
      const updated = await invoke<EnterpriseStoryItem>("mutate_story", { payload });
      applyServerSavedStory(updated);
      vibrateSuccess();
      return true;
    } catch (err) {
      syncError.value = String(err);
      vibrateWarning();
      return false;
    } finally {
      isSyncing.value = false;
    }
  }

  async function persistSpecificStory(
    story: EnterpriseStoryItem,
    fallbackProjectId: string
  ): Promise<EnterpriseStoryItem | null> {
    if (isSyncing.value) return null;
    isSyncing.value = true;
    syncError.value = null;

    const payload: SaveStoryPayload = {
      id: story.id,
      projectId: story.projectId || fallbackProjectId,
      title: story.title.trim() || t("studio.defaultSceneTitle"),
      content: story.content,
      category: story.category.trim() || t("studio.defaultCategory"),
      expectedRevision: story.revision,
      sortOrder: story.sortOrder,
      operator: activeOperator.value,
    };

    try {
      const updated = await invoke<EnterpriseStoryItem>("mutate_story", { payload });
      const idx = stories.value.findIndex((s) => s.id === updated.id);
      if (idx !== -1) {
        stories.value[idx] = updated;
      } else {
        stories.value.push(updated);
      }

      if (activeId.value === updated.id) {
        applyServerSavedStory(updated);
      }

      vibrateSuccess();
      return updated;
    } catch (err) {
      syncError.value = String(err);
      vibrateWarning();
      return null;
    } finally {
      isSyncing.value = false;
    }
  }

  async function deleteSystemStory(id: string): Promise<boolean> {
    if (!id) return false;
    isSyncing.value = true;
    syncError.value = null;

    try {
      await invoke("delete_story", { id, operator: activeOperator.value });
      stories.value = stories.value.filter((s) => s.id !== id);
      if (activeId.value === id) {
        if (stories.value.length > 0) {
          selectEnterpriseStory(stories.value[0]);
        } else {
          initializeBlankDraft();
        }
      }
      vibrateSuccess();
      return true;
    } catch (err) {
      syncError.value = String(err);
      vibrateWarning();
      return false;
    } finally {
      isSyncing.value = false;
    }
  }

  async function batchReorderStories(updates: StoryOrderUpdate[]): Promise<void> {
    try {
      await invoke("reorder_stories", { updates });
      for (const u of updates) {
        const found = stories.value.find((s) => s.id === u.id);
        if (found) {
          found.sortOrder = u.sortOrder;
          if (u.category) found.category = u.category;
        }
      }
      stories.value.sort((a, b) => a.sortOrder - b.sortOrder);
      const activeFound = stories.value.find((s) => s.id === activeId.value);
      if (activeFound) {
        currentSortOrder.value = activeFound.sortOrder;
        currentCategory.value = activeFound.category;
      }
      vibrateSuccess();
    } catch (err) {
      syncError.value = String(err);
      vibrateWarning();
    }
  }

  async function swapStoryPairNormalized(idA: string, idB: string): Promise<void> {
    const ordered = [...stories.value].sort((a, b) => a.sortOrder - b.sortOrder);
    const idxA = ordered.findIndex((s) => s.id === idA);
    const idxB = ordered.findIndex((s) => s.id === idB);
    if (idxA === -1 || idxB === -1) return;

    const temp = ordered[idxA];
    ordered[idxA] = ordered[idxB];
    ordered[idxB] = temp;

    const updates: StoryOrderUpdate[] = ordered.map((item, idx) => ({
      id: item.id,
      sortOrder: idx,
      category: item.category,
    }));

    await batchReorderStories(updates);
  }

  async function moveStoryToCategory(storyId: string, targetCategory: string): Promise<void> {
    const ordered = [...stories.value].sort((a, b) => a.sortOrder - b.sortOrder);
    const maxOrder = ordered.reduce((acc, s) => Math.max(acc, s.sortOrder), -1);
    await batchReorderStories([
      {
        id: storyId,
        sortOrder: maxOrder + 1,
        category: targetCategory,
      },
    ]);
  }

  async function performIndexedSearch(query: string, projectId: string): Promise<FtsResultItem[]> {
    try {
      return await invoke<FtsResultItem[]>("execute_fts_search", { query, projectId });
    } catch (err) {
      syncError.value = String(err);
      return [];
    }
  }

  async function exportProjectWstory(projectId: string): Promise<string> {
    return await invoke<string>("export_project_wstory", { projectId });
  }

  async function importProjectWstory(packageRaw: string): Promise<ProjectItem> {
    return await invoke<ProjectItem>("import_project_wstory", {
      packageRaw,
      operator: activeOperator.value,
    });
  }

  function initializeBlankDraft(
    category?: string,
    title = "",
    template = ""
  ): void {
    const resolvedCategory = category ?? t("studio.defaultCategory");
    activeId.value = "";
    currentTitle.value = title;
    currentCategory.value = resolvedCategory;
    currentContent.value = template;
    currentRevision.value = 0;
    currentSortOrder.value = stories.value.length;
    setBaseline("", title, resolvedCategory, template);
  }

  function createEntityDraft(type: EntityKind): void {
    if (type === "character") {
      initializeBlankDraft(
        t("codex.creator.categories.character"),
        t("codex.creator.templates.characterDefaultTitle"),
        t("codex.creator.templates.characterTemplate")
      );
    } else if (type === "artifact") {
      initializeBlankDraft(
        t("codex.creator.categories.artifact"),
        t("codex.creator.templates.artifactDefaultTitle"),
        t("codex.creator.templates.artifactTemplate")
      );
    } else if (type === "pantheon") {
      initializeBlankDraft(
        t("codex.creator.categories.pantheon"),
        t("codex.creator.templates.pantheonDefaultTitle"),
        t("codex.creator.templates.pantheonTemplate")
      );
    } else {
      initializeBlankDraft(
        t("codex.creator.categories.sanctuary"),
        t("codex.creator.templates.sanctuaryDefaultTitle"),
        t("codex.creator.templates.sanctuaryTemplate")
      );
    }
  }

  return {
    stories,
    activeId,
    currentTitle,
    currentContent,
    currentCategory,
    currentRevision,
    currentSortOrder,
    activeOperator,
    isSyncing,
    isDirty,
    syncError,
    activeStory,
    currentWords,
    totalSystemWords,
    applyServerSavedStory,
    registerExternalStory,
    loadSystemStories,
    selectEnterpriseStory,
    persistStoryMutation,
    persistSpecificStory,
    deleteSystemStory,
    batchReorderStories,
    swapStoryPairNormalized,
    moveStoryToCategory,
    performIndexedSearch,
    exportProjectWstory,
    importProjectWstory,
    initializeBlankDraft,
    createEntityDraft,
  };
}