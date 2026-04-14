<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useRoleStore } from "../stores/roleStore";
import { useUiStore } from "../stores/uiStore";
import { buildRelationDropdownOptions } from "../utils/relationOptions";
import {
  getPluginResolutionDebug,
  OCLIVE_DEFAULT_RELATION_SENTINEL,
  setEvolutionFactor,
  setRemoteLifeEnabled,
  setSessionPluginBackend,
  setUserRelation,
} from "../utils/tauri-api";
import HelpHint from "./HelpHint.vue";

const roleStore = useRoleStore();
const uiStore = useUiStore();
const localFactor = ref(roleStore.roleInfo.eventImpactFactor);
const busy = ref(false);

const pluginBackends = computed(() => roleStore.roleInfo.pluginBackends);
const pluginBackendsEffective = computed(
  () => roleStore.roleInfo.pluginBackendsEffective,
);
const pluginBackendsSessionOverride = computed(
  () => roleStore.roleInfo.pluginBackendsSessionOverride,
);
const pluginBackendsEffectiveSources = computed(
  () => roleStore.roleInfo.pluginBackendsEffectiveSources,
);
const pluginDebugSnapshot = ref("");
const pluginDebugCopyStatus = ref("");
const localMemoryProviderDraft = ref("");
const sourceLabel: Record<"pack_default" | "session_override" | "env_override", string> = {
  pack_default: "包默认",
  session_override: "会话覆盖",
  env_override: "环境覆盖",
};
const pluginBackendRows = [
  {
    key: "memory",
    label: "Memory",
    options: ["builtin", "builtin_v2", "remote", "local"],
  },
  {
    key: "emotion",
    label: "Emotion",
    options: ["builtin", "builtin_v2", "remote"],
  },
  {
    key: "event",
    label: "Event",
    options: ["builtin", "builtin_v2", "remote"],
  },
  {
    key: "prompt",
    label: "Prompt",
    options: ["builtin", "builtin_v2", "remote"],
  },
  {
    key: "llm",
    label: "LLM",
    options: ["ollama", "remote"],
  },
] as const;

const personalitySourceLabel = computed(() =>
  roleStore.roleInfo.personalitySource === "profile"
    ? "档案（可变正文由对话维护）"
    : "七维向量",
);

const personalitySourceHintParagraphs = computed(() =>
  roleStore.roleInfo.personalitySource === "profile"
    ? [
        "人格来源为 profile：运行时以核心性格档案与数据库中的「可变性格档案」为准；界面七维多为从正文归纳的视图。",
        "与 vector 模式（七维直接参与事件演化）不同；设计说明见仓库 docs/personality-archive-notes.md。",
      ]
    : [
        "人格来源为 vector：事件与情绪按七维精细化调整；与 settings 中 evolution.personality_source 一致。",
      ],
);

const relationRows = computed(() =>
  buildRelationDropdownOptions(
    roleStore.roleInfo.userRelations,
    roleStore.roleInfo.defaultRelation,
  ),
);

watch(
  () => [roleStore.currentRoleId, roleStore.roleInfo.eventImpactFactor] as const,
  () => {
    localFactor.value = roleStore.roleInfo.eventImpactFactor;
  },
);

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

async function onRelationChange(ev: Event) {
  const sel = ev.target as HTMLSelectElement;
  const next = sel.value;
  if (next === roleStore.relationSelectValue) return;
  busy.value = true;
  try {
    const perScene = roleStore.roleInfo.identityBinding === "per_scene";
    if (next === OCLIVE_DEFAULT_RELATION_SENTINEL) {
      if (perScene) {
        await roleStore.setManifestDefaultIdentity(uiStore.sceneId);
      } else {
        await roleStore.setManifestDefaultIdentity();
      }
    } else if (perScene) {
      await roleStore.setSceneUserRelation(uiStore.sceneId, next);
    } else {
      const info = await setUserRelation(roleStore.currentRoleId, next);
      roleStore.applyRoleInfo(info);
    }
  } finally {
    busy.value = false;
  }
}

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

