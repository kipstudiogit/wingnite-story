<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import { ref } from "vue";
import type { SnapshotItem, VaultStatus, VaultVerificationResult } from "../../types";
import { useI18n } from "../../i18n";

const {
  vaultStatus,
  snapshots,
  isSealing,
  isUnlocking,
  isVerifying,
  isLoadingSnapshots,
  verificationResult,
  vaultError,
  hasActiveStory,
} = defineProps<{
  vaultStatus: VaultStatus | null;
  snapshots: SnapshotItem[];
  isSealing: boolean;
  isUnlocking: boolean;
  isVerifying: boolean;
  isLoadingSnapshots: boolean;
  verificationResult: VaultVerificationResult | null;
  vaultError: string | null;
  hasActiveStory: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "seal", passphrase: string, lockEditing: boolean): void;
  (e: "unlock", passphrase: string): void;
  (e: "verify"): void;
  (e: "createSnapshot"): void;
  (e: "restoreSnapshot", snapshot: SnapshotItem): void;
  (e: "deleteSnapshot", id: string): void;
}>();

const { t, locale } = useI18n();

const activeTab = ref<"security" | "snapshots">("security");
const sealPassphrase = ref("");
const lockEditing = ref(true);
const unlockPassphrase = ref("");

function handleSeal() {
  if (lockEditing.value && !sealPassphrase.value.trim()) return;
  emit("seal", sealPassphrase.value, lockEditing.value);
  sealPassphrase.value = "";
}

function handleUnlock() {
  if (!unlockPassphrase.value.trim()) return;
  emit("unlock", unlockPassphrase.value);
  unlockPassphrase.value = "";
}

function formatDate(unixSec: number): string {
  return new Intl.DateTimeFormat(locale.value, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(unixSec * 1000));
}
</script>

<template>
  <aside class="vault-drawer">
    <header class="drawer-header-strip">
      <h4>{{ t("vault.title") }}</h4>
      <button class="btn-drawer-exit" @click="emit('close')">✕</button>
    </header>

    <div class="vault-tabs-bar">
      <button
        :class="['vault-tab-btn', { active: activeTab === 'security' }]"
        @click="activeTab = 'security'"
      >
        {{ t("vault.tabSecurity") }}
      </button>
      <button
        :class="['vault-tab-btn', { active: activeTab === 'snapshots' }]"
        @click="activeTab = 'snapshots'"
      >
        {{ t("vault.tabSnapshots", { count: snapshots.length }) }}
      </button>
    </div>

    <div v-if="activeTab === 'security'" class="vault-tab-body">
      <div class="vault-meta-widget">
        <div class="meta-row">
          <span class="meta-label">{{ t("vault.statusLabel") }}</span>
          <span
            class="meta-val"
            :class="{
              locked: vaultStatus?.isLocked,
              sealed: vaultStatus?.isSealed && !vaultStatus?.isLocked,
            }"
          >
            {{
              vaultStatus?.isLocked
                ? t("vault.locked")
                : vaultStatus?.isSealed
                ? t("vault.sealed")
                : t("vault.open")
            }}
          </span>
        </div>
        <div class="meta-row">
          <span class="meta-label">{{ t("vault.entitiesCount") }}</span>
          <span class="meta-val">{{ vaultStatus?.totalEntities || 0 }}</span>
        </div>
        <div class="meta-row">
          <span class="meta-label">{{ t("vault.revisionsSum") }}</span>
          <span class="meta-val">{{ vaultStatus?.totalRevisions || 0 }}</span>
        </div>
        <div class="meta-box">
          <span class="meta-label">{{ t("vault.merkleRootLabel") }}</span>
          <span class="merkle-code">{{ vaultStatus?.merkleRoot || t("vault.missing") }}</span>
        </div>
      </div>

      <div v-if="vaultError" class="vault-error-banner">
        {{ vaultError }}
      </div>

      <div class="vault-actions-cluster">
        <div v-if="vaultStatus?.isLocked" class="seal-form">
          <input
            v-model="unlockPassphrase"
            class="vault-input"
            type="password"
            :placeholder="t('vault.unlockPassphrasePlaceholder')"
            @keydown.enter="handleUnlock"
          />
          <button class="btn-vault-unlock" :disabled="isUnlocking" @click="handleUnlock">
            {{ isUnlocking ? t("vault.unlocking") : t("vault.unlockBtn") }}
          </button>
        </div>

        <div v-else class="seal-form">
          <input
            v-model="sealPassphrase"
            class="vault-input"
            type="password"
            :placeholder="t('vault.passphrasePlaceholder')"
            @keydown.enter="handleSeal"
          />
          <label class="lock-checkbox-row">
            <input v-model="lockEditing" type="checkbox" />
            <span>{{ t("vault.lockEditingLabel") }}</span>
          </label>
          <button
            class="btn-vault-seal"
            :disabled="isSealing || (lockEditing && !sealPassphrase.trim())"
            @click="handleSeal"
          >
            {{ isSealing ? t("vault.sealing") : t("vault.sealBtn") }}
          </button>
        </div>

        <button class="btn-vault-verify" :disabled="isVerifying" @click="emit('verify')">
          {{ isVerifying ? t("vault.verifying") : t("vault.verifyBtn") }}
        </button>

        <div v-if="verificationResult" class="verification-badge" :class="{ valid: verificationResult.isValid }">
          {{ verificationResult.isValid ? t("vault.validResult") : t("vault.invalidResult") }}
        </div>
      </div>
    </div>

    <div v-else class="vault-tab-body">
      <button
        class="btn-create-snapshot"
        :disabled="!hasActiveStory || vaultStatus?.isLocked"
        @click="emit('createSnapshot')"
      >
        {{ t("vault.createSnapshotBtn") }}
      </button>

      <div v-if="isLoadingSnapshots" class="snapshots-empty">
        {{ t("common.loading") }}
      </div>

      <div v-else-if="snapshots.length === 0" class="snapshots-empty">
        {{ t("vault.emptySnapshots") }}
      </div>

      <div v-else class="snapshots-list">
        <div v-for="snap in snapshots" :key="snap.id" class="snapshot-card">
          <div class="snap-top">
            <span class="snap-date">{{ formatDate(snap.createdAt) }}</span>
            <span class="snap-words">{{ t("vault.snapshotWords", { count: snap.wordCount }) }}</span>
          </div>
          <div class="snap-title">{{ snap.title || t("common.untitled") }}</div>
          <div class="snap-preview">{{ snap.content.slice(0, 90) }}</div>
          <div class="snap-actions">
            <button
              class="btn-snap-restore"
              :disabled="vaultStatus?.isLocked"
              @click="emit('restoreSnapshot', snap)"
            >
              {{ t("vault.restoreSnapshotBtn") }}
            </button>
            <button
              class="btn-snap-delete"
              :title="t('vault.deleteSnapshotTooltip')"
              @click="emit('deleteSnapshot', snap.id)"
            >
              ✕
            </button>
          </div>
        </div>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.vault-drawer {
  width: min(100%, 360px);
  background: var(--bg-surface-solid);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  padding: 16px;
  border-radius: 0 16px 16px 0;
  gap: 12px;
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  z-index: 35;
  overflow-y: auto;
  overscroll-behavior-y: contain;
  box-shadow: -10px 0 35px rgba(0, 0, 0, 0.45);
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
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  width: 30px;
  height: 30px;
  color: var(--text-muted);
  cursor: pointer;
}

