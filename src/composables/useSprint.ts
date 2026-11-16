// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

import { ref, computed } from "vue";
import { useAppSettings } from "./useAppSettings";
import { useHaptics } from "./useHaptics";

export function useSprint(
  getCurrentWordCount: () => number,
  onSprintComplete?: (wordsWritten: number) => void
) {
  const { settings } = useAppSettings();
  const { vibrateSuccess, vibrateLight } = useHaptics();

  const sprintActive = ref<boolean>(false);
  const sprintStartWords = ref<number>(0);
  const sprintSeconds = ref<number>(settings.value.sprintDurationMinutes * 60);
  let sprintTimerInterval: number | null = null;

  const sprintDelta = computed(() => {
    const current = getCurrentWordCount();
    return current >= sprintStartWords.value ? current - sprintStartWords.value : 0;
  });

  const sprintFormattedTime = computed(() => {
    const m = Math.floor(sprintSeconds.value / 60);
    const s = sprintSeconds.value % 60;
    return `${m}:${s < 10 ? "0" : ""}${s}`;
  });

  function toggleSprint() {
    if (sprintActive.value) {
      sprintActive.value = false;
      if (sprintTimerInterval) clearInterval(sprintTimerInterval);
      vibrateLight();
    } else {
      sprintActive.value = true;
      sprintStartWords.value = getCurrentWordCount();
      sprintSeconds.value = Math.max(60, settings.value.sprintDurationMinutes * 60);
      vibrateLight();
      sprintTimerInterval = window.setInterval(() => {
        if (sprintSeconds.value > 0) {
          sprintSeconds.value--;
        } else {
          sprintActive.value = false;
          if (sprintTimerInterval) clearInterval(sprintTimerInterval);
          vibrateSuccess();
          if (onSprintComplete) {
            onSprintComplete(sprintDelta.value);
          }
        }
      }, 1000);
    }
  }

  function cleanupSprint() {
    if (sprintTimerInterval) clearInterval(sprintTimerInterval);
  }

  return {
    sprintActive,
    sprintDelta,
    sprintFormattedTime,
    toggleSprint,
    cleanupSprint,
  };
}