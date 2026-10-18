import { ref, computed, onMounted, onUnmounted } from "vue";

export function useDevice() {
  const windowWidth = ref(window.innerWidth);
  const windowHeight = ref(window.innerHeight);
  const isTouchDevice = ref(false);
  const isKeyboardOpen = ref(false);

  function handleResize() {
    windowWidth.value = window.innerWidth;
    windowHeight.value = window.innerHeight;
  }

  function handleViewportChange() {
    if (window.visualViewport) {
      const heightDifference = window.innerHeight - window.visualViewport.height;
      isKeyboardOpen.value = heightDifference > 140;
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
    }
  });

  onUnmounted(() => {
    window.removeEventListener("resize", handleResize);
    if (window.visualViewport) {
      window.visualViewport.removeEventListener("resize", handleViewportChange);
    }
  });

  const isMobile = computed(() => {
    return windowWidth.value <= 820 || (isTouchDevice.value && windowWidth.value <= 1024);
  });

  return {
    windowWidth,
    windowHeight,
    isTouchDevice,
    isKeyboardOpen,
    isMobile,
  };
}