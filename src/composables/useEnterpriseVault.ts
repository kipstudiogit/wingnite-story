import { ref, shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface VaultStatus {
  isSealed: boolean;
  merkleRoot: string;
  totalEntities: number;
  totalRevisions: number;
  lastSealedAt: number;
}

export interface VaultSealPayload {
  passphraseSignature: string;
  operator: string;
}

export interface VaultVerificationResult {
  isValid: boolean;
  calculatedRoot: string;
  recordedRoot: string;
  verifiedNodes: number;
}

export function useEnterpriseVault() {
  const vaultStatus = ref<VaultStatus | null>(null);
  const isSealing = ref<boolean>(false);
  const isVerifying = ref<boolean>(false);
  const verificationResult = shallowRef<VaultVerificationResult | null>(null);
  const vaultError = ref<string | null>(null);

  async function loadVaultStatus(): Promise<void> {
    vaultError.value = null;
    try {
      vaultStatus.value = await invoke<VaultStatus>("vault_get_status");
    } catch (err) {
      vaultError.value = String(err);
    }
  }

  async function sealProject(passphraseSignature: string, operator = "Vault-Custodian"): Promise<boolean> {
    isSealing.value = true;
    vaultError.value = null;

    const payload: VaultSealPayload = {
      passphraseSignature,
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

  async function verifyVaultIntegrity(): Promise<boolean> {
    isVerifying.value = true;
    vaultError.value = null;

    try {
      const result = await invoke<VaultVerificationResult>("vault_verify_merkle_tree");
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

  return {
    vaultStatus,
    isSealing,
    isVerifying,
    verificationResult,
    vaultError,
    loadVaultStatus,
    sealProject,
    verifyVaultIntegrity,
  };
}