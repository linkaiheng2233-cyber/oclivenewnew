import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { usePluginStore } from "../stores/pluginStore";
import { useRoleStore } from "../stores/roleStore";
import {
  setRemoteLifeEnabled,
  setSessionPluginBackend,
  type PluginBackendSource,
} from "../utils/tauri-api";
import type { PluginUiTemplateName } from "../components/PluginUITemplates";

export type V2ModuleKey = "llm" | "emotion" | "complex_emotion";
export type V2TypeKey = "builtin" | "remote" | "directory";
export type V2StatusKey = "enabled" | "disabled" | "needs_config";

export interface PluginV2CardItem {
  id: string;
  title: string;
  module: V2ModuleKey;
  moduleLabel: string;
  type: V2TypeKey;
  status: V2StatusKey;
  /** Effective backend resolution source (drives chip styling). */
  sourceKey: PluginBackendSource;
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
  return "builtin";
}

function sourceLabelFor(
  t: (key: string, values?: Record<string, unknown>) => string,
  source: PluginBackendSource,
): string {
  const map: Record<PluginBackendSource, string> = {
    session_override: "pluginManager.source.session_override",
    env_override: "pluginManager.source.env_override",
    pack_default: "pluginManager.source.pack_default",
  };
  return t(map[source]);
}

