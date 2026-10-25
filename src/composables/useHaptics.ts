import { useAppSettings } from "./useAppSettings";

export function useHaptics() {
  const { settings } = useAppSettings();
  const isSupported = typeof navigator !== "undefined" && "vibrate" in navigator;

  function trigger(pattern: number | number[]) {
    if (isSupported && settings.value.hapticsEnabled) {
      try {
        navigator.vibrate(pattern);
      } catch {}
    }
  }

  function vibrateLight() {
    trigger(12);
  }

  function vibrateSelection() {
    trigger(20);
  }

  function vibrateSuccess() {
    trigger([16, 40, 24]);
  }

  function vibrateWarning() {
    trigger([35, 30, 35]);
  }

  function vibrateHeavy() {
    trigger([50, 40, 60]);
  }

  return {
    isSupported,
    vibrateLight,
    vibrateSelection,
    vibrateSuccess,
    vibrateWarning,
    vibrateHeavy,
  };
}