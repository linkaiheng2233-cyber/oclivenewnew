<script setup lang="ts">
import { Handle, Position } from "@vue-flow/core";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { ExpertNode } from "../../utils/tauri-api";

const props = defineProps<{
  id: string;
  data: {
    expert: ExpertNode;
    onPatch: (patch: Record<string, unknown>) => void;
    error?: boolean;
  };
  selected?: boolean;
}>();

const { t } = useI18n();

const n = computed(() => props.data.expert);

const metaLine = computed(() => {
  const x = n.value;
  if (x.type === "base_model") {
    const tail = (x.ggufPath || "").split(/[/\\]/).pop() || "—";
    return tail;
  }
  if (x.type === "lora_adapter") {
    const tail = (x.ggufPath || "").split(/[/\\]/).pop() || "—";
    return `${tail} · σ=${Number(x.strength).toFixed(2)}`;
  }
  if (x.type === "cloud_model") {
    const mid = (x.model ?? "").trim() || String(t("expertModels.canvas.nodeMeta.cloudHostDefault"));
    return mid;
  }
  if (x.type === "event_trigger") {
    const kw = (x.matchSubstring ?? "").trim() || "—";
    return String(t("expertModels.canvas.nodeMeta.eventKw", { kw }));
  }
  if (x.type === "prompt_style") {
    return String(t("expertModels.canvas.nodeMeta.promptStyle"));
  }
  return "";
});

const title = computed(() => {
  const x = n.value;
  if (x.type === "base_model") return "BaseModel";
  if (x.type === "lora_adapter") return "LoRA";
  if (x.type === "cloud_model") return "Cloud";
  if (x.type === "event_trigger") return "Event";
  return "PromptStyle";
});

function onStrengthInput(ev: Event): void {
  const v = Number((ev.target as HTMLInputElement).value);
  if (!Number.isFinite(v)) return;
  props.data.onPatch({ strength: v });
}
</script>

<template>
  <div class="efn-root" :class="{ 'efn-root--err': data.error, 'efn-root--sel': selected }">
    <Handle id="in" class="efn-handle" type="target" :position="Position.Top" />
    <div class="efn-head">
      <span class="efn-type">{{ title }}</span>
      <span class="efn-id">{{ id }}</span>
    </div>
    <div class="efn-meta">{{ metaLine }}</div>
    <div v-if="n.type === 'lora_adapter'" class="efn-portrow">
      <label class="efn-lab">{{ t("expertModels.canvas.loraStrengthPort") }}</label>
      <input
        class="efn-range"
        type="range"
        min="0"
        max="2"
        step="0.05"
        :value="n.strength"
        @input="onStrengthInput"
      />
    </div>
    <Handle id="out" class="efn-handle" type="source" :position="Position.Bottom" />
  </div>
</template>

<style scoped>
.efn-root {
  min-width: 200px;
  max-width: 260px;
  padding: 8px 10px 10px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  font-size: 12px;
  line-height: 1.35;
}
.efn-root--sel {
  outline: 2px solid color-mix(in srgb, var(--accent, #357cff) 55%, transparent);
  outline-offset: 1px;
}
.efn-root--err {
  border-color: color-mix(in srgb, var(--danger-600, #c0392b) 70%, var(--border-light));
  animation: efn-shake 0.45s ease-in-out 1;
}
@keyframes efn-shake {
  0%,
  100% {
    transform: translateX(0);
  }
  20% {
    transform: translateX(-3px);
  }
  40% {
    transform: translateX(3px);
  }
  60% {
    transform: translateX(-2px);
  }
  80% {
    transform: translateX(2px);
  }
}
.efn-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 6px;
  margin-bottom: 4px;
}
.efn-type {
  font-weight: 700;
}
.efn-id {
  font-family: ui-monospace, monospace;
  font-size: 10px;
  color: var(--text-secondary);
  word-break: break-all;
  text-align: right;
}
.efn-meta {
  color: var(--text-secondary);
  word-break: break-word;
  margin-bottom: 4px;
}
.efn-portrow {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 4px;
  padding-top: 6px;
  border-top: 1px dashed var(--border-light);
}
.efn-lab {
  font-size: 11px;
  color: var(--text-secondary);
}
.efn-range {
  width: 100%;
}
.efn-handle {
  width: 10px;
  height: 10px;
  background: var(--accent, #357cff);
  border: 2px solid var(--bg-primary);
}
</style>