async function commitFactor() {
  const v = localFactor.value;
  if (
    !Number.isFinite(v) ||
    v < 0.05 ||
    v > 5 ||
    Math.abs(v - roleStore.roleInfo.eventImpactFactor) < 1e-9
  ) {
    return;
  }
  busy.value = true;
  try {
    await setEvolutionFactor(roleStore.currentRoleId, v);
    await roleStore.refreshRoleInfo();
  } finally {
    busy.value = false;
  }
}

function onFactorEnter(ev: KeyboardEvent) {
  (ev.target as HTMLInputElement).blur();
}

async function onPluginBackendChange(
  module:
    | "memory"
    | "emotion"
    | "event"
    | "prompt"
    | "llm",
  ev: Event,
) {
  const selected = (ev.target as HTMLSelectElement).value;
  const backend = selected === "__pack_default__" ? null : selected;
  busy.value = true;
  try {
    const info = await setSessionPluginBackend(
      roleStore.currentRoleId,
      module,
      backend,
    );
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
    `env llm_override=${debug.llm_env_override ?? "none"}`,
    `env remote_plugin_url=${debug.remote_plugin_url_configured ? "set" : "unset"}`,
    `env remote_llm_url=${debug.remote_llm_url_configured ? "set" : "unset"}`,
    `local_providers count=${debug.local_provider_count} ids=${debug.local_provider_ids.join(",") || "none"}`,
  ].join("\n");
}

async function copyPluginDebugSnapshot() {
  if (!pluginDebugSnapshot.value) {
    await refreshPluginDebugSnapshot();
  }
  const text = pluginDebugSnapshot.value.trim();
  if (!text) return;
  try {
    if (!navigator.clipboard?.writeText) {
      throw new Error("clipboard API unavailable");
    }
    await navigator.clipboard.writeText(text);
    pluginDebugCopyStatus.value = "已复制";
  } catch {
    pluginDebugCopyStatus.value = "复制失败";
  }
  window.setTimeout(() => {
    pluginDebugCopyStatus.value = "";
  }, 1800);
}
</script>

