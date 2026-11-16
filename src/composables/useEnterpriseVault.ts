// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

import { ref, shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  SnapshotItem,
  VaultSealPayload,
  VaultStatus,
  VaultUnlockPayload,
  VaultVerificationResult,
} from "../types";

export function useEnterpriseVault(
  projectIdGetter: () => string,
  storyIdGetter: () => string
) {
  const vaultStatus = ref<VaultStatus | null>(null);
  const snapshots = shallowRef<SnapshotItem[]>([]);
  const isSealing = ref<boolean>(false);
  const isUnlocking = ref<boolean>(false);
  const isVerifying = ref<boolean>(false);
  const isLoadingSnapshots = ref<boolean>(false);
  const verificationResult = shallowRef<VaultVerificationResult | null>(null);
  const vaultError = ref<string | null>(null);

  async function loadVaultStatus(): Promise<void> {
    vaultError.value = null;
    try {
      vaultStatus.value = await invoke<VaultStatus>("vault_get_status", {
        projectId: projectIdGetter(),
      });
    } catch (err) {
      vaultError.value = String(err);
    }
  }

  async function sealProject(
    passphrase: string,
    lockEditing: boolean,
    operator = "Vault-Custodian"
  ): Promise<boolean> {
    isSealing.value = true;
    vaultError.value = null;
    verificationResult.value = null;

    const payload: VaultSealPayload = {
      projectId: projectIdGetter(),
      passphrase,
      lockEditing,
      operator,
    };

    try {
      const updated = await invoke<VaultStatus>("vault_seal_project", { payload });
      vaultStatus.value = updated;
      return true;
    } catch (err) {
      vaultError.value = String(err);
      return false;
    } finally {
      isSealing.value = false;
    }
  }

  async function unlockProject(
    passphrase: string,
    operator = "Vault-Custodian"
  ): Promise<boolean> {
    isUnlocking.value = true;
    vaultError.value = null;

    const payload: VaultUnlockPayload = {
      projectId: projectIdGetter(),
      passphrase,
      operator,
    };

    try {
      const updated = await invoke<VaultStatus>("vault_unlock_project", { payload });
      vaultStatus.value = updated;
      return true;
    } catch (err) {
      vaultError.value = String(err);
      return false;
    } finally {
      isUnlocking.value = false;
    }
  }

  async function verifyVaultIntegrity(): Promise<boolean> {
    isVerifying.value = true;
    vaultError.value = null;

    try {
      const result = await invoke<VaultVerificationResult>("vault_verify_merkle_tree", {
        projectId: projectIdGetter(),
      });
      verificationResult.value = result;
      return result.isValid;
    } catch (err) {
      vaultError.value = String(err);
      verificationResult.value = null;
      return false;
    } finally {
      isVerifying.value = false;
    }
  }

  async function loadSnapshots(explicitStoryId?: string): Promise<void> {
    const sid = explicitStoryId ?? storyIdGetter();
    if (!sid) {
      snapshots.value = [];
      return;
    }
    isLoadingSnapshots.value = true;
    try {
      snapshots.value = await invoke<SnapshotItem[]>("fetch_story_snapshots", {
        storyId: sid,
      });
    } catch {
      snapshots.value = [];
    } finally {
      isLoadingSnapshots.value = false;
    }
  }

  async function createSnapshot(
    storyId: string,
    title: string,
    content: string
  ): Promise<SnapshotItem | null> {
    if (!storyId) return null;
    vaultError.value = null;
    try {
      const item = await invoke<SnapshotItem>("create_story_snapshot", {
        storyId,
        projectId: projectIdGetter(),
        title,
        content,
      });
      snapshots.value = [item, ...snapshots.value];
      return item;
    } catch (err) {
      vaultError.value = String(err);
      return null;
    }
  }

  async function deleteSnapshot(id: string): Promise<void> {
    try {
      await invoke("delete_story_snapshot", { id });
      snapshots.value = snapshots.value.filter((s) => s.id !== id);
    } catch (err) {
      vaultError.value = String(err);
    }
  }

  return {
    vaultStatus,
    snapshots,
    isSealing,
    isUnlocking,
    isVerifying,
    isLoadingSnapshots,
    verificationResult,
    vaultError,
    loadVaultStatus,
    sealProject,
    unlockProject,
    verifyVaultIntegrity,
    loadSnapshots,
    createSnapshot,
    deleteSnapshot,
  };
}