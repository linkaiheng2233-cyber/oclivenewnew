<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
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
import { useAppToast } from "../composables/useAppToast";
import { usePluginStore } from "../stores/pluginStore";
import { usePermissionGate } from "../composables/usePermissionGate";

const roleStore = useRoleStore();
const { t } = useI18n();
const { showToast } = useAppToast();
const pluginStore = usePluginStore();
const { ensurePermissionsOrCancel } = usePermissionGate();
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
  pack_default: String(t("pluginBackendSessionPanel.sources.packDefault")),
  session_override: String(t("pluginBackendSessionPanel.sources.sessionOverride")),
  env_override: String(t("pluginBackendSessionPanel.sources.envOverride")),
};
const pluginBackendRows = [
  {
    key: "memory" as const,
    label: String(t("pluginBackendSessionPanel.modules.memory")),
    options: ["builtin", "builtin_v2", "remote", "local", "directory"],
  },
  {
    key: "emotion" as const,
    label: String(t("pluginBackendSessionPanel.modules.emotion")),
    options: ["builtin", "builtin_v2", "remote", "directory"],
  },
  {
    key: "event" as const,
    label: String(t("pluginBackendSessionPanel.modules.event")),
    options: ["builtin", "builtin_v2", "remote", "directory"],
  },
  {
    key: "prompt" as const,
    label: String(t("pluginBackendSessionPanel.modules.prompt")),
    options: ["builtin", "builtin_v2", "remote", "directory"],
  },
  {
    key: "llm" as const,
    label: String(t("pluginBackendSessionPanel.modules.llm")),
    options: ["ollama", "remote", "directory"],
  },
  {
    key: "agent" as const,
    label: String(t("pluginBackendSessionPanel.modules.agent")),
    options: ["builtin", "remote", "directory"],
  },
];
const directoryPluginsPackLine = computed(() => {
  const pb = pluginBackends.value;
  if (!usesDirectoryPlugins(pb)) return "";
  return String(
    t("pluginBackendSessionPanel.directoryPlugins.packLine", {
      v: formatDirectoryPluginSlots(pb.directory_plugins),
    }),
  );
});
const directoryPluginsEffectiveLine = computed(() => {
  const pb = pluginBackendsEffective.value;
  if (!usesDirectoryPlugins(pb)) return "";
  return String(
    t("pluginBackendSessionPanel.directoryPlugins.effectiveLine", {
      v: formatDirectoryPluginSlots(pb.directory_plugins),
    }),
  );
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

  // v1.0 权限弹窗（最小实现）：在切换到 directory/remote 前做前置授权。
  if (backend === "directory") {
    const slotId =
      (roleStore.roleInfo.pluginBackendsEffective?.directory_plugins as any)?.[module] ??
      (roleStore.roleInfo.pluginBackends?.directory_plugins as any)?.[module];
    const pid = String(slotId ?? "").trim();
    if (pid) {
      const entry = (pluginStore.catalog ?? []).find((x) => x.id === pid);
      const declared = (entry?.installMeta?.declaredPermissions ?? [])
        .map((s) => String(s ?? "").trim())
        .filter(Boolean);
      const required = declared.filter((p) => p === "process:spawn" || p === "network:*");
      if (required.length > 0) {
        const gate = await ensurePermissionsOrCancel({
          subjectId: pid,
          required,
          title: String(t("permissionGate.titles.enableDirectoryBackend")),
        });
        if (!gate.ok) return;
      }
    }
  }

  if (backend === "remote") {
    const systemProviderId =
      module === "llm"
        ? "system:remote_llm_http"
        : module === "agent"
          ? "system:remote_agent_http"
          : "system:remote_plugin_http";
    const gate = await ensurePermissionsOrCancel({
      subjectId: systemProviderId,
      required: ["network:*"],
      title: String(t("permissionGate.titles.enableRemoteBackend")),
    });
    if (!gate.ok) return;
  }

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
    pluginDebugCopyStatus.value = String(t("pluginBackendSessionPanel.debugSnapshot.copied"));
  } catch {
    pluginDebugCopyStatus.value = String(t("pluginBackendSessionPanel.debugSnapshot.copyFailed"));
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
    packStatus.value = String(t("pluginBackendSessionPanel.pack.noTargetHint"));
    return;
  }
  busy.value = true;
  try {
    const r = await packPlugin(target);
    packStatus.value = String(
      t("pluginBackendSessionPanel.pack.done", {
        path: r.archive_path,
        sha: `${r.sha256.slice(0, 12)}...`,
      }),
    );
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
      <span v-html="t('pluginBackendSessionPanel.leadHtml')" />
    </p>
    <div class="pb-meta">
      <p class="sub plugin-backends" :title="String(t('pluginBackendSessionPanel.meta.packTitle'))">
        {{ t("pluginBackendSessionPanel.meta.packLabel") }}：mem {{ pluginBackends.memory }} · emotion {{ pluginBackends.emotion }} · event
        {{ pluginBackends.event }} · prompt {{ pluginBackends.prompt }} · llm {{ pluginBackends.llm }} · agent {{ pluginBackends.agent }}
      </p>
      <p class="sub plugin-backends" :title="String(t('pluginBackendSessionPanel.meta.sessionEffectiveTitle'))">
        {{ t("pluginBackendSessionPanel.meta.sessionEffectiveLabel") }}：mem {{ pluginBackendsEffective.memory }} · emotion {{ pluginBackendsEffective.emotion }} · event
        {{ pluginBackendsEffective.event }} · prompt {{ pluginBackendsEffective.prompt }} · llm {{ pluginBackendsEffective.llm }} · agent {{ pluginBackendsEffective.agent }}
      </p>
      <p v-if="directoryPluginsPackLine" class="sub plugin-backends">{{ directoryPluginsPackLine }}</p>
      <p v-if="directoryPluginsEffectiveLine" class="sub plugin-backends">{{ directoryPluginsEffectiveLine }}</p>
      <p v-if="pluginBackendsSessionOverride" class="sub plugin-override-hint">
        {{ t("pluginBackendSessionPanel.meta.hasSessionOverrideHint") }}
      </p>
      <p class="sub plugin-backends" :title="String(t('pluginBackendSessionPanel.meta.sourcesTitle'))">
        {{ t("pluginBackendSessionPanel.meta.sourcesLabel") }}：mem {{ sourceLabel[pluginBackendsEffectiveSources.memory] }} · emotion
        {{ sourceLabel[pluginBackendsEffectiveSources.emotion] }} · event
        {{ sourceLabel[pluginBackendsEffectiveSources.event] }} · prompt
        {{ sourceLabel[pluginBackendsEffectiveSources.prompt] }} · llm {{ sourceLabel[pluginBackendsEffectiveSources.llm] }} · agent {{ sourceLabel[pluginBackendsEffectiveSources.agent] }}
      </p>
    </div>
    <div v-if="roleStore.interactionImmersive" class="row row-check">
      <label for="pb-remote-life">{{ t("pluginBackendSessionPanel.remoteLife.label") }}</label>
      <input
        id="pb-remote-life"
        type="checkbox"
        :checked="roleStore.roleInfo.remoteLifeEnabled"
        :disabled="busy"
        @change="onRemoteLifeChange"
      />
      <span v-if="roleStore.roleInfo.remoteLifePackDefault === true" class="hint">{{ t("pluginBackendSessionPanel.remoteLife.packDefaultHint") }}</span>
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
          <option value="__pack_default__">
            {{ t("pluginBackendSessionPanel.followPackDefault", { v: pluginBackends[item.key] }) }}
          </option>
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
          :placeholder="String(t('pluginBackendSessionPanel.localMemory.placeholder'))"
          @keydown.enter.prevent="commitLocalMemoryProviderId"
        />
        <button type="button" class="btn tiny" :disabled="busy" @click="commitLocalMemoryProviderId">
          {{ t("pluginBackendSessionPanel.localMemory.applyToSession") }}
        </button>
      </div>
      <div class="row backend-row">
        <label>{{ t("pluginBackendSessionPanel.debugSnapshot.label") }}</label>
        <button type="button" class="btn tiny" :disabled="busy" @click="refreshPluginDebugSnapshot">
          {{ t("common.refresh") }}
        </button>
        <button type="button" class="btn tiny" :disabled="busy" @click="copyPluginDebugSnapshot">
          {{ t("common.copy") }}
        </button>
        <span v-if="pluginDebugCopyStatus" class="debug-copy-status">{{ pluginDebugCopyStatus }}</span>
      </div>
      <div class="row backend-row">
        <label>{{ t("pluginBackendSessionPanel.pack.label") }}</label>
        <button type="button" class="btn tiny" :disabled="busy" @click="onPackCurrentPlugin">
          {{ t("pluginBackendSessionPanel.pack.oneClick") }}
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
