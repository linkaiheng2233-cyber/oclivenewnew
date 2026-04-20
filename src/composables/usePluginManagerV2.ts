import { computed, ref, watch } from "vue";
import { usePluginStore } from "../stores/pluginStore";
import { useRoleStore } from "../stores/roleStore";
import { setRemoteLifeEnabled, setSessionPluginBackend } from "../utils/tauri-api";
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

function toSourceLabel(source: string): string {
  if (source === "session_override") return "会话覆盖";
  if (source === "env_override") return "环境覆盖";
  return "角色包默认";
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
        title: "对话回复引擎",
        module: "llm",
        moduleLabel: "对话大脑（LLM）",
        type: normalizeType(effective.llm),
        status: effective.llm === "directory" && !llmDirectoryId ? "needs_config" : "enabled",
        sourceLabel: toSourceLabel(sources.llm),
        description: "决定回复模型来源：本地模型、远程服务或目录插件。",
        uiTemplate: "slot-selector",
        schema: {
          module: "llm",
          current: roleStore.roleInfo.pluginBackendsSessionOverride?.llm ?? "__pack_default__",
          directoryId: llmDirectoryId,
          options: [
            { value: "__pack_default__", label: `跟随角色包默认（${defaults.llm}）` },
            { value: "ollama", label: "Ollama（本地模型）" },
            { value: "remote", label: "远程服务" },
            { value: "directory", label: "目录插件" },
          ],
          directoryOptions: directoryOptions.value,
        },
      },
      {
        id: "llm-endpoint",
        title: "LLM 远程地址说明",
        module: "llm",
        moduleLabel: "对话大脑（LLM）",
        type: "remote",
        status: effective.llm === "remote" ? "enabled" : "disabled",
        sourceLabel: "环境变量",
        description: "选择远程服务时，优先读取 LLM 专用地址。",
        uiTemplate: "endpoint-config",
        schema: {
          summary: "建议在系统环境变量配置地址，便于迁移与排错。",
          fields: [
            { name: "OCLIVE_REMOTE_LLM_URL", description: "LLM 专用远程地址（优先）" },
            { name: "OCLIVE_REMOTE_PLUGIN_URL", description: "通用远程地址（兜底）" },
          ],
        },
      },
      {
        id: "emotion-main",
        title: "情绪推理引擎",
        module: "emotion",
        moduleLabel: "情绪引擎（Emotion）",
        type: normalizeType(effective.emotion),
        status:
          effective.emotion === "directory" && !emotionDirectoryId
            ? "needs_config"
            : "enabled",
        sourceLabel: toSourceLabel(sources.emotion),
        description: "控制情绪由内置逻辑、远程服务或目录插件处理。",
        uiTemplate: "slot-selector",
        schema: {
          module: "emotion",
          current:
            roleStore.roleInfo.pluginBackendsSessionOverride?.emotion ?? "__pack_default__",
          directoryId: emotionDirectoryId,
          options: [
            { value: "__pack_default__", label: `跟随角色包默认（${defaults.emotion}）` },
            { value: "builtin", label: "内置" },
            { value: "builtin_v2", label: "内置 V2" },
            { value: "remote", label: "远程服务" },
            { value: "directory", label: "目录插件" },
          ],
          directoryOptions: directoryOptions.value,
        },
      },
      {
        id: "emotion-endpoint",
        title: "Emotion 远程地址说明",
        module: "emotion",
        moduleLabel: "情绪引擎（Emotion）",
        type: "remote",
        status: effective.emotion === "remote" ? "enabled" : "disabled",
        sourceLabel: "环境变量",
        description: "情绪 remote 默认读取通用远程地址。",
        uiTemplate: "endpoint-config",
        schema: {
          summary: "建议在系统环境变量配置地址，避免写死到角色包。",
          fields: [
            { name: "OCLIVE_REMOTE_PLUGIN_URL", description: "Emotion 常用远程入口" },
          ],
        },
      },
      {
        id: "complex-switch",
        title: "复杂情感开关",
        module: "complex_emotion",
        moduleLabel: "复杂情感（Complex Emotion）",
        type: "remote",
        status: roleStore.roleInfo.remoteLifeEnabled ? "enabled" : "disabled",
        sourceLabel: roleStore.roleInfo.remoteLifeEnabled
          ? "当前会话已开启"
          : "当前会话已关闭",
        description: "开启后启用异地心声链路，复杂情感表现更明显。",
        uiTemplate: "switch-toggle",
        schema: {
          checked: roleStore.roleInfo.remoteLifeEnabled,
          label: "启用复杂情感（异地心声）",
          hint: "开启后建议配置 URL 与 TOKEN 环境变量。",
        },
      },
      {
        id: "complex-endpoint",
        title: "复杂情感地址说明",
        module: "complex_emotion",
        moduleLabel: "复杂情感（Complex Emotion）",
        type: "remote",
        status: roleStore.roleInfo.remoteLifeEnabled ? "enabled" : "disabled",
        sourceLabel: "环境变量",
        description: "复杂情感服务通常独立部署，支持鉴权 token。",
        uiTemplate: "endpoint-config",
        schema: {
          summary: "若服务要求鉴权，请同时配置 URL 和 TOKEN。",
          fields: [
            { name: "OCLIVE_COMPLEX_EMOTION_URL", description: "复杂情感服务地址" },
            { name: "OCLIVE_COMPLEX_EMOTION_TOKEN", description: "复杂情感服务鉴权 Token" },
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
      { id: "all", label: "全部功能", count: rows.length },
      {
        id: "module:llm",
        label: "对话大脑（LLM）",
        count: countBy((x) => x.module === "llm"),
      },
      {
        id: "module:emotion",
        label: "情绪引擎（Emotion）",
        count: countBy((x) => x.module === "emotion"),
      },
      {
        id: "module:complex_emotion",
        label: "复杂情感（Complex Emotion）",
        count: countBy((x) => x.module === "complex_emotion"),
      },
      { id: "type:builtin", label: "内置", count: countBy((x) => x.type === "builtin") },
      { id: "type:remote", label: "远程", count: countBy((x) => x.type === "remote") },
      {
        id: "type:directory",
        label: "本地目录插件",
        count: countBy((x) => x.type === "directory"),
      },
      {
        id: "status:enabled",
        label: "已启用",
        count: countBy((x) => x.status === "enabled"),
      },
      {
        id: "status:disabled",
        label: "已关闭",
        count: countBy((x) => x.status === "disabled"),
      },
      {
        id: "status:needs_config",
        label: "还需配置",
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
      return "地址说明项无需保存，请在环境变量中配置。";
    }
    if (item.uiTemplate === "switch-toggle") {
      const info = await setRemoteLifeEnabled(
        roleStore.currentRoleId,
        Boolean(payload.enabled),
      );
      roleStore.applyRoleInfo(info);
      return "复杂情感开关已更新。";
    }

    const module = String((item.schema as { module?: string }).module ?? "");
    if (module !== "llm" && module !== "emotion") {
      throw new Error("当前仅支持 LLM / Emotion 配置写入。");
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
    return "配置已写入当前会话。";
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
