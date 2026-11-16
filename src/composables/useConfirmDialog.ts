// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

import { ref } from "vue";
import type { ConfirmDialogOptions } from "../types";

interface ActiveConfirmState extends ConfirmDialogOptions {
  isOpen: boolean;
}

const state = ref<ActiveConfirmState>({
  isOpen: false,
  title: "",
  message: "",
  confirmLabel: undefined,
  cancelLabel: undefined,
  danger: false,
});

let pendingResolver: ((value: boolean) => void) | null = null;

export function useConfirmDialog() {
  function requestConfirm(options: ConfirmDialogOptions): Promise<boolean> {
    if (pendingResolver) {
      pendingResolver(false);
      pendingResolver = null;
    }

    state.value = {
      isOpen: true,
      title: options.title,
      message: options.message,
      confirmLabel: options.confirmLabel,
      cancelLabel: options.cancelLabel,
      danger: options.danger ?? false,
    };

    return new Promise<boolean>((resolve) => {
      pendingResolver = resolve;
    });
  }

  function resolveConfirm(accepted: boolean) {
    state.value.isOpen = false;
    if (pendingResolver) {
      const resolver = pendingResolver;
      pendingResolver = null;
      resolver(accepted);
    }
  }

  return {
    confirmState: state,
    requestConfirm,
    resolveConfirm,
  };
}