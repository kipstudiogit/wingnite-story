import { ref, computed, shallowRef } from "vue";
import { invoke, Channel } from "@tauri-apps/api/core";
import type {
  EnterpriseStoryItem,
  SaveStoryPayload,
  AuditLogEntry,
  FtsResultItem,
  ProjectItem,
  SaveProjectPayload,
} from "../types";

const projects = ref<ProjectItem[]>([]);
const activeProjectId = ref<string>("default_project");
const stories = ref<EnterpriseStoryItem[]>([]);
const auditLogs = shallowRef<AuditLogEntry[]>([]);
const activeId = ref<string>("");
const currentTitle = ref<string>("");
const currentContent = ref<string>("");
const currentCategory = ref<string>("");
const currentRevision = ref<number>(0);
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

const activeProject = computed(() =>
  projects.value.find((p) => p.id === activeProjectId.value)
);

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

async function loadSystemProjects(): Promise<void> {
  try {
    const list = await invoke<ProjectItem[]>("fetch_projects");
    projects.value = list;
    if (list.length > 0 && !projects.value.some((p) => p.id === activeProjectId.value)) {
      activeProjectId.value = list[0].id;
    }
  } catch (err) {
    syncError.value = String(err);
  }
}

async function createProject(title: string, description: string): Promise<ProjectItem | null> {
  isSyncing.value = true;
  syncError.value = null;
  try {
    const payload: SaveProjectPayload = {
      title: title.trim() || "Новое произведение",
      description: description.trim() || "Архивная книга",
    };
    const saved = await invoke<ProjectItem>("mutate_project", { payload });
    projects.value.unshift(saved);
    await switchProject(saved.id);
    return saved;
  } catch (err) {
    syncError.value = String(err);
    return null;
  } finally {
    isSyncing.value = false;
  }
}

async function deleteActiveProject(id: string): Promise<boolean> {
  if (projects.value.length <= 1) {
    syncError.value = "Нельзя удалить единственное оставшееся произведение.";
    return false;
  }

  isSyncing.value = true;
  syncError.value = null;
  try {
    await invoke("delete_project", { id });
    projects.value = projects.value.filter((p) => p.id !== id);
    const fallbackId = projects.value[0]?.id || "default_project";
    await switchProject(fallbackId);
    return true;
  } catch (err) {
    syncError.value = String(err);
    return false;
  } finally {
    isSyncing.value = false;
  }
}

async function switchProject(projectId: string): Promise<void> {
  activeProjectId.value = projectId;
  activeId.value = "";
  await loadSystemStories(projectId);
}

async function loadSystemStories(targetProjectId?: string): Promise<void> {
  isSyncing.value = true;
  syncError.value = null;
  const pid = targetProjectId || activeProjectId.value;
  try {
    const records = await invoke<EnterpriseStoryItem[]>("fetch_stories", {
      projectId: pid,
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
}

async function persistStoryMutation(): Promise<boolean> {
  if (isSyncing.value) return false;
  isSyncing.value = true;
  syncError.value = null;

  const payload: SaveStoryPayload = {
    id: activeId.value,
    projectId: activeProjectId.value,
    title: currentTitle.value.trim() || "Сцена без названия",
    content: currentContent.value,
    category: currentCategory.value.trim() || "Акт I",
    expectedRevision: currentRevision.value,
    operator: activeOperator.value,
  };

  try {
    const updated = await invoke<EnterpriseStoryItem>("mutate_story", { payload });
    activeId.value = updated.id;
    currentTitle.value = updated.title;
    currentContent.value = updated.content;
    currentCategory.value = updated.category;
    currentRevision.value = updated.revision;

    const idx = stories.value.findIndex((s) => s.id === updated.id);
    if (idx !== -1) {
      stories.value[idx] = updated;
    } else {
      stories.value.unshift(updated);
    }
    return true;
  } catch (err) {
    syncError.value = String(err);
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
    return true;
  } catch (err) {
    syncError.value = String(err);
    return false;
  } finally {
    isSyncing.value = false;
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
  await loadSystemProjects();
  await switchProject(imported.id);
  return imported;
}

function initializeBlankDraft(
  category = "Акт I",
  title = "Новая сцена",
  template = "# Новая сцена\n\nТекст сцены..."
): void {
  activeId.value = "";
  currentTitle.value = title;
  currentCategory.value = category;
  currentContent.value = template;
  currentRevision.value = 0;
}

function createEntityDraft(type: "artifact" | "pantheon" | "sanctuary" | "character"): void {
  if (type === "character") {
    initializeBlankDraft(
      "Персонаж",
      "Новый Персонаж",
      "### Досье Персонажа\n* **Архетип:** Протагонист / Искатель\n* **Мотивация:** Поиск истины среди руин забытой цивилизации\n* **Конфликт:** Преданность идеалам против суровых законов мира\n* **Связи:** Союзник ордена\n\nИстория и описание характера..."
    );
  } else if (type === "artifact") {
    initializeBlankDraft(
      "Божественный Артефакт",
      "Око Бесконечности",
      "### Реликвия Высшего Порядка\n* **Класс:** Первородный Артефакт\n* **Владелец:** Павший Демиург Самаэль\n* **Истинное свойство:** Позволяет созерцать параллельные временные ветви ценой увядания воспоминаний\n* **Печать ограничения:** Не может быть активирован смертным без жертвы кровью рода\n* **История обретения:** Извлечен со дна Бездны на исходе Второй Эпохи..."
    );
  } else if (type === "pantheon") {
    initializeBlankDraft(
      "Демиург / Сущность",
      "Элион Хранитель Равновесия",
      "### Досье Божественной Сущности\n* **Истинный ранг:** Архитектор Времени\n* **Культ почитания:** Орден Золотого Веретена\n* **Облик:** Аморфное созвездие из вращающихся серебряных колец\n* **Уязвимость:** Искажение изначального ритма ткани мироздания\n* **Отношение к героям:** Бесстрастное наблюдение за фатальным выбором"
    );
  } else {
    initializeBlankDraft(
      "Святилище Сил",
      "Монолит Астрального Престола",
      "### Локация Высшего Присутствия\n* **Регион:** Расколотые Пики Вечности\n* **Атмосфера:** Гравитационные аномалии, кристаллический пепел\n* **Тайный проход:** Открывается в момент затмения двух солнц"
    );
  }
}

export function useEnterpriseStory() {
  return {
    projects,
    activeProjectId,
    activeProject,
    stories,
    auditLogs,
    activeId,
    currentTitle,
    currentContent,
    currentCategory,
    currentRevision,
    activeOperator,
    isSyncing,
    syncError,
    activeStory,
    currentWords,
    totalSystemWords,
    loadSystemProjects,
    createProject,
    deleteActiveProject,
    switchProject,
    loadSystemStories,
    selectEnterpriseStory,
    persistStoryMutation,
    deleteSystemStory,
    startAuditStream,
    performIndexedSearch,
    exportProjectWstory,
    importProjectWstory,
    initializeBlankDraft,
    createEntityDraft,
  };
}