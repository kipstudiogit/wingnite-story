import { ref, computed, shallowRef } from "vue";
import { invoke, Channel } from "@tauri-apps/api/core";
import type {
  EnterpriseStoryItem,
  SaveStoryPayload,
  StoryOrderUpdate,
  AuditLogEntry,
  FtsResultItem,
  ProjectItem,
} from "../types";
import { useHaptics } from "./useHaptics";

const stories = ref<EnterpriseStoryItem[]>([]);
const auditLogs = shallowRef<AuditLogEntry[]>([]);
const activeId = ref<string>("");
const currentTitle = ref<string>("");
const currentContent = ref<string>("");
const currentCategory = ref<string>("Акт I");
const currentRevision = ref<number>(0);
const currentSortOrder = ref<number>(0);
const activeOperator = ref<string>("Lead-Editor-Console");
const isSyncing = ref<boolean>(false);
const syncError = ref<string | null>(null);

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

  const activeStory = computed(() =>
    stories.value.find((s) => s.id === activeId.value)
  );

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
  }

  async function persistStoryMutation(projectId: string): Promise<boolean> {
    if (isSyncing.value) return false;
    isSyncing.value = true;
    syncError.value = null;

    const payload: SaveStoryPayload = {
      id: activeId.value,
      projectId,
      title: currentTitle.value.trim() || "Новая сцена",
      content: currentContent.value,
      category: currentCategory.value.trim() || "Акт I",
      expectedRevision: currentRevision.value,
      sortOrder: currentSortOrder.value,
      operator: activeOperator.value,
    };

    try {
      const updated = await invoke<EnterpriseStoryItem>("mutate_story", { payload });
      activeId.value = updated.id;
      currentTitle.value = updated.title;
      currentContent.value = updated.content;
      currentCategory.value = updated.category;
      currentRevision.value = updated.revision;
      currentSortOrder.value = updated.sortOrder;

      const idx = stories.value.findIndex((s) => s.id === updated.id);
      if (idx !== -1) {
        stories.value[idx] = updated;
      } else {
        stories.value.push(updated);
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
      vibrateSuccess();
    } catch (err) {
      syncError.value = String(err);
    }
  }

  async function startAuditStream(): Promise<void> {
    const channel = new Channel<AuditLogEntry>();
    const collected: AuditLogEntry[] = [];

    channel.onmessage = (entry: AuditLogEntry) => {
      collected.push(entry);
      auditLogs.value = [...collected];
    };

    try {
      await invoke("stream_audit_trail", { channel });
    } catch (err) {
      syncError.value = String(err);
    }
  }

  async function performIndexedSearch(query: string): Promise<FtsResultItem[]> {
    try {
      return await invoke<FtsResultItem[]>("execute_fts_search", { query });
    } catch (err) {
      syncError.value = String(err);
      return [];
    }
  }

  async function exportProjectWstory(projectId: string): Promise<string> {
    return await invoke<string>("export_project_wstory", { projectId });
  }

  async function importProjectWstory(packageRaw: string): Promise<ProjectItem> {
    const imported = await invoke<ProjectItem>("import_project_wstory", {
      packageRaw,
      operator: activeOperator.value,
    });
    return imported;
  }

  function initializeBlankDraft(
    category = "Акт I",
    title = "",
    template = ""
  ): void {
    activeId.value = "";
    currentTitle.value = title;
    currentCategory.value = category;
    currentContent.value = template;
    currentRevision.value = 0;
    currentSortOrder.value = stories.value.length;
  }

  function createEntityDraft(type: "artifact" | "pantheon" | "sanctuary" | "character"): void {
    if (type === "character") {
      initializeBlankDraft(
        "Персонаж",
        "Новый Персонаж",
        "### Досье Персонажа\n* **Класс:** Протагонист / Искатель\n* **Лояльность:** Орден\n* **Статус:** Жив\n* **Мотивация:** Поиск истины\n\nБиография..."
      );
    } else if (type === "artifact") {
      initializeBlankDraft(
        "Божественный Артефакт",
        "Новая Реликвия",
        "### Реликвия\n* **Класс:** Древний Артефакт\n* **Владелец:** Неизвестно\n* **Свойство:** Манипуляция энергией\n* **Уязвимость:** Неуязвим\n\nИстория..."
      );
    } else if (type === "pantheon") {
      initializeBlankDraft(
        "Демиург / Сущность",
        "Новый Демиург",
        "### Высшая Сущность\n* **Ранг:** Архитектор\n* **Статус:** Бессмертен\n* **Облик:** Созвездие\n* **Уязвимость:** Искажение ритма\n\nОписание..."
      );
    } else {
      initializeBlankDraft(
        "Святилище Сил",
        "Новая Локация",
        "### Локация\n* **Регион:** Расколотые земли\n* **Атмосфера:** Аномалии\n* **Особенность:** Древний портал\n\nОписание..."
      );
    }
  }

  return {
    stories,
    auditLogs,
    activeId,
    currentTitle,
    currentContent,
    currentCategory,
    currentRevision,
    currentSortOrder,
    activeOperator,
    isSyncing,
    syncError,
    activeStory,
    currentWords,
    totalSystemWords,
    loadSystemStories,
    selectEnterpriseStory,
    persistStoryMutation,
    deleteSystemStory,
    batchReorderStories,
    startAuditStream,
    performIndexedSearch,
    exportProjectWstory,
    importProjectWstory,
    initializeBlankDraft,
    createEntityDraft,
  };
}