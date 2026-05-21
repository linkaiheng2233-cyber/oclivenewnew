<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useRoleStore } from "../stores/roleStore";
import { usePluginStore } from "../stores/pluginStore";
import { useAppToast } from "../composables/useAppToast";
import { setSessionPluginBackend } from "../utils/tauri-api";
import { formatDirectoryPluginSlots } from "../utils/pluginBackendsDisplay";

const emit = defineEmits<{
  "focus-plugin": [pluginId: string];
}>();

const roleStore = useRoleStore();
const pluginStore = usePluginStore();
const { showToast } = useAppToast();
const { t } = useI18n();

const busy = ref(false);
const expandedDirectory = ref<string | null>(null);

type CoreModule = "memory" | "emotion" | "event" | "prompt" | "llm" | "agent";

const coreModules: {
  key: CoreModule;
  labelKey: string;
  icon: string;
  options: string[];
}[] = [
  { key: "memory", labelKey: "pluginWorkbench.graph.memory", icon: "🧠", options: ["builtin", "builtin_v2", "remote", "local", "directory"] },
  { key: "emotion", labelKey: "pluginWorkbench.graph.emotion", icon: "💭", options: ["builtin", "builtin_v2", "remote", "directory"] },
  { key: "event", labelKey: "pluginWorkbench.graph.event", icon: "⚡", options: ["builtin", "builtin_v2", "remote", "directory"] },
  { key: "prompt", labelKey: "pluginWorkbench.graph.prompt", icon: "📝", options: ["builtin", "builtin_v2", "remote", "directory"] },
  { key: "llm", labelKey: "pluginWorkbench.graph.llm", icon: "🤖", options: ["ollama", "remote", "directory"] },
  { key: "agent", labelKey: "pluginWorkbench.graph.agent", icon: "🛠", options: ["builtin", "remote", "directory"] },
];

const pluginBackends = computed(() => roleStore.roleInfo.pluginBackends);
const pluginBackendsEffective = computed(() => roleStore.roleInfo.pluginBackendsEffective);
const pluginBackendsSessionOverride = computed(
  () => roleStore.roleInfo.pluginBackendsSessionOverride,
);

function effectiveBackend(key: CoreModule): string {
  return String(pluginBackendsEffective.value[key] ?? "");
}

function lineClass(key: CoreModule): string {
  const b = effectiveBackend(key);
  if (b === "remote") return "arch-line arch-line--remote";
  if (b === "directory") return "arch-line arch-line--directory";
  return "arch-line arch-line--builtin";
}

function directoryLabel(key: CoreModule): string {
  const slots = pluginBackendsEffective.value.directory_plugins;
  const id = slots?.[key]?.trim();
  return id ?? "";
}

async function onBackendChange(module: CoreModule, ev: Event) {
  const selected = (ev.target as HTMLSelectElement).value;
  const backend = selected === "__pack_default__" ? null : selected;
  busy.value = true;
  try {
    const info = await setSessionPluginBackend(roleStore.currentRoleId, module, backend);
    roleStore.applyRoleInfo(info);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    busy.value = false;
  }
}

function toggleDirectoryExpand(key: string) {
  expandedDirectory.value = expandedDirectory.value === key ? null : key;
}

function onFocusPlugin(id: string) {
  emit("focus-plugin", id);
}

const directorySummary = computed(() => {
  const pb = pluginBackendsEffective.value;
  if (!pb.directory_plugins) return "";
  return formatDirectoryPluginSlots(pb.directory_plugins);
});
</script>

