import { computed, ref, watch } from "vue";
import { usePluginStore } from "../stores/pluginStore";
import { useRoleStore } from "../stores/roleStore";
import { setRemoteLifeEnabled, setSessionPluginBackend } from "../utils/tauri-api";
import type { PluginUiTemplateName } from "../components/PluginUITemplates";
import { i18n } from "../i18n";

function t(key: string, params?: Record<string, unknown>): string {
  return String(i18n.global.t(key as any, params as any));
}

export type V2ModuleKey = "llm" | "emotion" | "complex_emotion";
export type V2TypeKey = "builtin" | "remote" | "directory" | "none";
export type V2StatusKey = "enabled" | "disabled" | "needs_config";

export interface PluginV2CardItem {
  id: string;
  title: string;
  module: V2ModuleKey;
  moduleLabel: string;
  type: V2TypeKey;
  status: V2StatusKey;
  sourceLabel: string;
  description: string;
  uiTemplate: PluginUiTemplateName;
  schema: Record<string, unknown>;
}

export interface PluginV2CategoryItem {
  id: string;
  label: string;
  count: number;
}

function normalizeType(backend: string): V2TypeKey {
  if (backend === "remote") return "remote";
  if (backend === "directory") return "directory";
  if (backend === "none") return "none";
  return "builtin";
}

function toSourceLabel(source: string): string {
  if (source === "session_override") return t("pluginManagerV2.sources.sessionOverride");
  if (source === "env_override") return t("pluginManagerV2.sources.envOverride");
  return t("pluginManagerV2.sources.packDefault");
}

