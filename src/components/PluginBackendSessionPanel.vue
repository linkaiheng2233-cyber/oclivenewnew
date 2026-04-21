<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useRoleStore } from "../stores/roleStore";
import {
  packPlugin,
  getPluginResolutionDebug,
  setRemoteLifeEnabled,
  setSessionPluginBackend,
} from "../utils/tauri-api";
import {
  formatDirectoryPluginSlots,
  usesDirectoryPlugins,
} from "../utils/pluginBackendsDisplay";
import AgentDebugPanel from "./AgentDebugPanel.vue";

const roleStore = useRoleStore();
const busy = ref(false);
const pluginBackends = computed(() => roleStore.roleInfo.pluginBackends);
const pluginBackendsEffective = computed(() => roleStore.roleInfo.pluginBackendsEffective);
const pluginBackendsSessionOverride = computed(
  () => roleStore.roleInfo.pluginBackendsSessionOverride,
);
const pluginBackendsEffectiveSources = computed(
  () => roleStore.roleInfo.pluginBackendsEffectiveSources,
);
const pluginDebugSnapshot = ref("");
const pluginDebugCopyStatus = ref("");
const localMemoryProviderDraft = ref("");
const packStatus = ref("");
const sourceLabel: Record<"pack_default" | "session_override" | "env_override", string> = {
  pack_default: "包默认",
  session_override: "会话覆盖",
  env_override: "环境覆盖",
};
const pluginBackendRows = [
  { key: "memory" as const, label: "Memory", options: ["builtin", "builtin_v2", "remote", "local", "directory"] },
  { key: "emotion" as const, label: "Emotion", options: ["builtin", "builtin_v2", "remote", "directory"] },
  { key: "event" as const, label: "Event", options: ["builtin", "builtin_v2", "remote", "directory"] },
  { key: "prompt" as const, label: "Prompt", options: ["builtin", "builtin_v2", "remote", "directory"] },
  { key: "llm" as const, label: "LLM", options: ["ollama", "remote", "directory"] },
  { key: "agent" as const, label: "Agent", options: ["builtin", "remote", "directory"] },
];
const directoryPluginsPackLine = computed(() => {
  const pb = pluginBackends.value;
  if (!usesDirectoryPlugins(pb)) return "";
  return `包 · directory_plugins：${formatDirectoryPluginSlots(pb.directory_plugins)}`;
});
const directoryPluginsEffectiveLine = computed(() => {
  const pb = pluginBackendsEffective.value;
  if (!usesDirectoryPlugins(pb)) return "";
  return `生效 · directory_plugins：${formatDirectoryPluginSlots(pb.directory_plugins)}`;
});
watch(
  () =>
    [
      roleStore.currentRoleId,
      roleStore.roleInfo.pluginBackendsEffective.memory,
      roleStore.roleInfo.pluginBackendsEffective.local_memory_provider_id ?? "",
    ] as const,
  ([, , id]) => {
    localMemoryProviderDraft.value = id;
  },
  { immediate: true },
);
async function onRemoteLifeChange(ev: Event) {
  const checked = (ev.target as HTMLInputElement).checked;
  busy.value = true;
  try {
    const info = await setRemoteLifeEnabled(roleStore.currentRoleId, checked);
    roleStore.applyRoleInfo(info);
  } finally {
    busy.value = false;
  }
}
async function onPluginBackendChange(
  module: "memory" | "emotion" | "event" | "prompt" | "llm" | "agent",
  ev: Event,
) {
  const selected = (ev.target as HTMLSelectElement).value;
  const backend = selected === "__pack_default__" ? null : selected;
  busy.value = true;
  try {
    const info = await setSessionPluginBackend(roleStore.currentRoleId, module, backend);
    roleStore.applyRoleInfo(info);
    await refreshPluginDebugSnapshot();
  } finally {
    busy.value = false;
  }
}
async function commitLocalMemoryProviderId() {
  busy.value = true;
  try {
    const info = await setSessionPluginBackend(
      roleStore.currentRoleId,
      "memory",
      undefined,
      localMemoryProviderDraft.value.trim(),
    );
    roleStore.applyRoleInfo(info);
    await refreshPluginDebugSnapshot();
  } finally {
    busy.value = false;
  }
}
async function refreshPluginDebugSnapshot() {
  const debug = await getPluginResolutionDebug(roleStore.currentRoleId);
  pluginDebugSnapshot.value = [
    `meta app=${debug.app_version} api=${debug.api_version} schema=${debug.schema_version}`,
    `session=${debug.session_namespace}`,
    `effective mem=${debug.plugin_backends_effective.memory}(${debug.plugin_backends_effective_sources.memory}) local_mem_id=${debug.plugin_backends_effective.local_memory_provider_id ?? "none"}`,
    `effective emotion=${debug.plugin_backends_effective.emotion}(${debug.plugin_backends_effective_sources.emotion})`,
    `effective event=${debug.plugin_backends_effective.event}(${debug.plugin_backends_effective_sources.event})`,
    `effective prompt=${debug.plugin_backends_effective.prompt}(${debug.plugin_backends_effective_sources.prompt})`,
    `effective llm=${debug.plugin_backends_effective.llm}(${debug.plugin_backends_effective_sources.llm})`,
    `effective agent=${debug.plugin_backends_effective.agent}(${debug.plugin_backends_effective_sources.agent})`,
    `pack directory_plugins=${formatDirectoryPluginSlots(debug.plugin_backends_pack_default.directory_plugins)}`,
    `effective directory_plugins=${formatDirectoryPluginSlots(debug.plugin_backends_effective.directory_plugins)}`,
    `env llm_override=${debug.llm_env_override ?? "none"}`,
    `env remote_plugin_url=${debug.remote_plugin_url_configured ? "set" : "unset"}`,
    `env remote_llm_url=${debug.remote_llm_url_configured ? "set" : "unset"}`,
    `local_providers count=${debug.local_provider_count} ids=${debug.local_provider_ids.join(",") || "none"}`,
  ].join("\n");
}
async function copyPluginDebugSnapshot() {
  if (!pluginDebugSnapshot.value) await refreshPluginDebugSnapshot();
  const text = pluginDebugSnapshot.value.trim();
  if (!text) return;
  try {
    if (!navigator.clipboard?.writeText) throw new Error("clipboard API unavailable");
    await navigator.clipboard.writeText(text);
    pluginDebugCopyStatus.value = "已复制";
  } catch {
    pluginDebugCopyStatus.value = "复制失败";
  }
  window.setTimeout(() => {
    pluginDebugCopyStatus.value = "";
  }, 1800);
}

