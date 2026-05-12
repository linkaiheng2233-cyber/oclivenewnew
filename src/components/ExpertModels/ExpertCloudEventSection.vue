<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useExpertModelsStore } from "../../stores/expertModelsStore";
import type { ExpertNode } from "../../utils/tauri-api";

const store = useExpertModelsStore();
const { t } = useI18n();

type CloudNode = Extract<ExpertNode, { type: "cloud_model" }>;
type EventNode = Extract<ExpertNode, { type: "event_trigger" }>;

const cloudNodes = computed(() =>
  (store.draftGraph.nodes ?? []).filter((n): n is CloudNode => n.type === "cloud_model"),
);
const eventNodes = computed(() =>
  (store.draftGraph.nodes ?? []).filter((n): n is EventNode => n.type === "event_trigger"),
);

function patchNode(id: string, patch: Record<string, unknown>): void {
  const g = store.draftGraph;
  store.draftGraph = {
    ...g,
    nodes: (g.nodes ?? []).map((n) =>
      String((n as { id?: string }).id ?? "") === id ? ({ ...(n as object), ...patch } as ExpertNode) : n,
    ),
  };
}

function addCloudModel(): void {
  const g = store.draftGraph;
  const id = `cloud_${Math.random().toString(36).slice(2, 8)}`;
  const n: CloudNode = {
    type: "cloud_model",
    id,
    hostSource: "host",
    model: "",
    enabled: true,
    ui: null,
  };
  store.draftGraph = { ...g, nodes: [...(g.nodes ?? []), n] };
}

function removeCloudModel(id: string): void {
  const g = store.draftGraph;
  store.draftGraph = {
    ...g,
    nodes: (g.nodes ?? []).filter((n) => !(n.type === "cloud_model" && n.id === id)),
    edges: (g.edges ?? []).filter((e) => e.from !== id && e.to !== id),
  };
}

function addEventTrigger(): void {
  const g = store.draftGraph;
  const id = `evt_${Math.random().toString(36).slice(2, 8)}`;
  const n: EventNode = {
    type: "event_trigger",
    id,
    matchSubstring: "",
    memoryContent: "",
    importance: 0.75,
    enabled: true,
    matchScope: "any",
    ui: null,
  };
  store.draftGraph = { ...g, nodes: [...(g.nodes ?? []), n] };
}

function removeEventTrigger(id: string): void {
  const g = store.draftGraph;
  store.draftGraph = {
    ...g,
    nodes: (g.nodes ?? []).filter((n) => !(n.type === "event_trigger" && n.id === id)),
    edges: (g.edges ?? []).filter((e) => e.from !== id && e.to !== id),
  };
}
</script>

<template>
  <div class="ece-grid">
    <div class="ece-card">
      <div class="ece-h">
        {{ t("expertModels.cloudEvent.cloudTitle") }}
        <button type="button" class="ece-add" @click="addCloudModel">
          {{ t("expertModels.cloudEvent.addCloud") }}
        </button>
      </div>
      <p class="ece-hint">{{ t("expertModels.cloudEvent.cloudHint") }}</p>
      <div v-if="!cloudNodes.length" class="ece-empty">{{ t("expertModels.cloudEvent.noCloud") }}</div>
      <div v-for="c in cloudNodes" :key="c.id" class="ece-row">
        <div class="ece-row-head">
          <span class="ece-id">{{ c.id }}</span>
          <button type="button" class="ece-remove" @click="removeCloudModel(c.id)">
            {{ t("expertModels.form.remove") }}
          </button>
        </div>
        <label class="ece-field">
          <span class="ece-label">{{ t("expertModels.cloudEvent.modelIdLabel") }}</span>
          <input
            class="ece-input"
            type="text"
            :value="c.model ?? ''"
            :placeholder="t('expertModels.cloudEvent.modelIdPlaceholder')"
            @input="patchNode(c.id, { model: ($event.target as HTMLInputElement).value })"
          />
        </label>
        <label class="ece-check">
          <input
            type="checkbox"
            :checked="c.enabled"
            @change="patchNode(c.id, { enabled: ($event.target as HTMLInputElement).checked })"
          />
          {{ t("expertModels.cloudEvent.enabled") }}
        </label>
      </div>
    </div>

    <div class="ece-card">
      <div class="ece-h">
        {{ t("expertModels.cloudEvent.eventTitle") }}
        <button type="button" class="ece-add" @click="addEventTrigger">
          {{ t("expertModels.cloudEvent.addEvent") }}
        </button>
      </div>
      <p class="ece-hint">{{ t("expertModels.cloudEvent.eventHint") }}</p>
      <div v-if="!eventNodes.length" class="ece-empty">{{ t("expertModels.cloudEvent.noEvents") }}</div>
      <div v-for="e in eventNodes" :key="e.id" class="ece-row">
        <div class="ece-row-head">
          <span class="ece-id">{{ e.id }}</span>
          <button type="button" class="ece-remove" @click="removeEventTrigger(e.id)">
            {{ t("expertModels.form.remove") }}
          </button>
        </div>
        <label class="ece-field">
          <span class="ece-label">{{ t("expertModels.cloudEvent.matchLabel") }}</span>
          <input
            class="ece-input"
            type="text"
            :value="e.matchSubstring"
            @input="patchNode(e.id, { matchSubstring: ($event.target as HTMLInputElement).value })"
          />
        </label>
        <label class="ece-field">
          <span class="ece-label">{{ t("expertModels.cloudEvent.memoryLabel") }}</span>
          <textarea
            class="ece-text"
            rows="3"
            :value="e.memoryContent"
            @input="patchNode(e.id, { memoryContent: ($event.target as HTMLTextAreaElement).value })"
          />
        </label>
        <label class="ece-field">
          <span class="ece-label">{{ t("expertModels.cloudEvent.importanceLabel") }}</span>
          <input
            class="ece-input ece-num"
            type="number"
            step="0.05"
            min="0"
            max="1"
            :value="e.importance"
            @input="patchNode(e.id, { importance: Number(($event.target as HTMLInputElement).value) })"
          />
        </label>
        <label class="ece-check">
          <input
            type="checkbox"
            :checked="e.enabled"
            @change="patchNode(e.id, { enabled: ($event.target as HTMLInputElement).checked })"
          />
          {{ t("expertModels.cloudEvent.enabled") }}
        </label>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ece-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 12px;
}
.ece-card {
  border: 1px solid var(--border-light);
  border-radius: 10px;
  padding: 10px 12px;
  background: var(--bg-elevated);
}
.ece-h {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  font-weight: 600;
  font-size: 13px;
}
.ece-hint {
  margin: 6px 0 10px;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.ece-add {
  padding: 4px 10px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  cursor: pointer;
  font-size: 12px;
}
.ece-empty {
  font-size: 12px;
  color: var(--text-secondary);
}
.ece-row {
  border-top: 1px solid var(--border-light);
  padding-top: 10px;
  margin-top: 10px;
}
.ece-row-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}
.ece-id {
  font-family: ui-monospace, monospace;
  font-size: 11px;
  color: var(--text-secondary);
}
.ece-remove {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: transparent;
  cursor: pointer;
  color: var(--danger, #c44);
}
.ece-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 8px;
}
.ece-label {
  font-size: 11px;
  color: var(--text-secondary);
}
.ece-input,
.ece-text {
  padding: 6px 8px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 13px;
}
.ece-num {
  max-width: 120px;
}
.ece-check {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-secondary);
}
</style>