export function usePluginManagerV2() {
  const roleStore = useRoleStore();
  const pluginStore = usePluginStore();

  const searchKeyword = ref("");
  const selectedCategory = ref("all");
  const selectedCardId = ref("");

  const directoryOptions = computed(() =>
    pluginStore.catalog.map((c) => ({ value: c.id, label: c.id })),
  );

  const cards = computed<PluginV2CardItem[]>(() => {
    const effective = roleStore.roleInfo.pluginBackendsEffective;
    const defaults = roleStore.roleInfo.pluginBackends;
    const sources = roleStore.roleInfo.pluginBackendsEffectiveSources;
    const dirs = effective.directory_plugins ?? {};
    const overrideDirs = roleStore.roleInfo.pluginBackendsSessionOverride?.directory_plugins ?? {};

    const llmDirectoryId = overrideDirs.llm ?? dirs.llm ?? "";
    const emotionDirectoryId = overrideDirs.emotion ?? dirs.emotion ?? "";

    return [
      {
        id: "llm-main",
        title: t("pluginManagerV2.cards.llmMain.title"),
        module: "llm",
        moduleLabel: t("pluginManagerV2.modules.llm"),
        type: normalizeType(effective.llm),
        status: effective.llm === "directory" && !llmDirectoryId ? "needs_config" : "enabled",
        sourceLabel: toSourceLabel(sources.llm),
        description: t("pluginManagerV2.cards.llmMain.description"),
        uiTemplate: "slot-selector",
        schema: {
          module: "llm",
          current: roleStore.roleInfo.pluginBackendsSessionOverride?.llm ?? "__pack_default__",
          directoryId: llmDirectoryId,
          options: [
            {
              value: "__pack_default__",
              label: t("pluginManagerV2.options.followPackDefault", { v: defaults.llm }),
            },
            { value: "ollama", label: t("pluginManagerV2.options.ollama") },
            { value: "remote", label: t("pluginManagerV2.options.remote") },
            { value: "directory", label: t("pluginManagerV2.options.directory") },
            { value: "none", label: t("pluginManagerV2.options.none") },
          ],
          directoryOptions: directoryOptions.value,
        },
      },
      {
        id: "llm-endpoint",
        title: t("pluginManagerV2.cards.llmEndpoint.title"),
        module: "llm",
        moduleLabel: t("pluginManagerV2.modules.llm"),
        type: "remote",
        status: effective.llm === "remote" ? "enabled" : "disabled",
        sourceLabel: t("pluginManagerV2.sources.envVar"),
        description: t("pluginManagerV2.cards.llmEndpoint.description"),
        uiTemplate: "endpoint-config",
        schema: {
          summary: t("pluginManagerV2.cards.llmEndpoint.summary"),
          fields: [
            {
              name: "OCLIVE_REMOTE_LLM_URL",
              description: t("pluginManagerV2.cards.llmEndpoint.fields.remoteLlmUrl"),
            },
            {
              name: "OCLIVE_REMOTE_PLUGIN_URL",
              description: t("pluginManagerV2.cards.llmEndpoint.fields.remotePluginUrl"),
            },
          ],
        },
      },
      {
        id: "emotion-main",
        title: t("pluginManagerV2.cards.emotionMain.title"),
        module: "emotion",
        moduleLabel: t("pluginManagerV2.modules.emotion"),
        type: normalizeType(effective.emotion),
        status:
          effective.emotion === "directory" && !emotionDirectoryId
            ? "needs_config"
            : "enabled",
        sourceLabel: toSourceLabel(sources.emotion),
        description: t("pluginManagerV2.cards.emotionMain.description"),
        uiTemplate: "slot-selector",
        schema: {
          module: "emotion",
          current:
            roleStore.roleInfo.pluginBackendsSessionOverride?.emotion ?? "__pack_default__",
          directoryId: emotionDirectoryId,
          options: [
            {
              value: "__pack_default__",
              label: t("pluginManagerV2.options.followPackDefault", { v: defaults.emotion }),
            },
            { value: "builtin", label: t("pluginManagerV2.options.builtin") },
            { value: "builtin_v2", label: t("pluginManagerV2.options.builtinV2") },
            { value: "remote", label: t("pluginManagerV2.options.remote") },
            { value: "directory", label: t("pluginManagerV2.options.directory") },
            { value: "none", label: t("pluginManagerV2.options.none") },
          ],
          directoryOptions: directoryOptions.value,
        },
      },
      {
        id: "emotion-endpoint",
        title: t("pluginManagerV2.cards.emotionEndpoint.title"),
        module: "emotion",
        moduleLabel: t("pluginManagerV2.modules.emotion"),
        type: "remote",
        status: effective.emotion === "remote" ? "enabled" : "disabled",
        sourceLabel: t("pluginManagerV2.sources.envVar"),
        description: t("pluginManagerV2.cards.emotionEndpoint.description"),
        uiTemplate: "endpoint-config",
        schema: {
          summary: t("pluginManagerV2.cards.emotionEndpoint.summary"),
          fields: [
            {
              name: "OCLIVE_REMOTE_PLUGIN_URL",
              description: t("pluginManagerV2.cards.emotionEndpoint.fields.remotePluginUrl"),
            },
          ],
        },
      },
      {
        id: "complex-switch",
        title: t("pluginManagerV2.cards.complexSwitch.title"),
        module: "complex_emotion",
        moduleLabel: t("pluginManagerV2.modules.complexEmotion"),
        type: "remote",
        status: roleStore.roleInfo.remoteLifeEnabled ? "enabled" : "disabled",
        sourceLabel: roleStore.roleInfo.remoteLifeEnabled
          ? t("pluginManagerV2.sources.sessionEnabled")
          : t("pluginManagerV2.sources.sessionDisabled"),
        description: t("pluginManagerV2.cards.complexSwitch.description"),
        uiTemplate: "switch-toggle",
        schema: {
          checked: roleStore.roleInfo.remoteLifeEnabled,
          label: t("pluginManagerV2.cards.complexSwitch.label"),
          hint: t("pluginManagerV2.cards.complexSwitch.hint"),
        },
      },
      {
        id: "complex-endpoint",
        title: t("pluginManagerV2.cards.complexEndpoint.title"),
        module: "complex_emotion",
        moduleLabel: t("pluginManagerV2.modules.complexEmotion"),
        type: "remote",
        status: roleStore.roleInfo.remoteLifeEnabled ? "enabled" : "disabled",
        sourceLabel: t("pluginManagerV2.sources.envVar"),
        description: t("pluginManagerV2.cards.complexEndpoint.description"),
        uiTemplate: "endpoint-config",
        schema: {
          summary: t("pluginManagerV2.cards.complexEndpoint.summary"),
          fields: [
            {
              name: "OCLIVE_COMPLEX_EMOTION_URL",
              description: t("pluginManagerV2.cards.complexEndpoint.fields.url"),
            },
            {
              name: "OCLIVE_COMPLEX_EMOTION_TOKEN",
              description: t("pluginManagerV2.cards.complexEndpoint.fields.token"),
            },
          ],
        },
      },
    ];
  });

  const categories = computed<PluginV2CategoryItem[]>(() => {
    const rows = cards.value;
    const countBy = (fn: (x: PluginV2CardItem) => boolean) =>
      rows.filter((x) => fn(x)).length;
    return [
      { id: "all", label: t("pluginManagerV2.categories.all"), count: rows.length },
      {
        id: "module:llm",
        label: t("pluginManagerV2.modules.llm"),
        count: countBy((x) => x.module === "llm"),
      },
      {
        id: "module:emotion",
        label: t("pluginManagerV2.modules.emotion"),
        count: countBy((x) => x.module === "emotion"),
      },
      {
        id: "module:complex_emotion",
        label: t("pluginManagerV2.modules.complexEmotion"),
        count: countBy((x) => x.module === "complex_emotion"),
      },
      { id: "type:builtin", label: t("pluginManagerV2.categories.builtin"), count: countBy((x) => x.type === "builtin") },
      { id: "type:remote", label: t("pluginManagerV2.categories.remote"), count: countBy((x) => x.type === "remote") },
      {
        id: "type:directory",
        label: t("pluginManagerV2.categories.directory"),
        count: countBy((x) => x.type === "directory"),
      },
      {
        id: "type:none",
        label: t("pluginManagerV2.categories.none"),
        count: countBy((x) => x.type === "none"),
      },
      {
        id: "status:enabled",
        label: t("pluginManagerV2.categories.statusEnabled"),
        count: countBy((x) => x.status === "enabled"),
      },
      {
        id: "status:disabled",
        label: t("pluginManagerV2.categories.statusDisabled"),
        count: countBy((x) => x.status === "disabled"),
      },
      {
        id: "status:needs_config",
        label: t("pluginManagerV2.categories.statusNeedsConfig"),
        count: countBy((x) => x.status === "needs_config"),
      },
    ];
  });

  const filteredCards = computed(() => {
    const keyword = searchKeyword.value.trim().toLowerCase();
    return cards.value.filter((item) => {
      if (selectedCategory.value !== "all") {
        const [kind, val] = selectedCategory.value.split(":");
        if (kind === "module" && item.module !== val) return false;
        if (kind === "type" && item.type !== val) return false;
        if (kind === "status" && item.status !== val) return false;
      }
      if (!keyword) return true;
      const hay = `${item.title} ${item.description} ${item.moduleLabel}`.toLowerCase();
      return hay.includes(keyword);
    });
  });

  const selectedCard = computed(() =>
    filteredCards.value.find((x) => x.id === selectedCardId.value) ?? null,
  );

  watch(
    filteredCards,
    (rows) => {
      if (rows.length === 0) {
        selectedCardId.value = "";
        return;
      }
      if (!rows.some((x) => x.id === selectedCardId.value)) {
        selectedCardId.value = rows[0].id;
      }
    },
    { immediate: true },
  );

  async function applyCardChange(
    item: PluginV2CardItem,
    payload: Record<string, unknown>,
  ): Promise<string> {
    if (item.uiTemplate === "endpoint-config") {
      return t("pluginManagerV2.toasts.endpointNoSave");
    }
    if (item.uiTemplate === "switch-toggle") {
      const info = await setRemoteLifeEnabled(
        roleStore.currentRoleId,
        Boolean(payload.enabled),
      );
      roleStore.applyRoleInfo(info);
      return t("pluginManagerV2.toasts.complexSwitchUpdated");
    }

    const module = String((item.schema as { module?: string }).module ?? "");
    if (module !== "llm" && module !== "emotion") {
      throw new Error(t("pluginManagerV2.errors.onlyLlmEmotionSupported"));
    }
    const info = await setSessionPluginBackend(
      roleStore.currentRoleId,
      module,
      (payload.backend as string | null | undefined) ?? undefined,
      undefined,
      undefined,
      (payload.directoryId as string | null | undefined) ?? undefined,
    );
    roleStore.applyRoleInfo(info);
    return t("pluginManagerV2.toasts.writtenToSession");
  }

  return {
    searchKeyword,
    selectedCategory,
    selectedCardId,
    categories,
    filteredCards,
    selectedCard,
    applyCardChange,
  };
}