async function onPackCurrentPlugin(): Promise<void> {
  const active = roleStore.roleInfo.pluginBackendsEffective.directory_plugins?.agent;
  const target =
    active && active.trim()
      ? active.trim()
      : roleStore.roleInfo.pluginBackendsEffective.directory_plugins?.llm?.trim() || "";
  if (!target) {
    packStatus.value = "请先在目录插件槽位中配置目标插件（agent 或 llm）。";
    return;
  }
  busy.value = true;
  try {
    const r = await packPlugin(target);
    packStatus.value = `打包完成：${r.archive_path}（sha256=${r.sha256.slice(0, 12)}...）`;
  } catch (e) {
    packStatus.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="pb-root">
    <p class="pb-lead">
      以下为 <strong>settings.json → plugin_backends</strong> 的包默认与会话级覆盖；不写入磁盘角色包。
    </p>
    <div class="pb-meta">
      <p class="sub plugin-backends" title="settings.json → plugin_backends">
        模块后端：mem {{ pluginBackends.memory }} · emotion {{ pluginBackends.emotion }} · event
        {{ pluginBackends.event }} · prompt {{ pluginBackends.prompt }} · llm {{ pluginBackends.llm }} · agent {{ pluginBackends.agent }}
      </p>
      <p class="sub plugin-backends" title="会话生效">
        会话生效：mem {{ pluginBackendsEffective.memory }} · emotion {{ pluginBackendsEffective.emotion }} · event
        {{ pluginBackendsEffective.event }} · prompt {{ pluginBackendsEffective.prompt }} · llm {{ pluginBackendsEffective.llm }} · agent {{ pluginBackendsEffective.agent }}
      </p>
      <p v-if="directoryPluginsPackLine" class="sub plugin-backends">{{ directoryPluginsPackLine }}</p>
      <p v-if="directoryPluginsEffectiveLine" class="sub plugin-backends">{{ directoryPluginsEffectiveLine }}</p>
      <p v-if="pluginBackendsSessionOverride" class="sub plugin-override-hint">
        当前会话已启用模块覆盖（仅本会话生效，不写入角色包）。
      </p>
      <p class="sub plugin-backends" title="来源">
        来源：mem {{ sourceLabel[pluginBackendsEffectiveSources.memory] }} · emotion
        {{ sourceLabel[pluginBackendsEffectiveSources.emotion] }} · event
        {{ sourceLabel[pluginBackendsEffectiveSources.event] }} · prompt
        {{ sourceLabel[pluginBackendsEffectiveSources.prompt] }} · llm {{ sourceLabel[pluginBackendsEffectiveSources.llm] }} · agent {{ sourceLabel[pluginBackendsEffectiveSources.agent] }}
      </p>
    </div>
    <div v-if="roleStore.interactionImmersive" class="row row-check">
      <label for="pb-remote-life">异地心声</label>
      <input
        id="pb-remote-life"
        type="checkbox"
        :checked="roleStore.roleInfo.remoteLifeEnabled"
        :disabled="busy"
        @change="onRemoteLifeChange"
      />
      <span v-if="roleStore.roleInfo.remoteLifePackDefault === true" class="hint">包默认建议开</span>
    </div>
    <div class="backend-grid">
      <div v-for="item in pluginBackendRows" :key="item.key" class="row backend-row">
        <label :for="`pb-backend-${item.key}`">{{ item.label }}</label>
        <select
          :id="`pb-backend-${item.key}`"
          class="select"
          :disabled="busy"
          :value="pluginBackendsSessionOverride?.[item.key] ?? '__pack_default__'"
          @change="onPluginBackendChange(item.key, $event)"
        >
          <option value="__pack_default__">跟随包默认（{{ pluginBackends[item.key] }}）</option>
          <option v-for="v in item.options" :key="v" :value="v">{{ v }}</option>
        </select>
      </div>
      <div
        v-if="pluginBackendsEffective.memory === 'local'"
        class="row backend-row local-mem-provider-row"
      >
        <label for="pb-local-mem-provider">Local memory id</label>
        <input
          id="pb-local-mem-provider"
          v-model="localMemoryProviderDraft"
          class="input-text"
          type="text"
          autocomplete="off"
          :disabled="busy"
          placeholder="provider_id，空串清除本会话覆盖"
          @keydown.enter.prevent="commitLocalMemoryProviderId"
        />
        <button type="button" class="btn tiny" :disabled="busy" @click="commitLocalMemoryProviderId">
          应用到本会话
        </button>
      </div>
      <div class="row backend-row">
        <label>调试快照</label>
        <button type="button" class="btn tiny" :disabled="busy" @click="refreshPluginDebugSnapshot">
          刷新
        </button>
        <button type="button" class="btn tiny" :disabled="busy" @click="copyPluginDebugSnapshot">
          复制
        </button>
        <span v-if="pluginDebugCopyStatus" class="debug-copy-status">{{ pluginDebugCopyStatus }}</span>
      </div>
      <div class="row backend-row">
        <label>打包插件</label>
        <button type="button" class="btn tiny" :disabled="busy" @click="onPackCurrentPlugin">
          一键打包（agent/llm）
        </button>
        <span v-if="packStatus" class="debug-copy-status">{{ packStatus }}</span>
      </div>
      <pre v-if="pluginDebugSnapshot" class="backend-debug">{{ pluginDebugSnapshot }}</pre>
    </div>
  </div>
  <AgentDebugPanel />
</template>

<style scoped>
.pb-root {
  font-size: 13px;
  color: var(--text-primary);
}
.pb-lead {
  margin: 0 0 12px;
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-secondary);
}
.pb-meta {
  margin-bottom: 12px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border-light);
}
.sub {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
  opacity: 0.9;
}
.plugin-backends {
  margin-top: 6px;
  font-family: ui-monospace, monospace;
  font-size: 11px;
  line-height: 1.4;
  word-break: break-word;
}
.plugin-override-hint {
  margin-top: 6px;
  color: var(--accent, #6b8cff);
}
.row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}
.backend-grid {
  margin-top: 4px;
  padding-top: 10px;
  border-top: 1px dashed var(--border-light);
}
.backend-debug {
  margin: 8px 0 0;
  padding: 8px 10px;
  border: 1px solid var(--border-light);
  border-radius: 8px;
  background: var(--panel-bg-soft);
  color: var(--text-secondary);
  font-size: 12px;
  line-height: 1.45;
  white-space: pre-wrap;
  word-break: break-word;
}
label {
  min-width: 72px;
  color: var(--text-secondary);
}
.select {
  flex: 1;
  padding: 6px 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: inherit;
}
.local-mem-provider-row .input-text {
  flex: 1;
  min-width: 120px;
  padding: 6px 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  font-family: ui-monospace, monospace;
  font-size: 12px;
}
.row-check input[type="checkbox"] {
  width: auto;
  accent-color: var(--accent, #6b8cff);
}
.hint {
  font-size: 11px;
  color: var(--text-secondary);
}
.btn.tiny {
  font-size: 12px;
  padding: 4px 10px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
}
.btn.tiny:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.debug-copy-status {
  font-size: 11px;
  color: var(--text-secondary);
}
</style>
