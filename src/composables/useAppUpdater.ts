import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { UpdateStatusPayload } from "../types";

export function useAppUpdater() {
  const isChecking = ref(false);
  const isInstalling = ref(false);
  const updateAvailable = ref(false);
  const updateInfo = ref<UpdateStatusPayload | null>(null);
  const updateError = ref<string | null>(null);

  async function checkForUpdates(silent = false): Promise<boolean> {
    isChecking.value = true;
    updateError.value = null;

    try {
      const result = await invoke<UpdateStatusPayload>("check_application_update");
      updateInfo.value = result;
      updateAvailable.value = result.available;
      return result.available;
    } catch (err) {
      if (!silent) {
        updateError.value = String(err);
      }
      updateAvailable.value = false;
      return false;
    } finally {
      isChecking.value = false;
    }
  }

  async function installAndRestart(): Promise<boolean> {
    isInstalling.value = true;
    updateError.value = null;

    try {
      await invoke("install_application_update");
      return true;
    } catch (err) {
      updateError.value = String(err);
      return false;
    } finally {
      isInstalling.value = false;
    }
  }

  function dismissUpdate() {
    updateAvailable.value = false;
  }

  return {
    isChecking,
    isInstalling,
    updateAvailable,
    updateInfo,
    updateError,
    checkForUpdates,
    installAndRestart,
    dismissUpdate,
  };
}