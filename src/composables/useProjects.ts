import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ProjectItem, SaveProjectPayload } from "../types";
import { useHaptics } from "./useHaptics";

const projects = ref<ProjectItem[]>([]);
const activeProjectId = ref<string>("default_project");
const isProjectSyncing = ref(false);
const projectError = ref<string | null>(null);

export function useProjects() {
  const { vibrateSuccess, vibrateWarning } = useHaptics();

  const activeProject = computed(() =>
    projects.value.find((p) => p.id === activeProjectId.value)
  );

  async function loadSystemProjects(): Promise<void> {
    try {
      const list = await invoke<ProjectItem[]>("fetch_projects");
      projects.value = list;
      if (list.length > 0 && !projects.value.some((p) => p.id === activeProjectId.value)) {
        activeProjectId.value = list[0].id;
      }
    } catch (err) {
      projectError.value = String(err);
    }
  }

  async function createProject(title: string, description: string): Promise<ProjectItem | null> {
    isProjectSyncing.value = true;
    projectError.value = null;
    try {
      const payload: SaveProjectPayload = {
        title: title.trim() || "Новое произведение",
        description: description.trim() || "Архивная книга",
      };
      const saved = await invoke<ProjectItem>("mutate_project", { payload });
      projects.value.unshift(saved);
      activeProjectId.value = saved.id;
      vibrateSuccess();
      return saved;
    } catch (err) {
      projectError.value = String(err);
      vibrateWarning();
      return null;
    } finally {
      isProjectSyncing.value = false;
    }
  }

  async function deleteActiveProject(id: string): Promise<boolean> {
    if (projects.value.length <= 1) {
      projectError.value = "Нельзя удалить единственное оставшееся произведение.";
      vibrateWarning();
      return false;
    }

    isProjectSyncing.value = true;
    projectError.value = null;
    try {
      await invoke("delete_project", { id });
      projects.value = projects.value.filter((p) => p.id !== id);
      activeProjectId.value = projects.value[0]?.id || "default_project";
      vibrateSuccess();
      return true;
    } catch (err) {
      projectError.value = String(err);
      return false;
    } finally {
      isProjectSyncing.value = false;
    }
  }

  function switchProject(id: string) {
    if (activeProjectId.value !== id) {
      activeProjectId.value = id;
    }
  }

  return {
    projects,
    activeProjectId,
    activeProject,
    isProjectSyncing,
    projectError,
    loadSystemProjects,
    createProject,
    deleteActiveProject,
    switchProject,
  };
}