export function usePluginManagerV2() {
  const { t, locale } = useI18n();
  const roleStore = useRoleStore();
  const pluginStore = usePluginStore();

  const searchKeyword = ref("");
  const selectedCategory = ref("all");
  const selectedCardId = ref("");

  const directoryOptions = computed(() =>
    pluginStore.catalog.map((c) => ({ value: c.id, label: c.id })),
  );

  const cards = computed<PluginV2CardItem[]>(() => {
    void locale.value;
    const effective = roleStore.roleInfo.pluginBackendsEffective;
    const defaults = roleStore.roleInfo.pluginBackends;
    const sources = roleStore.roleInfo.pluginBackendsEffectiveSources;
    const dirs = effective.directory_plugins ?? {};
    const overrideDirs = roleStore.roleInfo.pluginBackendsSessionOverride?.directory_plugins ?? {};

    const llmDirectoryId = overrideDirs.llm ?? dirs.llm ?? "";
    const emotionDirectoryId = overrideDirs.emotion ?? dirs.emotion ?? "";

    const llmPackOpt = t("pluginManager.cards.optionPackDefault", {
      backend: defaults.llm,
    });
    const emotionPackOpt = t("pluginManager.cards.optionPackDefault", {
      backend: defaults.emotion,
    });

    return [
      {
        id: "llm-main",
        title: t("pluginManager.cards.llmMain.title"),
        module: "llm" as const,
        moduleLabel: t("pluginTerms.module.llm"),
        type: normalizeType(effective.llm),
        status:
          effective.llm === "directory" && !llmDirectoryId ? "needs_config" : "enabled",
        sourceKey: sources.llm,
        sourceLabel: sourceLabelFor(t, sources.llm),
        description: t("pluginManager.cards.llmMain.description"),
        uiTemplate: "slot-selector",
        schema: {
          module: "llm",
          current: roleStore.roleInfo.pluginBackendsSessionOverride?.llm ?? "__pack_default__",
          directoryId: llmDirectoryId,
          options: [
            { value: "__pack_default__", label: llmPackOpt },
            { value: "ollama", label: t("pluginTerms.backend.ollama") },
            { value: "remote", label: t("pluginTerms.backend.remote") },
            { value: "directory", label: t("pluginTerms.backend.directory") },
          ],
          directoryOptions: directoryOptions.value,
        },
      },
      {
        id: "llm-endpoint",
        title: t("pluginManager.cards.llmEndpoint.title"),
        module: "llm" as const,
        moduleLabel: t("pluginTerms.module.llm"),
        type: "remote" as const,
        status: effective.llm === "remote" ? "enabled" : "disabled",
        sourceKey: "env_override",
        sourceLabel: t("pluginManager.env.label"),
        description: t("pluginManager.cards.llmEndpoint.description"),
        uiTemplate: "endpoint-config",
        schema: {
          summary: t("pluginManager.cards.llmEndpoint.summary"),
          fields: [
            {
              name: "OCLIVE_REMOTE_LLM_URL",
              description: t("pluginManager.cards.llmEndpoint.fieldLlmUrl"),
            },
            {
              name: "OCLIVE_REMOTE_PLUGIN_URL",
              description: t("pluginManager.cards.llmEndpoint.fieldPluginUrl"),
            },
          ],
        },
      },
      {
        id: "emotion-main",
        title: t("pluginManager.cards.emotionMain.title"),
        module: "emotion" as const,
        moduleLabel: t("pluginTerms.module.emotion"),
        type: normalizeType(effective.emotion),
        status:
          effective.emotion === "directory" && !emotionDirectoryId
            ? "needs_config"
            : "enabled",
        sourceKey: sources.emotion,
        sourceLabel: sourceLabelFor(t, sources.emotion),
        description: t("pluginManager.cards.emotionMain.description"),
        uiTemplate: "slot-selector",
        schema: {
          module: "emotion",
          current:
            roleStore.roleInfo.pluginBackendsSessionOverride?.emotion ?? "__pack_default__",
          directoryId: emotionDirectoryId,
          options: [
            { value: "__pack_default__", label: emotionPackOpt },
            { value: "builtin", label: t("pluginTerms.type.builtin") },
            { value: "builtin_v2", label: t("pluginTerms.backend.builtin_v2") },
            { value: "remote", label: t("pluginTerms.backend.remote") },
            { value: "directory", label: t("pluginTerms.backend.directory") },
          ],
          directoryOptions: directoryOptions.value,
        },
      },
      {
        id: "emotion-endpoint",
        title: t("pluginManager.cards.emotionEndpoint.title"),
        module: "emotion" as const,
        moduleLabel: t("pluginTerms.module.emotion"),
        type: "remote" as const,
        status: effective.emotion === "remote" ? "enabled" : "disabled",
        sourceKey: "env_override",
        sourceLabel: t("pluginManager.env.label"),
        description: t("pluginManager.cards.emotionEndpoint.description"),
        uiTemplate: "endpoint-config",
        schema: {
          summary: t("pluginManager.cards.emotionEndpoint.summary"),
          fields: [
            {
              name: "OCLIVE_REMOTE_PLUGIN_URL",
              description: t("pluginManager.cards.emotionEndpoint.fieldPluginUrl"),
            },
          ],
        },
      },
      {
        id: "complex-switch",
        title: t("pluginManager.cards.complexSwitch.title"),
        module: "complex_emotion" as const,
        moduleLabel: t("pluginTerms.module.complex_emotion"),
        type: "remote" as const,
        status: roleStore.roleInfo.remoteLifeEnabled ? "enabled" : "disabled",
        sourceKey: roleStore.roleInfo.remoteLifeEnabled ? "session_override" : "pack_default",
        sourceLabel: roleStore.roleInfo.remoteLifeEnabled
          ? t("pluginManager.cards.complexSwitch.sessionOn")
          : t("pluginManager.cards.complexSwitch.sessionOff"),
        description: t("pluginManager.cards.complexSwitch.description"),
        uiTemplate: "switch-toggle",
        schema: {
          checked: roleStore.roleInfo.remoteLifeEnabled,
          label: t("pluginManager.cards.complexSwitch.label"),
          hint: t("pluginManager.cards.complexSwitch.hint"),
        },
      },
      {
        id: "complex-endpoint",
        title: t("pluginManager.cards.complexEndpoint.title"),
        module: "complex_emotion" as const,
        moduleLabel: t("pluginTerms.module.complex_emotion"),
        type: "remote" as const,
        status: roleStore.roleInfo.remoteLifeEnabled ? "enabled" : "disabled",
        sourceKey: "env_override",
        sourceLabel: t("pluginManager.env.label"),
        description: t("pluginManager.cards.complexEndpoint.description"),
        uiTemplate: "endpoint-config",
        schema: {
          summary: t("pluginManager.cards.complexEndpoint.summary"),
          fields: [
            {
              name: "OCLIVE_COMPLEX_EMOTION_URL",
              description: t("pluginManager.cards.complexEndpoint.fieldUrl"),
            },
            {
              name: "OCLIVE_COMPLEX_EMOTION_TOKEN",
              description: t("pluginManager.cards.complexEndpoint.fieldToken"),
            },
          ],
        },
      },
    ];
  });

  const categories = computed<PluginV2CategoryItem[]>(() => {
    void locale.value;
    const rows = cards.value;
    const countBy = (fn: (x: PluginV2CardItem) => boolean) =>
      rows.filter((x) => fn(x)).length;
    return [
      { id: "all", label: t("pluginTerms.category.all"), count: rows.length },
      {
        id: "module:llm",
        label: t("pluginTerms.module.llm"),
        count: countBy((x) => x.module === "llm"),
      },
      {
        id: "module:emotion",
        label: t("pluginTerms.module.emotion"),
        count: countBy((x) => x.module === "emotion"),
      },
      {
        id: "module:complex_emotion",
        label: t("pluginTerms.module.complex_emotion"),
        count: countBy((x) => x.module === "complex_emotion"),
      },
      {
        id: "type:builtin",
        label: t("pluginTerms.type.builtin"),
        count: countBy((x) => x.type === "builtin"),
      },
      {
        id: "type:remote",
        label: t("pluginTerms.type.remote"),
        count: countBy((x) => x.type === "remote"),
      },
      {
        id: "type:directory",
        label: t("pluginTerms.type.directory"),
        count: countBy((x) => x.type === "directory"),
      },
      {
        id: "status:enabled",
        label: t("pluginTerms.status.enabled"),
        count: countBy((x) => x.status === "enabled"),
      },
      {
        id: "status:disabled",
        label: t("pluginTerms.status.disabled"),
        count: countBy((x) => x.status === "disabled"),
      },
      {
        id: "status:needs_config",
        label: t("pluginTerms.status.needs_config"),
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
      return t("pluginManager.apply.endpointNoSave");
    }
    if (item.uiTemplate === "switch-toggle") {
      const info = await setRemoteLifeEnabled(
        roleStore.currentRoleId,
        Boolean(payload.enabled),
      );
      roleStore.applyRoleInfo(info);
      return t("pluginManager.apply.remoteLifeUpdated");
    }

    const module = String((item.schema as { module?: string }).module ?? "");
    if (module !== "llm" && module !== "emotion") {
      throw new Error(t("pluginManager.apply.unsupported"));
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
    return t("pluginManager.apply.sessionSaved");
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
