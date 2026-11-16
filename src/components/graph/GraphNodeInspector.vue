<!-- Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio) -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->
<script setup lang="ts">
import type { GraphNodePacked } from "../../types";
import { useI18n } from "../../i18n";

export interface InspectorRelationView {
  id: string;
  peerTitle: string;
  relationType: string;
  description: string;
}

const { node, nodeRelations } = defineProps<{
  node: GraphNodePacked;
  nodeRelations: InspectorRelationView[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "openStory", node: GraphNodePacked): void;
  (e: "openLinkModal"): void;
  (e: "unlink", relationId: string): void;
}>();

const { t } = useI18n();
</script>

<template>
  <aside class="node-inspector-drawer">
    <div class="inspector-header">
      <div class="inspector-titles">
        <span class="inspector-badge">{{ node.category }}</span>
        <h3>{{ node.label }}</h3>
      </div>
      <button class="btn-close-inspector" @click="emit('close')">✕</button>
    </div>

    <div class="inspector-stats">
      <span>{{ t("graph.connectionsCount") }} <b>{{ node.degree }}</b></span>
      <span>{{ t("graph.nodeMass") }} <b>{{ node.mass.toFixed(1) }}</b></span>
    </div>

    <div v-if="nodeRelations.length > 0" class="inspector-links-section">
      <div class="links-heading">{{ t("graph.relationsListTitle") }}</div>
      <div class="links-scroll">
        <div v-for="rel in nodeRelations" :key="rel.id" class="inspector-link-row">
          <div class="link-meta">
            <span class="link-type">{{ rel.relationType }}</span>
            <b class="link-peer">{{ rel.peerTitle }}</b>
          </div>
          <button
            class="btn-unlink-node"
            :title="t('graph.unlinkBtn')"
            @click="emit('unlink', rel.id)"
          >
            ✕
          </button>
        </div>
      </div>
    </div>

    <div class="inspector-actions">
      <button class="btn-open-story" @click="emit('openStory', node)">
        {{ t("graph.openSceneBtn") }}
      </button>
      <button class="btn-add-rel" @click="emit('openLinkModal')">
        {{ t("graph.linkNodeBtn") }}
      </button>
    </div>
  </aside>
</template>

<style scoped>
.node-inspector-drawer {
  position: absolute;
  top: 18px;
  left: 18px;
  width: 320px;
  max-height: calc(100% - 36px);
  overflow-y: auto;
  background: var(--bg-surface-solid);
  border: 1px solid var(--border-strong);
  border-radius: 16px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  z-index: 15;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.65);
}

.inspector-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 8px;
}

.inspector-titles {
  min-width: 0;
}

.inspector-badge {
  font-size: 10px;
  font-weight: 800;
  color: var(--accent-cyan);
  text-transform: uppercase;
}

.inspector-header h3 {
  font-size: 15px;
  font-weight: 800;
  color: var(--text-pure);
  margin-top: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.btn-close-inspector {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  width: 30px;
  height: 30px;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 14px;
  flex-shrink: 0;
}

.inspector-stats {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--font-mono);
  background: var(--bg-card);
  padding: 8px 10px;
  border-radius: 8px;
}

.inspector-stats b {
  color: var(--accent-cyan);
}

.inspector-links-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.links-heading {
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--text-dim);
}

.links-scroll {
  max-height: 140px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.inspector-link-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 6px 8px;
  gap: 6px;
}

.link-meta {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.link-type {
  font-size: 9px;
  font-weight: 700;
  color: var(--accent-cyan);
}

.link-peer {
  font-size: 11px;
  color: var(--text-pure);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.btn-unlink-node {
  background: transparent;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  padding: 2px 6px;
}

.btn-unlink-node:hover {
  color: var(--accent-rose);
}

.inspector-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.btn-open-story {
  background: linear-gradient(135deg, var(--accent-indigo) 0%, #4338ca 100%);
  border: none;
  color: #fff;
  border-radius: 8px;
  min-height: 38px;
  padding: 8px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.btn-add-rel {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-pure);
  border-radius: 8px;
  min-height: 38px;
  padding: 8px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

@media (max-width: 900px) {
  .node-inspector-drawer {
    top: auto;
    bottom: max(10px, var(--safe-bottom));
    left: max(10px, var(--safe-left));
    right: max(10px, var(--safe-right));
    width: auto;
    max-height: 46dvh;
  }

  .inspector-actions {
    flex-direction: row;
  }

  .inspector-actions button {
    flex: 1;
  }
}
</style>