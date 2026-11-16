// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { DEFAULT_PROJECT_ID } from "../constants/app";
import type { ProjectItem, SaveProjectPayload } from "../types";
import { useHaptics } from "./useHaptics";
import { useI18n } from "../i18n";

const projects = ref<ProjectItem[]>([]);
const activeProjectId = ref<string>(DEFAULT_PROJECT_ID);
const isProjectSyncing = ref(false);
const projectError = ref<string | null>(null);

export function useProjects() {
  const { vibrateSuccess, vibrateWarning } = useHaptics();
  const { t } = useI18n();

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
        title: title.trim() || t("projects.defaultTitle"),
        description: description.trim() || t("projects.defaultDescription"),
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

  async function updateProject(
    id: string,
    title: string,
    description: string
  ): Promise<ProjectItem | null> {
    isProjectSyncing.value = true;
    projectError.value = null;
    try {
      const payload: SaveProjectPayload = {
        id,
        title: title.trim() || t("projects.defaultTitle"),
        description: description.trim() || t("projects.defaultDescription"),
      };
      const updated = await invoke<ProjectItem>("mutate_project", { payload });
      const idx = projects.value.findIndex((p) => p.id === updated.id);
      if (idx !== -1) {
        projects.value[idx] = updated;
      }
      vibrateSuccess();
      return updated;
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
      projectError.value = t("errors.projectDeleteLast");
      vibrateWarning();
      return false;
    }

    isProjectSyncing.value = true;
    projectError.value = null;
    try {
      await invoke("delete_project", { id });
      projects.value = projects.value.filter((p) => p.id !== id);
      activeProjectId.value = projects.value[0]?.id || DEFAULT_PROJECT_ID;
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
    updateProject,
    deleteActiveProject,
    switchProject,
  };
}