<template>
  <div class="arch-root">
    <p class="arch-lead">{{ t("pluginWorkbench.graph.lead") }}</p>
    <div class="arch-canvas" role="img" :aria-label="t('pluginWorkbench.graph.canvasAria')">
      <svg class="arch-svg" viewBox="0 0 520 340" aria-hidden="true">
        <line
          v-for="(mod, i) in coreModules"
          :key="'ln-' + mod.key"
          :class="lineClass(mod.key)"
          x1="118"
          y1="170"
          :x2="320"
          :y2="48 + i * 44"
        />
        <line class="arch-line arch-line--builtin" x1="118" y1="170" x2="200" y2="300" />
      </svg>

      <div class="arch-kernel" :title="t('pluginWorkbench.graph.kernelTitle')">
        <span class="arch-kernel-ico" aria-hidden="true">⚙️</span>
        <span class="arch-kernel-lbl">{{ t("pluginWorkbench.graph.kernel") }}</span>
        <span class="arch-kernel-sub">process_message</span>
      </div>

      <div
        v-for="(mod, i) in coreModules"
        :key="mod.key"
        class="arch-node"
        :style="{ top: `${8 + i * 13.5}%` }"
      >
        <div class="arch-node-head">
          <span aria-hidden="true">{{ mod.icon }}</span>
          <span class="arch-node-title">{{ t(mod.labelKey) }}</span>
        </div>
        <select
          class="arch-select"
          :disabled="busy"
          :aria-label="t('pluginWorkbench.graph.backendSelect', { slot: t(mod.labelKey) })"
          :value="pluginBackendsSessionOverride?.[mod.key] ?? '__pack_default__'"
          @change="onBackendChange(mod.key, $event)"
        >
          <option value="__pack_default__">
            {{
              t("pluginWorkbench.graph.followPack", {
                value: pluginBackends[mod.key],
              })
            }}
          </option>
          <option v-for="v in mod.options" :key="v" :value="v">{{ v }}</option>
        </select>
        <p class="arch-effective">
          {{ t("pluginWorkbench.graph.effective") }}: <strong>{{ effectiveBackend(mod.key) }}</strong>
        </p>
        <div v-if="effectiveBackend(mod.key) === 'directory'" class="arch-dir-tags">
          <button
            v-if="directoryLabel(mod.key)"
            type="button"
            class="arch-tag"
            @click="onFocusPlugin(directoryLabel(mod.key))"
          >
            {{ directoryLabel(mod.key) }}
          </button>
          <button
            v-else-if="directorySummary"
            type="button"
            class="arch-tag arch-tag--muted"
            @click="toggleDirectoryExpand(mod.key)"
          >
            {{ t("pluginWorkbench.graph.pickDirectory") }}
          </button>
        </div>
      </div>

      <div class="arch-complex">
        <div class="arch-node-head">
          <span aria-hidden="true">🎭</span>
          <span class="arch-node-title">{{ t("pluginWorkbench.graph.complexEmotion") }}</span>
        </div>
        <p class="arch-effective">
          {{ t("pluginWorkbench.graph.complexHint") }}
        </p>
      </div>
    </div>

    <div class="arch-legend" role="list" :aria-label="t('pluginWorkbench.graph.legendAria')">
      <span class="arch-legend-item" role="listitem">
        <span class="arch-swatch arch-swatch--builtin" />{{ t("pluginWorkbench.graph.legendBuiltin") }}
      </span>
      <span class="arch-legend-item" role="listitem">
        <span class="arch-swatch arch-swatch--remote" />{{ t("pluginWorkbench.graph.legendRemote") }}
      </span>
      <span class="arch-legend-item" role="listitem">
        <span class="arch-swatch arch-swatch--directory" />{{ t("pluginWorkbench.graph.legendDirectory") }}
      </span>
    </div>
  </div>
</template>

<style scoped>
.arch-root {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.arch-lead {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.arch-canvas {
  position: relative;
  min-height: 360px;
  border: 1px dashed var(--border-light);
  border-radius: var(--radius-card);
  background: color-mix(in srgb, var(--bg-elevated) 65%, var(--bg-primary));
  overflow: hidden;
}
.arch-svg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}
.arch-line {
  stroke-width: 2;
  fill: none;
}
.arch-line--builtin {
  stroke: #3d9a5d;
  stroke-dasharray: none;
}
.arch-line--remote {
  stroke: #3b82c4;
  stroke-dasharray: 6 4;
}
.arch-line--directory {
  stroke: #8b5cf6;
  stroke-dasharray: 2 4;
}
.arch-kernel {
  position: absolute;
  left: 4%;
  top: 38%;
  width: 108px;
  text-align: center;
  padding: 12px 8px;
  border-radius: 50%;
  border: 2px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
  z-index: 2;
}
.arch-kernel-ico {
  font-size: 28px;
  display: block;
}
.arch-kernel-lbl {
  display: block;
  font-size: 12px;
  font-weight: 600;
  margin-top: 4px;
}
.arch-kernel-sub {
  display: block;
  font-size: 10px;
  color: var(--text-secondary);
  font-family: ui-monospace, monospace;
}
.arch-node,
.arch-complex {
  position: absolute;
  right: 4%;
  width: min(240px, 42%);
  padding: 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  z-index: 2;
}
.arch-complex {
  bottom: 6%;
  left: 28%;
  right: auto;
  width: min(220px, 38%);
}
.arch-node-head {
  display: flex;
  align-items: center;
  gap: 6px;
  font-weight: 600;
  font-size: 13px;
  margin-bottom: 6px;
}
.arch-select {
  width: 100%;
  padding: 6px 8px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  font-size: 12px;
}
.arch-effective {
  margin: 6px 0 0;
  font-size: 11px;
  color: var(--text-secondary);
}
.arch-dir-tags {
  margin-top: 6px;
}
.arch-tag {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: var(--radius-pill);
  border: 1px solid color-mix(in srgb, #8b5cf6 40%, var(--border-light));
  background: color-mix(in srgb, #8b5cf6 10%, transparent);
  cursor: pointer;
}
.arch-tag--muted {
  color: var(--text-secondary);
}
.arch-legend {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
  font-size: 11px;
  color: var(--text-secondary);
}
.arch-legend-item {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
.arch-swatch {
  width: 20px;
  height: 0;
  border-top: 3px solid;
}
.arch-swatch--builtin {
  border-color: #3d9a5d;
}
.arch-swatch--remote {
  border-color: #3b82c4;
  border-top-style: dashed;
}
.arch-swatch--directory {
  border-color: #8b5cf6;
  border-top-style: dotted;
}
</style>