<template>
  <section class="runtime">
    <div class="meta">
      <p v-if="roleStore.roleInfo.description" class="desc">
        {{ roleStore.roleInfo.description }}
      </p>
      <p class="sub">
        版本 {{ roleStore.roleInfo.version || "—" }} · 作者
        {{ roleStore.roleInfo.author || "—" }}
      </p>
      <p
        class="sub plugin-backends"
        title="settings.json → plugin_backends（见 creator-docs/plugin-and-architecture/PLUGIN_V1.md）"
      >
        模块后端：mem {{ pluginBackends.memory }} · emotion {{ pluginBackends.emotion }} · event {{ pluginBackends.event }} · prompt {{ pluginBackends.prompt }} · llm {{ pluginBackends.llm }}
      </p>
      <p
        class="sub plugin-backends"
        title="会话级覆盖叠加后的实际生效后端"
      >
        会话生效：mem {{ pluginBackendsEffective.memory }} · emotion {{ pluginBackendsEffective.emotion }} · event {{ pluginBackendsEffective.event }} · prompt {{ pluginBackendsEffective.prompt }} · llm {{ pluginBackendsEffective.llm }}
      </p>
      <p v-if="pluginBackendsSessionOverride" class="sub plugin-override-hint">
        当前会话已启用模块覆盖（仅本会话生效，不写入角色包）。
      </p>
      <p class="sub plugin-backends" title="当前会话生效后端的来源">
        来源：mem {{ sourceLabel[pluginBackendsEffectiveSources.memory] }} ·
        emotion {{ sourceLabel[pluginBackendsEffectiveSources.emotion] }} ·
        event {{ sourceLabel[pluginBackendsEffectiveSources.event] }} ·
        prompt {{ sourceLabel[pluginBackendsEffectiveSources.prompt] }} ·
        llm {{ sourceLabel[pluginBackendsEffectiveSources.llm] }}
      </p>
      <p class="sub personality-source-line">
        <span class="ps-inline">
          人格来源：<strong>{{ personalitySourceLabel }}</strong>
          <HelpHint :paragraphs="personalitySourceHintParagraphs" />
        </span>
      </p>
    </div>
    <div v-if="roleStore.interactionImmersive" class="row row-check">
      <label for="remote-life">异地心声</label>
      <input
        id="remote-life"
        type="checkbox"
        :checked="roleStore.roleInfo.remoteLifeEnabled"
        :disabled="busy"
        @change="onRemoteLifeChange"
      />
      <span
        v-if="roleStore.roleInfo.remoteLifePackDefault === true"
        class="hint"
      >包默认建议开</span>
    </div>
    <template v-if="roleStore.roleInfo.userRelations.length > 0">
    <div class="row">
      <label for="rel-select">关系</label>
      <select
        id="rel-select"
        class="select"
        :disabled="busy"
        :value="roleStore.relationSelectValue"
        @change="onRelationChange"
      >
        <option
          v-for="r in relationRows"
          :key="r.id"
          :value="r.id"
        >
          {{ r.name || r.id }}
        </option>
      </select>
    </div>
    <div class="row">
      <label for="evolve-factor">事件影响</label>
      <input
        id="evolve-factor"
        v-model.number="localFactor"
        class="input-num"
        type="number"
        min="0.05"
        max="5"
        step="0.05"
        :disabled="busy"
        @blur="commitFactor"
        @keydown.enter.prevent="onFactorEnter"
      />
    </div>
    </template>
    <div class="backend-grid">
      <div
        v-for="item in pluginBackendRows"
        :key="item.key"
        class="row backend-row"
      >
        <label :for="`backend-${item.key}`">{{ item.label }}</label>
        <select
          :id="`backend-${item.key}`"
          class="select"
          :disabled="busy"
          :value="
            pluginBackendsSessionOverride?.[item.key] ?? '__pack_default__'
          "
          @change="onPluginBackendChange(item.key, $event)"
        >
          <option value="__pack_default__">
            跟随包默认（{{ pluginBackends[item.key] }}）
          </option>
          <option v-for="v in item.options" :key="v" :value="v">{{ v }}</option>
        </select>
      </div>
      <div
        v-if="pluginBackendsEffective.memory === 'local'"
        class="row backend-row local-mem-provider-row"
      >
        <label for="local-mem-provider">Local memory id</label>
        <input
          id="local-mem-provider"
          v-model="localMemoryProviderDraft"
          class="input-text"
          type="text"
          autocomplete="off"
          :disabled="busy"
          placeholder="provider_id，空串清除本会话覆盖"
          @keydown.enter.prevent="commitLocalMemoryProviderId"
        />
        <button
          type="button"
          class="btn tiny"
          :disabled="busy"
          @click="commitLocalMemoryProviderId"
        >
          应用到本会话
        </button>
      </div>
      <div class="row backend-row">
        <label>调试快照</label>
        <button class="btn tiny" :disabled="busy" @click="refreshPluginDebugSnapshot">
          刷新
        </button>
        <button class="btn tiny" :disabled="busy" @click="copyPluginDebugSnapshot">
          复制
        </button>
        <span v-if="pluginDebugCopyStatus" class="debug-copy-status">
          {{ pluginDebugCopyStatus }}
        </span>
      </div>
      <pre v-if="pluginDebugSnapshot" class="backend-debug">{{ pluginDebugSnapshot }}</pre>
    </div>
  </section>
</template>

<style scoped>
.runtime {
  padding: 10px 18px 12px;
  margin: 0;
  font-size: 13px;
  background: var(--bg-primary);
  border-bottom: 1px solid var(--border-light);
  border-radius: 0;
  box-shadow: none;
}
.meta {
  margin-bottom: 10px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border-light);
}
.desc {
  margin: 0 0 6px;
  line-height: 1.45;
  color: var(--text-secondary);
  font-size: 12px;
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
.personality-source-line {
  margin-top: 8px;
}
.ps-inline {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}
.row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}
.row:last-child {
  margin-bottom: 0;
}
.backend-grid {
  margin-top: 12px;
  padding-top: 10px;
  border-top: 1px dashed var(--border-light);
}
.backend-row {
  margin-bottom: 6px;
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
.debug-copy-status {
  font-size: 11px;
  color: var(--text-secondary);
  opacity: 0.9;
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
.input-num {
  width: 100px;
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
  color: inherit;
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
  opacity: 0.9;
}
</style>
