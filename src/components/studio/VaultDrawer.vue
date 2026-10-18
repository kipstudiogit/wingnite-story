<script setup lang="ts">
import { ref } from "vue";
import type { VaultStatus, VaultVerificationResult } from "../../types";

const { vaultStatus, isSealing, isVerifying, verificationResult } = defineProps<{
  vaultStatus: VaultStatus | null;
  isSealing: boolean;
  isVerifying: boolean;
  verificationResult: VaultVerificationResult | null;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "seal", passphrase: string): void;
  (e: "verify"): void;
}>();

const sealPassphrase = ref("");

function handleSeal() {
  if (!sealPassphrase.value.trim()) return;
  emit("seal", sealPassphrase.value);
  sealPassphrase.value = "";
}
</script>

<template>
  <aside class="vault-drawer">
    <header class="drawer-header-strip">
      <h4>Криптографический сейф</h4>
      <button class="btn-drawer-exit" @click="emit('close')">✕</button>
    </header>

    <div class="vault-meta-widget">
      <div class="meta-row">
        <span class="meta-label">Статус:</span>
        <span class="meta-val" :class="{ sealed: vaultStatus?.isSealed }">
          {{ vaultStatus?.isSealed ? 'ЗАПЕЧАТАН' : 'ОТКРЫТ ДЛЯ ПРАВОК' }}
        </span>
      </div>
      <div class="meta-row">
        <span class="meta-label">Сущностей:</span>
        <span class="meta-val">{{ vaultStatus?.totalEntities || 0 }}</span>
      </div>
      <div class="meta-row">
        <span class="meta-label">Сумма ревизий:</span>
        <span class="meta-val">{{ vaultStatus?.totalRevisions || 0 }}</span>
      </div>
      <div class="meta-box">
        <span class="meta-label">Merkle Root:</span>
        <span class="merkle-code">{{ vaultStatus?.merkleRoot || 'отсутствует' }}</span>
      </div>
    </div>

    <div class="vault-actions-cluster">
      <div class="seal-form">
        <input
          v-model="sealPassphrase"
          class="vault-input"
          type="password"
          placeholder="Парольная фраза запечатывания..."
        />
        <button class="btn-vault-seal" :disabled="isSealing" @click="handleSeal">
          {{ isSealing ? 'Запечатывание...' : 'Запечатать сейф' }}
        </button>
      </div>

      <button class="btn-vault-verify" :disabled="isVerifying" @click="emit('verify')">
        {{ isVerifying ? 'Проверка дерева Меркла...' : 'Верифицировать целостность' }}
      </button>

      <div v-if="verificationResult" class="verification-badge" :class="{ valid: verificationResult.isValid }">
        {{ verificationResult.isValid ? 'Целостность подтверждена' : 'Внимание: несоответствие хэшей' }}
      </div>
    </div>
  </aside>
</template>

<style scoped>
.vault-drawer {
  width: 340px;
  background: var(--bg-surface-solid);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  padding: 16px;
  border-radius: 0 16px 16px 0;
  gap: 14px;
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  z-index: 25;
  overflow-y: auto;
}

.drawer-header-strip {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.drawer-header-strip h4 {
  font-size: 13px;
  font-weight: 800;
  color: var(--text-pure);
}

.btn-drawer-exit {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
}

.vault-meta-widget {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.meta-row {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
}

.meta-label {
  color: var(--text-muted);
}

.meta-val.sealed {
  color: var(--accent-emerald);
  font-weight: 700;
}

.meta-box {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.merkle-code {
  display: block;
  font-family: var(--font-mono);
  font-size: 9px;
  word-break: break-all;
  background: rgba(0, 0, 0, 0.4);
  padding: 6px;
  border-radius: 6px;
  color: var(--accent-cyan);
}

.vault-actions-cluster {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.seal-form {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.vault-input {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-pure);
  border-radius: 8px;
  padding: 8px 12px;
  font-size: 12px;
  outline: none;
}

.btn-vault-seal {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  border-radius: 8px;
  padding: 10px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.btn-vault-verify {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-pure);
  border-radius: 8px;
  padding: 10px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.verification-badge {
  text-align: center;
  font-size: 11px;
  font-weight: 700;
  padding: 6px;
  border-radius: 6px;
  background: rgba(244, 63, 94, 0.15);
  color: var(--accent-rose);
}

.verification-badge.valid {
  background: rgba(16, 185, 129, 0.15);
  color: var(--accent-emerald);
}
</style>