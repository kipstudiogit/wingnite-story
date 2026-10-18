import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { EnterpriseStoryItem, SnapshotItem } from "../types";

export function useStories() {
  const stories = ref<EnterpriseStoryItem[]>([]);
  const snapshots = ref<SnapshotItem[]>([]);
  const activeId = ref<string>("");
  const currentTitle = ref<string>("Глава 1: Тени Цитадели");
  const currentCategory = ref<string>("Акт I");
  const currentContent = ref<string>(
    "# Глава 1: Тени Цитадели\n\nСеверный ветер трепал штандарты над каменными бастионами цитадели, когда караван беглецов перевалил через заснеженный перевал...\n\n---\n\n> «История никогда не начинается в тишине. Ей всегда предшествует раскат грома.»\n\nОн коснулся рукояти старинного клинка, чувствуя холод древней стали."
  );
  const statusMsg = ref<string>("");

  const activeStory = computed(() => stories.value.find((s) => s.id === activeId.value));

  function calculateWordCount(text: string): number {
    let count = 0;
    let inWord = false;
    for (let i = 0; i < text.length; i++) {
      const charCode = text.charCodeAt(i);
      if (charCode > 32) {
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

  const wordCount = computed(() => calculateWordCount(currentContent.value));

  const totalBookWords = computed(() => {
    return stories.value.reduce((acc, s) => acc + calculateWordCount(s.content), 0);
  });

  const actsList = computed(() => {
    const categories = new Set<string>();
    for (const story of stories.value) {
      categories.add(story.category);
    }
    return Array.from(categories);
  });

  async function loadStories() {
    try {
      const list = await invoke<EnterpriseStoryItem[]>("fetch_stories");
      stories.value = list;
      if (list.length > 0 && !activeId.value) {
        selectStory(list[0]);
      }
    } catch (err) {
      statusMsg.value = String(err);
    }
  }

  async function loadSnapshots(storyId: string) {
    if (!storyId) {
      snapshots.value = [];
      return;
    }
    try {
      snapshots.value = await invoke<SnapshotItem[]>("get_snapshots", { storyId });
    } catch (err) {
      statusMsg.value = String(err);
    }
  }

  function selectStory(story: EnterpriseStoryItem) {
    activeId.value = story.id;
    currentTitle.value = story.title;
    currentContent.value = story.content;
    currentCategory.value = story.category;
    loadSnapshots(story.id);
  }

  async function saveCurrentStory() {
    try {
      const payload: EnterpriseStoryItem = {
        id: activeId.value,
        title: currentTitle.value,
        content: currentContent.value,
        category: currentCategory.value,
        revision: activeStory.value?.revision ?? 1,
        contentHash: activeStory.value?.contentHash ?? "",
        updatedAt: 0,
      };
      const saved = await invoke<EnterpriseStoryItem>("save_story", { story: payload });
      activeId.value = saved.id;
      statusMsg.value = "Синхронизировано";
      await loadStories();
    } catch (err) {
      statusMsg.value = String(err);
    }
  }

  async function deleteCurrentStory() {
    if (!activeId.value) return;
    try {
      await invoke("delete_story", { id: activeId.value, operator: "Legacy-Operator" });
      activeId.value = "";
      currentTitle.value = "";
      currentContent.value = "";
      currentCategory.value = "";
      snapshots.value = [];
      await loadStories();
    } catch (err) {
      statusMsg.value = String(err);
    }
  }

  async function makeSnapshot() {
    if (!activeId.value) return;
    try {
      await invoke<SnapshotItem>("create_snapshot", {
        storyId: activeId.value,
        title: currentTitle.value,
        content: currentContent.value,
      });
      statusMsg.value = "Снимок сохранен";
      await loadSnapshots(activeId.value);
    } catch (err) {
      statusMsg.value = String(err);
    }
  }

  function restoreSnapshot(snap: SnapshotItem) {
    currentTitle.value = snap.title;
    currentContent.value = snap.content;
    statusMsg.value = "Снимок восстановлен";
  }

  function createNewStory(category = "Акт I", title = "Новая сцена", template = "# Новая сцена\n\n") {
    activeId.value = "";
    currentTitle.value = title;
    currentCategory.value = category;
    currentContent.value = template;
    snapshots.value = [];
  }

  function createDivineEntity(type: "artifact" | "pantheon" | "sanctuary") {
    if (type === "artifact") {
      createNewStory(
        "Божественный Артефакт",
        "Око Бесконечности",
        "### Реликвия Высшего Порядка\n* **Класс:** Первородный Артефакт\n* **Владелец:** Павший Демиург Самаэль\n* **Истинное свойство:** Позволяет созерцать параллельные временные ветви ценой увядания воспоминаний\n* **Печать ограничения:** Не может быть активирован смертным без жертвы кровью рода\n* **История обретения:** Извлечен со дна Бездны на исходе Второй Эпохи..."
      );
    } else if (type === "pantheon") {
      createNewStory(
        "Демиург / Сущность",
        "Элион Хранитель Равновесия",
        "### Досье Божественной Сущности\n* **Истинный ранг:** Архитектор Времени\n* **Культ почитания:** Орден Золотого Веретена\n* **Облик:** Аморфное созвездие из вращающихся серебряных колец\n* **Уязвимость:** Искажение изначального ритма ткани мироздания\n* **Отношение к героям:** Бесстрастное наблюдение за фатальным выбором"
      );
    } else {
      createNewStory(
        "Святилище Сил",
        "Монолит Астрального Престола",
        "### Локация Высшего Присутствия\n* **Регион:** Расколотые Пики Вечности\n* **Атмосфера:** Гравитационные аномалии, кристаллический пепел\n* **Тайный проход:** Открывается в момент затмения двух солнц"
      );
    }
  }

  async function exportFullManuscript() {
    try {
      const manuscript = await invoke<string>("export_manuscript");
      const blob = new Blob([manuscript], { type: "text/markdown;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "Wingnite_Story_Complete.md";
      a.click();
      URL.revokeObjectURL(url);
    } catch (err) {
      statusMsg.value = String(err);
    }
  }

  return {
    stories,
    snapshots,
    activeId,
    currentTitle,
    currentCategory,
    currentContent,
    statusMsg,
    activeStory,
    wordCount,
    totalBookWords,
    actsList,
    loadStories,
    selectStory,
    saveCurrentStory,
    deleteCurrentStory,
    makeSnapshot,
    restoreSnapshot,
    createNewStory,
    createDivineEntity,
    exportFullManuscript,
    calculateWordCount,
  };
}