.vault-tabs-bar {
  display: flex;
  gap: 4px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  padding: 3px;
  border-radius: 10px;
}

.vault-tab-btn {
  flex: 1;
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 700;
  padding: 7px 8px;
  border-radius: 7px;
  cursor: pointer;
}

.vault-tab-btn.active {
  background: var(--accent-indigo);
  color: #fff;
}

.vault-tab-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
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

.meta-val.locked {
  color: var(--accent-rose);
  font-weight: 800;
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
  background: var(--bg-input);
  border: 1px solid var(--border-subtle);
  padding: 6px;
  border-radius: 6px;
  color: var(--accent-cyan);
}

.vault-error-banner {
  background: rgba(244, 63, 94, 0.15);
  border: 1px solid var(--accent-rose);
  color: var(--accent-rose);
  padding: 8px 10px;
  border-radius: 8px;
  font-size: 11px;
  font-weight: 700;
}

.vault-actions-cluster {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.seal-form {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.vault-input {
  background: var(--bg-input);
  border: 1px solid var(--border);
  color: var(--text-pure);
  border-radius: 8px;
  min-height: 40px;
  padding: 8px 12px;
  font-size: 12px;
  outline: none;
}

.lock-checkbox-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  color: var(--text-muted);
  cursor: pointer;
}

.lock-checkbox-row input {
  accent-color: var(--accent-indigo);
}

.btn-vault-seal {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  border-radius: 8px;
  min-height: 40px;
  padding: 10px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.btn-vault-seal:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.btn-vault-unlock {
  background: var(--accent-emerald);
  border: none;
  color: #fff;
  border-radius: 8px;
  min-height: 40px;
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
  min-height: 40px;
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

.btn-create-snapshot {
  background: var(--accent-indigo);
  border: none;
  color: #fff;
  border-radius: 8px;
  min-height: 40px;
  padding: 10px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.btn-create-snapshot:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.snapshots-empty {
  text-align: center;
  font-size: 11px;
  color: var(--text-dim);
  padding: 24px 10px;
}

.snapshots-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.snapshot-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.snap-top {
  display: flex;
  justify-content: space-between;
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--accent-cyan);
}

.snap-title {
  font-size: 12px;
  font-weight: 800;
  color: var(--text-pure);
}

.snap-preview {
  font-size: 11px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.snap-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 4px;
}

.btn-snap-restore {
  background: rgba(99, 102, 241, 0.18);
  border: 1px solid var(--accent-indigo);
  color: var(--text-pure);
  font-size: 11px;
  font-weight: 700;
  padding: 5px 10px;
  border-radius: 6px;
  cursor: pointer;
}

.btn-snap-restore:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-snap-delete {
  background: transparent;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
}

@media (max-width: 900px) {
  .vault-drawer {
    width: 100%;
    border-left: none;
    border-radius: 14px;
  }
}
</style>