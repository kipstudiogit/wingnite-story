import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

export function useDevice() {
  const windowWidth = ref(window.innerWidth);
  const windowHeight = ref(window.innerHeight);
  const viewportHeight = ref(window.visualViewport ? window.visualViewport.height : window.innerHeight);
  const isTouchDevice = ref(false);
  const isKeyboardOpen = ref(false);
  const keyboardHeight = ref(0);

  function handleResize() {
    windowWidth.value = window.innerWidth;
    windowHeight.value = window.innerHeight;
  }

  function handleViewportChange() {
    if (window.visualViewport) {
      const vh = window.visualViewport.height;
      viewportHeight.value = vh;
      const heightDifference = window.innerHeight - vh;
      const opened = heightDifference > 140;
      isKeyboardOpen.value = opened;
      keyboardHeight.value = opened ? heightDifference : 0;
    }
  }

  function scrollToCursor(textarea: HTMLTextAreaElement | null) {
    if (!textarea) return;
    requestAnimationFrame(() => {
      const cursorPosition = textarea.selectionStart;
      const textBeforeCursor = textarea.value.substring(0, cursorPosition);
      const lines = textBeforeCursor.split("\n");
      const currentLineIndex = lines.length;
      const approxLineHeight = 28;
      const targetScroll = Math.max(0, currentLineIndex * approxLineHeight - 120);

      textarea.scrollTo({
        top: targetScroll,
        behavior: "smooth",
      });
    });
  }

  function syncAndroidThemeColor(hexColor: string) {
    let meta = document.querySelector('meta[name="theme-color"]') as HTMLMetaElement | null;
    if (!meta) {
      meta = document.createElement("meta");
      meta.name = "theme-color";
      document.head.appendChild(meta);
    }
    meta.setAttribute("content", hexColor);
  }

  async function shareOrExportFile(
    filename: string,
    content: string,
    mimeType = "text/plain"
  ): Promise<string> {
    const isMobileEnv = isMobile.value;

    if (isMobileEnv && typeof navigator.share === "function") {
      try {
        const file = new File([content], filename, { type: mimeType });
        if (navigator.canShare && navigator.canShare({ files: [file] })) {
          await navigator.share({
            title: filename,
            files: [file],
          });
          return `Файл "${filename}" успешно передан в систему`;
        }
      } catch (err) {
        if ((err as Error).name === "AbortError") {
          return "Экспорт отменен пользователем";
        }
      }
    }

    try {
      const savedPath = await invoke<string>("save_export_file", {
        filename,
        content,
      });
      return `Сохранено на устройстве: ${savedPath}`;
    } catch {
      const blob = new Blob([content], { type: mimeType });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = filename;
      a.click();
      URL.revokeObjectURL(url);
      return `Файл "${filename}" скачан`;
    }
  }

  onMounted(() => {
    isTouchDevice.value =
      "ontouchstart" in window ||
      navigator.maxTouchPoints > 0 ||
      window.matchMedia("(pointer: coarse)").matches;

    window.addEventListener("resize", handleResize);

    if (window.visualViewport) {
      window.visualViewport.addEventListener("resize", handleViewportChange);
      window.visualViewport.addEventListener("scroll", handleViewportChange);
    }
  });

  onUnmounted(() => {
    window.removeEventListener("resize", handleResize);
    if (window.visualViewport) {
      window.visualViewport.removeEventListener("resize", handleViewportChange);
      window.visualViewport.removeEventListener("scroll", handleViewportChange);
    }
  });

  const isMobile = computed(() => {
    return windowWidth.value <= 820 || (isTouchDevice.value && windowWidth.value <= 1024);
  });

  return {
    windowWidth,
    windowHeight,
    viewportHeight,
    isTouchDevice,
    isKeyboardOpen,
    keyboardHeight,
    isMobile,
    scrollToCursor,
    syncAndroidThemeColor,
    shareOrExportFile,
  };
}