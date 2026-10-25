import { onMounted, onUnmounted } from "vue";

export function useKeyboardShortcuts(actions: {
  onSave: () => void;
  onTogglePalette: () => void;
  onEscape: () => void;
}) {
  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
      e.preventDefault();
      actions.onTogglePalette();
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
      e.preventDefault();
      actions.onSave();
    }
    if (e.key === "Escape") {
      actions.onEscape();
    }
  }

  onMounted(() => {
    window.addEventListener("keydown", handleKeydown);
  });

  onUnmounted(() => {
    window.removeEventListener("keydown", handleKeydown);
  });
}