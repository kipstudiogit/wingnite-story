// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

import { onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

export function useLifecycle(onSuspendCallback?: () => Promise<void> | void) {
  async function handleVisibilityChange() {
    if (document.visibilityState === "hidden") {
      if (onSuspendCallback) {
        await onSuspendCallback();
      }
      try {
        await invoke("app_checkpoint_wal");
      } catch {
      }
    }
  }

  async function handlePageHide() {
    if (onSuspendCallback) {
      await onSuspendCallback();
    }
    try {
      await invoke("app_checkpoint_wal");
    } catch {
    }
  }

  onMounted(() => {
    document.addEventListener("visibilitychange", handleVisibilityChange);
    window.addEventListener("pagehide", handlePageHide);
  });

  onUnmounted(() => {
    document.removeEventListener("visibilitychange", handleVisibilityChange);
    window.removeEventListener("pagehide", handlePageHide);
  });
}