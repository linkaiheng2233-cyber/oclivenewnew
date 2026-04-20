import { invoke } from "@tauri-apps/api/tauri";

export const TransactionErrorMessages: Record<string, string> = {
  TXN_BEGIN_FAILED: "事务启动失败，请稍后重试。",
  TXN_RUNTIME_ENSURE_FAILED: "角色运行时状态初始化失败。",
  TXN_PERSONALITY_INSERT_FAILED: "性格数据写入失败。",
  TXN_FAVORABILITY_UPDATE_FAILED: "好感度更新失败。",
  TXN_FAVORABILITY_HISTORY_INSERT_FAILED: "好感度历史记录失败。",
  TXN_MEMORY_INSERT_FAILED: "记忆数据保存失败。",
  TXN_SHORT_TERM_INSERT_FAILED: "对话记录写入失败。",
  TXN_SHORT_TERM_TRIM_FAILED: "对话记录整理失败。",
  TXN_EVENT_INSERT_FAILED: "事件写入失败。",
  TXN_FAVORABILITY_READ_FAILED: "好感度读取失败。",
  TXN_COMMIT_FAILED: "事务提交失败，请稍后再试。",
  TXN_ROLLBACK_FAILED: "事务回滚异常，请联系技术支持。",
};

const CommonErrorMessages: Record<string, string> = {
  DB_ERROR: "数据库操作失败，请稍后重试。",
  IO_ERROR: "本地文件读写失败，请检查环境权限。",
  API_PLUGIN_NOT_FOUND: "未找到该目录插件或插件未扫描到，请检查插件 id 与安装路径。",
  API_PERMISSION_DENIED: "插件权限不足，请在 manifest 中声明所需权限。",
  API_INVALID_MANIFEST: "插件 manifest 无效，请检查 manifest.json。",
  LLM_ERROR:
    "模型调用失败（常见：Ollama 未启动、模型未下载或名称不对）。请执行 ollama list，并设置环境变量 OLLAMA_MODEL 为已有模型名；默认 qwen2.5:7b。",
  ROLE_NOT_FOUND: "角色不存在，请确认 role_id。",
  ROLE_PACK_EXISTS: "该角色 ID 已存在。若要替换本地版本，请选择覆盖。",
  INVALID_PARAMETER: "参数无效，请检查输入内容。",
  OLLAMA_TIMEOUT: "沐沐走神了，再问一次吧。",
  TXN_ROLLBACK: "操作失败，请稍后再试。",
  SERDE_ERROR: "数据解析失败，请稍后重试。",
  UNKNOWN_ERROR: "发生未知错误，请稍后重试。",
};

function parseBackendError(err: unknown): { code?: string; raw: string } {
  const raw = String(err ?? "");
  const match = raw.match(/\[([A-Z0-9_]+)\]/);
  return { code: match?.[1], raw };
}

export interface FriendlyError {
  code?: string;
  message: string;
  raw: string;
}

type ErrorReporter = (err: FriendlyError) => void;

let errorReporter: ErrorReporter | null = null;

export function setErrorReporter(reporter: ErrorReporter | null): void {
  errorReporter = reporter;
}

/** 从 `invoke` 抛出的字符串中解析 `[CODE]`（与 Rust `ApiError` 等一致）。 */
export function parseApiErrorCode(err: unknown): string | undefined {
  return parseBackendError(err).code;
}

/** 是否为目录插件未找到类错误（便于 UI 分支）。 */
export function isPluginNotFoundError(err: unknown): boolean {
  return parseApiErrorCode(err) === "API_PLUGIN_NOT_FOUND";
}

export function isPermissionDeniedError(err: unknown): boolean {
  return parseApiErrorCode(err) === "API_PERMISSION_DENIED";
}

export function isInvalidParameterError(err: unknown): boolean {
  return parseApiErrorCode(err) === "INVALID_PARAMETER";
}

export function toFriendlyErrorMessage(err: unknown): string {
  const { code, raw } = parseBackendError(err);
  if (!code) return raw;
  if (code === "INVALID_PARAMETER" && raw.includes("角色包格式错误")) {
    const bracket = raw.indexOf("]");
    if (bracket !== -1) return raw.slice(bracket + 1).trim();
  }
  if (code === "ROLE_NOT_FOUND") {
    const bracket = raw.indexOf("]");
    if (bracket !== -1) {
      const detail = raw.slice(bracket + 1).trim();
      if (detail.startsWith("Role not found:")) {
        return `角色不存在或找不到 manifest。${detail.slice("Role not found:".length).trim()}`;
      }
    }
  }
  if (code === "IO_ERROR" && raw.includes("host json")) {
    return "插件桥返回的数据无法序列化为 JSON，可能是宿主与插件接口不兼容，请查看控制台日志。";
  }
  return TransactionErrorMessages[code] ?? CommonErrorMessages[code] ?? raw;
}

export function toFriendlyError(err: unknown): FriendlyError {
  const { code, raw } = parseBackendError(err);
  return {
    code,
    raw,
    message: toFriendlyErrorMessage(err),
  };
}

async function invokeWithFriendlyError<T>(
  command: string,
  payload: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, payload);
  } catch (err) {
    const friendly = toFriendlyError(err);
    // 友好文案会盖住后端细节；开发排查时看控制台完整 raw
    console.error(`[tauri:${command}]`, friendly.code ?? "?", friendly.raw);
    if (errorReporter) {
      errorReporter(friendly);
    } else if (friendly.code) {
      console.warn(`[api-error] code=${friendly.code} msg=${friendly.message}`);
    }
    throw new Error(friendly.message);
  }
}

export interface SendMessageRequest {
  role_id: string;
  user_message: string;
  scene_id?: string | null;
}

export interface EmotionDto {
  joy: number;
  sadness: number;
  anger: number;
  fear: number;
  surprise: number;
  disgust: number;
  neutral: number;
}

export interface DetectedEventDto {
  event_type: string;
  confidence: number;
}

export type PresenceMode = "co_present" | "remote_stub" | "remote_life";

export interface SendMessageResponse {
  api_version: number;
  schema: number;
  /** 共景 / 异地占位 / 异地心声 */
  presence_mode: PresenceMode;
  /** 本回合结束后的关系阶段（与 `role_runtime.relation_state` 一致） */
  relation_state: string;
  reply: string;
  emotion: EmotionDto;
  /** 本回合 bot 情绪标签（小写英文） */
  bot_emotion: string;
  /** 立绘用（与 DB current_emotion 一致）；对话语气见 bot_emotion */
  portrait_emotion: string;
  favorability_delta: number;
  favorability_current: number;
  events: DetectedEventDto[];
  scene_id: string;
  /** 后端判定用户有前往/位移意图时置 true；实际切换仅通过 switch_scene */
  offer_destination_picker: boolean;
  /** 检测到「一起去/跟我来」等邀请同行语义时置 true；确认后 `switch_scene`（同行）或仅叙事切换 */
  offer_together_travel: boolean;
  /** 主 LLM 失败时使用备用短回复 */
  reply_is_fallback?: boolean;
  /** 本回合注入 Prompt 的知识块条数（共景/异地心声；占位为 0） */
  knowledge_chunks_in_prompt?: number;
  timestamp: number;
}

/** 身份下拉里「跟随 manifest 默认身份」选项的值（与后端 `OCLIVE_DEFAULT_RELATION_SENTINEL` 一致） */
export const OCLIVE_DEFAULT_RELATION_SENTINEL = "__oclive_default__";

export interface UserRelationDto {
  id: string;
  name: string;
  prompt_hint: string;
  favor_multiplier: number;
  /** 角色包配置的初始好感度（0～100）；切换顶栏身份时会同步到当前好感 */
  initial_favorability: number;
}

/** 虚拟时间 + manifest `life_schedule` 推断的当前活动 */
export interface LifeStateDto {
  label: string;
  activity_key: string;
  busy_level: number;
  preferred_scene_id: string | null;
}

/** 与 `plugin_backends.directory_plugins` 一致（snake_case JSON 字段） */
export interface DirectoryPluginSlots {
  memory?: string | null;
  emotion?: string | null;
  event?: string | null;
  prompt?: string | null;
  llm?: string | null;
}

/** 与 `settings.json` → `plugin_backends` 一致（snake_case，与后端 serde 对齐） */
export interface PluginBackends {
  memory: "builtin" | "builtin_v2" | "remote" | "local" | "directory";
  /** `memory === "local"` 时可选：与 `_local_plugins` 中 descriptor 的 `provider_id` 一致 */
  local_memory_provider_id?: string | null;
  emotion: "builtin" | "builtin_v2" | "remote" | "directory";
  event: "builtin" | "builtin_v2" | "remote" | "directory";
  prompt: "builtin" | "builtin_v2" | "remote" | "directory";
  llm: "ollama" | "remote" | "directory";
  /** 各模块为 `directory` 时对应的 manifest `id`（见 DIRECTORY_PLUGINS.md） */
  directory_plugins?: DirectoryPluginSlots;
}

export interface PluginBackendsOverride {
  memory?: PluginBackends["memory"] | null;
  local_memory_provider_id?: string | null;
  emotion?: PluginBackends["emotion"] | null;
  event?: PluginBackends["event"] | null;
  prompt?: PluginBackends["prompt"] | null;
  llm?: PluginBackends["llm"] | null;
  /** 会话级与包内按槽合并（当前 UI 未编辑；仅展示与调试） */
  directory_plugins?: DirectoryPluginSlots | null;
}

export type PluginBackendSource = "pack_default" | "session_override" | "env_override";

export interface PluginBackendsSourceMap {
  memory: PluginBackendSource;
  emotion: PluginBackendSource;
  event: PluginBackendSource;
  prompt: PluginBackendSource;
  llm: PluginBackendSource;
}

export interface PluginResolutionDebugInfo {
  app_version: string;
  api_version: number;
  schema_version: number;
  role_id: string;
  session_namespace: string;
  plugin_backends_pack_default: PluginBackends;
  plugin_backends_session_override?: PluginBackendsOverride | null;
  plugin_backends_effective: PluginBackends;
  plugin_backends_effective_sources: PluginBackendsSourceMap;
  llm_env_override?: string | null;
  remote_plugin_url_configured: boolean;
  remote_llm_url_configured: boolean;
  local_provider_ids: string[];
  local_provider_count: number;
}

/**
 * `load_role` 返回的扁平快照。
 * 身份相关：`default_relation` 来自角色包；`current_user_relation` 为解析后的有效键（`identity_binding: per_scene` 时场景覆盖优先，否则为全局 manifest 默认或 DB）；
 * `use_manifest_default` 仅表示用户是否选了「默认身份」选项；好感/阶段与当前有效身份一致。
 */
/** `evolution.personality_source` */
export type PersonalitySource = "vector" | "profile";

/** 角色包根目录 `ui.json`（与后端 `UiConfig` 对齐；插槽键含点号） */
export interface PackUiSlotConfig {
  order: string[];
  visible: string[];
}

export interface PackUiSlots {
  chat_toolbar: PackUiSlotConfig;
  "settings.panel": PackUiSlotConfig;
  "role.detail": PackUiSlotConfig;
  sidebar: PackUiSlotConfig;
  "chat.header": PackUiSlotConfig;
}

export interface PackUiTheme {
  primaryColor?: string;
  backgroundColor?: string;
  fontFamily?: string;
}

export interface PackUiLayout {
  sidebar?: string;
  chatInput?: string;
}

export interface PackUiConfig {
  shell: string;
  theme: PackUiTheme;
  layout: PackUiLayout;
  slots: PackUiSlots;
}

export function emptyPackUiConfig(): PackUiConfig {
  return {
    shell: "",
    theme: { primaryColor: "", backgroundColor: "", fontFamily: "" },
    layout: { sidebar: "", chatInput: "" },
    slots: {
      chat_toolbar: { order: [], visible: [] },
      "settings.panel": { order: [], visible: [] },
      "role.detail": { order: [], visible: [] },
      sidebar: { order: [], visible: [] },
      "chat.header": { order: [], visible: [] },
    },
  };
}

/** 与后端 `models::author_pack::AuthorPackFile` 对齐（snake_case 字段）。 */
export interface AuthorRecommendedPlugin {
  id: string;
  version_range?: string | null;
  slots?: string[];
  for_backends?: string[];
  optional?: boolean;
  note?: string | null;
}

export interface AuthorPackFile {
  schema_version: number;
  summary?: string;
  detail_markdown?: string;
  recommended_plugins?: AuthorRecommendedPlugin[];
  suggested_ui?: PackUiConfig | null;
  suggested_plugin_backends?: PluginBackends | null;
}

export function normalizePackUiConfig(
  raw: PackUiConfig | undefined | null,
): PackUiConfig {
  const e = emptyPackUiConfig();
  if (!raw) return e;
  const slots = raw.slots;
  const slot = (k: keyof PackUiSlots): PackUiSlotConfig => {
    const s = slots?.[k];
    return {
      order: Array.isArray(s?.order) ? s!.order.map(String) : [],
      visible: Array.isArray(s?.visible) ? s!.visible.map(String) : [],
    };
  };
  return {
    shell: typeof raw.shell === "string" ? raw.shell : e.shell,
    theme: {
      primaryColor: raw.theme?.primaryColor?.trim() ?? "",
      backgroundColor: raw.theme?.backgroundColor?.trim() ?? "",
      fontFamily: raw.theme?.fontFamily?.trim() ?? "",
    },
    layout: {
      sidebar: (raw.layout?.sidebar ?? "").trim().toLowerCase(),
      chatInput: (raw.layout?.chatInput ?? "").trim().toLowerCase(),
    },
    slots: {
      chat_toolbar: slot("chat_toolbar"),
      "settings.panel": slot("settings.panel"),
      "role.detail": slot("role.detail"),
      sidebar: slot("sidebar"),
      "chat.header": slot("chat.header"),
    },
  };
}

export interface RoleData {
  role_id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  personality_vector: number[];
  current_favorability: number;
  current_emotion: string;
  memory_count: number;
  event_count: number;
  user_relations: UserRelationDto[];
  default_relation: string;
  relation_state: string;
  current_user_relation: string;
  /** 是否选中「默认身份」（跟随 manifest `default_relation`） */
  use_manifest_default: boolean;
  /** 异地心声开关（DB） */
  remote_life_enabled: boolean;
  /** 角色包建议默认是否开启异地心声（settings.json → remote_presence.default_enabled） */
  remote_life_pack_default: boolean | null;
  event_impact_factor: number;
  /** `evolution.personality_source`；缺省为 vector */
  personality_source?: PersonalitySource;
  /** manifest → OLLAMA_MODEL → 全局默认 */
  effective_ollama_model: string;
  /** 全局单一身份 vs 按场景覆盖（manifest `identity_binding`） */
  identity_binding: "global" | "per_scene";
  /** 当前交互模式（DB）：`immersive` | `pure_chat` */
  interaction_mode: "immersive" | "pure_chat";
  /** 角色包 settings.json 建议默认（可选） */
  interaction_mode_pack_default: "immersive" | "pure_chat" | null;
  /** 当前日程推断（无配置或未命中时段时为 null） */
  current_life: LifeStateDto | null;
  /** 模块化子系统后端（与 `PluginHost` 解析一致） */
  plugin_backends: PluginBackends;
  /** 当前会话覆盖（无覆盖时为 null） */
  plugin_backends_session_override?: PluginBackendsOverride | null;
  /** 叠加会话覆盖后的有效后端 */
  plugin_backends_effective?: PluginBackends;
  /** 叠加后的后端来源（pack/session/env） */
  plugin_backends_effective_sources?: PluginBackendsSourceMap;
  /** 角色包 `ui.json`（主题、布局、插槽） */
  pack_ui_config: PackUiConfig;
  /** `author.suggested_ui` 优先时的有效 UI 基线（与后端 `pack_ui_baseline` 一致） */
  pack_ui_baseline?: PackUiConfig;
  /** 可选 `author.json` */
  author_pack?: AuthorPackFile | null;
}

export interface SceneLabelEntry {
  id: string;
  label: string;
}

/** `get_role_info` / `switch_scene` 等与 UI 同步的快照；身份字段语义同 {@link RoleData}。 */
export interface RoleInfo {
  role_id: string;
  role_name: string;
  version: string;
  author: string;
  description: string;
  current_favorability: number;
  current_emotion: string;
  personality_vector: number[];
  /** `evolution.personality_source`；缺省为 vector */
  personality_source?: PersonalitySource;
  last_interaction?: string | null;
  scenes: string[];
  /** 与 scenes 顺序一致；label 来自角色包 scene.json 或内置映射 */
  scene_labels: SceneLabelEntry[];
  current_scene: string | null;
  /** 用户叙事场景（DB）；与 current_scene 可不同 */
  user_presence_scene: string | null;
  virtual_time_ms: number;
  user_relations: UserRelationDto[];
  default_relation: string;
  current_user_relation: string;
  /** 是否选中「默认身份」（下拉应显示 `OCLIVE_DEFAULT_RELATION_SENTINEL`） */
  use_manifest_default: boolean;
  /** 关系阶段（`role_runtime.relation_state`） */
  relation_state: string;
  remote_life_enabled: boolean;
  remote_life_pack_default: boolean | null;
  event_impact_factor: number;
  /** manifest → OLLAMA_MODEL → 全局默认 */
  effective_ollama_model: string;
  /** 全局单一身份 vs 按场景覆盖（manifest `identity_binding`） */
  identity_binding: "global" | "per_scene";
  interaction_mode: "immersive" | "pure_chat";
  interaction_mode_pack_default: "immersive" | "pure_chat" | null;
  /** 当前日程推断（无配置或未命中时段时为 null） */
  current_life: LifeStateDto | null;
  /** 模块化子系统后端（与 `load_role` 一致） */
  plugin_backends: PluginBackends;
  /** 当前会话覆盖（无覆盖时为 null） */
  plugin_backends_session_override?: PluginBackendsOverride | null;
  /** 叠加会话覆盖后的有效后端 */
  plugin_backends_effective?: PluginBackends;
  /** 叠加后的后端来源（pack/session/env） */
  plugin_backends_effective_sources?: PluginBackendsSourceMap;
  /** 是否已从磁盘构建世界观知识索引 */
  knowledge_enabled?: boolean;
  /** 知识块条数；未加载索引时为 0 */
  knowledge_chunk_count?: number;
  /** 角色包 `ui.json`（主题、布局、插槽） */
  pack_ui_config: PackUiConfig;
  pack_ui_baseline?: PackUiConfig;
  author_pack?: AuthorPackFile | null;
}

/** `switch_scene` 扁平化返回：RoleInfo 字段 + 可选场景欢迎语 */
export type SwitchSceneResponse = RoleInfo & {
  scene_welcome?: string | null;
};

export interface TimeStateResponse {
  virtual_time_ms: number;
  iso_datetime: string;
}

export interface JumpTimeResponse {
  virtual_time_ms: number;
  iso_datetime: string;
  monologues: string[];
  favorability_delta: number;
  favorability_current: number;
  /** 虚拟时间规则是否将角色 current_scene 从 from 切到 to */
  autonomous_scene_from?: string | null;
  autonomous_scene_to?: string | null;
}

export interface ExportChatLogsResponse {
  content: string;
  suggested_filename: string;
}

export interface QueryMemoriesRequest {
  role_id: string;
  limit: number;
  offset: number;
}

export interface MemoryItem {
  id: string;
  role_id: string;
  content: string;
  memory_type: string;
  timestamp: string;
  importance: number;
}

export interface QueryEventsRequest {
  role_id: string;
  limit: number;
  offset: number;
}

export interface EventItem {
  id: number;
  role_id: string;
  event_type: string;
  user_emotion?: string | null;
  bot_emotion?: string | null;
  timestamp: string;
  description?: string | null;
}

export interface CreateEventRequest {
  role_id: string;
  event_type: string;
  description?: string | null;
}

export interface CreateEventResponse {
  id: number;
  role_id: string;
  event_type: string;
  timestamp: string;
  description?: string | null;
}

export async function sendMessage(
  req: SendMessageRequest,
): Promise<SendMessageResponse> {
  return invokeWithFriendlyError<SendMessageResponse>("send_message", { req });
}

export async function loadRole(roleId: string): Promise<RoleData> {
  return invokeWithFriendlyError<RoleData>("load_role", { roleId });
}

/** 若文件存在则返回绝对路径，否则 `null`（不抛错）。 */
export async function resolveRoleAssetPath(
  roleId: string,
  relative: string,
): Promise<string | null> {
  return invoke<string | null>("resolve_role_asset_path", {
    roleId,
    relative,
  });
}

/** `sessionId` 与发消息的会话 id 一致时，返回该命名空间下的 `plugin_backends_*` 等快照。 */
export async function getRoleInfo(
  roleId: string,
  sessionId?: string | null,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>("get_role_info", {
    req: {
      role_id: roleId,
      session_id: sessionId ?? null,
    },
  });
}

export async function queryMemories(
  req: QueryMemoriesRequest,
): Promise<MemoryItem[]> {
  return invokeWithFriendlyError<MemoryItem[]>("query_memories", { req });
}

export async function queryEvents(req: QueryEventsRequest): Promise<EventItem[]> {
  return invokeWithFriendlyError<EventItem[]>("query_events", { req });
}

export async function createEvent(
  req: CreateEventRequest,
): Promise<CreateEventResponse> {
  return invokeWithFriendlyError<CreateEventResponse>("create_event", { req });
}

export async function reloadPolicyPlugins(): Promise<string> {
  return invokeWithFriendlyError<string>("reload_policy_plugins", {});
}

export async function listRoles(): Promise<Array<{ id: string; name: string }>> {
  const rows = await invokeWithFriendlyError<
    Array<{ id: string; name: string; version: string; author: string }>
  >("list_roles", {});
  return rows.map((r) => ({ id: r.id, name: r.name }));
}

export async function switchRole(roleId: string): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>("switch_role", { roleId });
}

export async function setUserRelation(
  roleId: string,
  relation: string,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>("set_user_relation", {
    req: { role_id: roleId, relation },
  });
}

export async function setEvolutionFactor(
  roleId: string,
  eventImpactFactor: number,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>("set_evolution_factor", {
    req: { role_id: roleId, event_impact_factor: eventImpactFactor },
  });
}

export async function setRemoteLifeEnabled(
  roleId: string,
  enabled: boolean,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>("set_remote_life_enabled", {
    req: { role_id: roleId, enabled },
  });
}

export async function setRoleInteractionMode(
  roleId: string,
  mode: "immersive" | "pure_chat",
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>("set_role_interaction_mode", {
    req: { role_id: roleId, mode },
  });
}

export async function setSessionPluginBackend(
  roleId: string,
  module: "memory" | "emotion" | "event" | "prompt" | "llm",
  /** 与后端 `parse_backend_wire` 一致，如 `builtin_v2`、`directory`、`remote` */
  backend?: string | null,
  localMemoryProviderId?: string,
  sessionId?: string | null,
  directoryId?: string | null,
): Promise<RoleInfo> {
  const req: Record<string, unknown> = {
    role_id: roleId,
    module,
    session_id: sessionId ?? null,
  };
  if (backend !== undefined) {
    req.backend = backend;
  }
  if (localMemoryProviderId !== undefined) {
    req.local_memory_provider_id = localMemoryProviderId;
  }
  if (directoryId !== undefined) {
    req.directory_id = directoryId;
  }
  return invokeWithFriendlyError<RoleInfo>("set_session_plugin_backend", {
    req,
  });
}

/** 将 `author.json` → `suggested_plugin_backends` 写入当前会话后端覆盖。 */
export async function applyAuthorSuggestedPluginBackends(
  roleId: string,
  sessionId?: string | null,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>("apply_author_suggested_plugin_backends", {
    req: {
      role_id: roleId,
      session_id: sessionId ?? null,
    },
  });
}

export async function getPluginResolutionDebug(
  roleId: string,
  sessionId?: string | null,
): Promise<PluginResolutionDebugInfo> {
  return invokeWithFriendlyError<PluginResolutionDebugInfo>(
    "get_plugin_resolution_debug",
    {
      req: {
        role_id: roleId,
        session_id: sessionId ?? null,
      },
    },
  );
}

export async function switchScene(
  roleId: string,
  sceneId: string,
  /** `true`：角色与用户同场景；`false`：仅更新用户叙事场景 */
  together: boolean = true,
): Promise<SwitchSceneResponse> {
  return invokeWithFriendlyError<SwitchSceneResponse>("switch_scene", {
    req: { role_id: roleId, scene_id: sceneId, together },
  });
}

export async function setUserPresenceScene(
  roleId: string,
  sceneId: string,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>("set_user_presence_scene", {
    req: { role_id: roleId, scene_id: sceneId },
  });
}

export async function getTimeState(roleId: string): Promise<TimeStateResponse> {
  return invokeWithFriendlyError<TimeStateResponse>("get_time_state", {
    roleId,
  });
}

export async function jumpTime(
  roleId: string,
  timestampMs?: number,
  preset?: "+2h" | "+6h" | "next_morning" | "skip_idle_time",
): Promise<JumpTimeResponse> {
  return invokeWithFriendlyError<JumpTimeResponse>("jump_time", {
    req: { role_id: roleId, timestamp_ms: timestampMs ?? null, preset: preset ?? null },
  });
}

export async function setSceneUserRelation(
  roleId: string,
  sceneId: string,
  relation: string,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>("set_scene_user_relation", {
    req: { role_id: roleId, scene_id: sceneId, relation },
  });
}

/** 移除当前场景的身份覆盖，再与 `set_user_relation` 配合可恢复「默认身份」全局状态 */
export async function clearSceneUserRelation(
  roleId: string,
  sceneId: string,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>("clear_scene_user_relation", {
    req: { role_id: roleId, scene_id: sceneId },
  });
}

export async function generateMonologue(roleId: string): Promise<string> {
  const res = await invokeWithFriendlyError<{ text: string }>(
    "generate_monologue",
    { req: { role_id: roleId } },
  );
  return res.text;
}

/** `.ocpak`：ZIP 打包的 `roles/{id}/` 目录（与 `.zip` 相同容器；亦可导入已解压目录路径） */
export async function exportRolePack(
  roleId: string,
  destPath: string,
): Promise<void> {
  return invokeWithFriendlyError<void>("export_role_pack_command", {
    role_id: roleId,
    dest_path: destPath,
  });
}

export interface RolePackPeek {
  id: string;
  name: string;
  version: string;
}

/** 预览角色包：`srcPath` 可为 `.ocpak` / `.zip` 或已解压目录（与 `roles/{id}/` 一致）。 */
export async function peekRolePack(srcPath: string): Promise<RolePackPeek> {
  return invokeWithFriendlyError<RolePackPeek>("peek_role_pack_command", {
    src_path: srcPath,
  });
}

/** 导入角色包：同上，支持压缩包或已解压目录。 */
export async function importRolePack(
  srcPath: string,
  overwrite: boolean,
): Promise<string> {
  return invokeWithFriendlyError<string>("import_role_pack_command", {
    src_path: srcPath,
    overwrite,
  });
}

export async function exportChatLogs(params: {
  roleId?: string;
  allRoles?: boolean;
  format: "json" | "txt";
  includePluginResolutionDebug?: boolean;
  sessionId?: string | null;
}): Promise<ExportChatLogsResponse> {
  return invokeWithFriendlyError<ExportChatLogsResponse>("export_chat_logs", {
    req: {
      role_id: params.roleId ?? null,
      all_roles: params.allRoles ?? false,
      format: params.format,
      include_plugin_resolution_debug: params.includePluginResolutionDebug ?? false,
      session_id: params.sessionId ?? null,
    },
  });
}

/** 嵌入主界面插槽（`chat_toolbar` / `settings.panel`），由 bootstrap 返回。 */
export interface PluginUiSlotInfo {
  pluginId: string;
  slot: string;
  /** manifest `ui_slots[].appearance_id`；空字符串为默认变体 */
  appearanceId?: string;
  /** manifest `ui_slots[].label` */
  label?: string | null;
  /** manifest `ui_slots[].entry`（相对插件根） */
  entry: string;
  /** manifest `vueComponent`；存在时优先原生 Vue，失败则回退 `url` iframe */
  vueComponent?: string | null;
  url: string;
}

/** 读取目录插件根下文本文件（宿主编译 `.vue` 等）。 */
export async function readPluginAssetText(
  pluginId: string,
  rel: string,
): Promise<string> {
  return invokeWithFriendlyError<string>("read_plugin_asset_text", {
    pluginId,
    rel,
  });
}

/** 目录插件启动引导（整壳 URL、已扫描插件 id、开发者模式、UI 插槽）。 */
export interface DirectoryPluginBootstrap {
  shellUrl?: string | null;
  shellPluginId?: string | null;
  /** 整壳 `manifest.shell.vueEntry`（相对插件根）；与 `forceIframeMode` 决定是否走宿主 Vue 整壳。 */
  shellVueEntry?: string | null;
  /** 与 `plugin_state.force_iframe_mode` 一致；为真时忽略 Vue 整壳与插槽 Vue。 */
  forceIframeMode?: boolean;
  pluginIds: string[];
  developerMode: boolean;
  /** 当前角色下已启用插件在 manifest `bridge.events` 中声明的宿主事件名。 */
  subscribedHostEvents: string[];
  uiSlots: PluginUiSlotInfo[];
}

/** `check_plugin_updates` 单插件结果（在线检查预留）。 */
export interface PluginUpdateInfo {
  hasUpdate: boolean;
  latestVersion?: string | null;
  message?: string | null;
}

export async function checkPluginUpdates(
  pluginIds: string[],
): Promise<Record<string, PluginUpdateInfo>> {
  return invokeWithFriendlyError<Record<string, PluginUpdateInfo>>(
    "check_plugin_updates",
    { plugin_ids: pluginIds },
  );
}

export async function extractPluginZip(
  zipPath: string,
  pluginId: string,
): Promise<void> {
  return invokeWithFriendlyError<void>("extract_plugin_zip", {
    zip_path: zipPath,
    plugin_id: pluginId,
  });
}

/** 同一 `role_id` 上并发的 bootstrap 合并为单次 IPC，避免多插槽同时挂载时重复打后端。 */
const directoryBootstrapInflight = new Map<
  string,
  Promise<DirectoryPluginBootstrap>
>();

function directoryBootstrapCacheKey(roleId?: string | null): string {
  const t = (roleId ?? "").trim();
  return t.length > 0 ? t : "__default__";
}

export async function getDirectoryPluginBootstrap(
  roleId?: string | null,
): Promise<DirectoryPluginBootstrap> {
  const key = directoryBootstrapCacheKey(roleId);
  const existing = directoryBootstrapInflight.get(key);
  if (existing) {
    return existing;
  }
  const p = invokeWithFriendlyError<DirectoryPluginBootstrap>(
    "get_directory_plugin_bootstrap",
    { role_id: roleId ?? null },
  ).finally(() => {
    if (directoryBootstrapInflight.get(key) === p) {
      directoryBootstrapInflight.delete(key);
    }
  });
  directoryBootstrapInflight.set(key, p);
  return p;
}

/** 与 `app_data/plugin_state.json` 中单角色 slots 段一致（snake_case）。 */
export interface PluginStateFile {
  disabled_plugins: string[];
  slot_order: Record<string, string[]>;
  disabled_slot_contributions: Record<string, string[]>;
  /** `plugin_id` → `slot` → `appearance_id` */
  slot_appearance?: Record<string, Record<string, string>>;
  /** 为真时忽略 `vueComponent`，嵌入插槽仅用 iframe。 */
  force_iframe_mode?: boolean;
}

/** 单角色的目录插件 UI 状态（含整壳 id，与后端 `RolePluginStateDto` 一致）。 */
export interface RolePluginState extends PluginStateFile {
  shellPluginId: string;
}

export interface PluginStateGetResponse {
  role: RolePluginState;
  /** 后端 `serde(rename_all = "camelCase")` → `globalDefaults` */
  globalDefaults: RolePluginState;
}

/** 并发 `get_plugin_state(role_id)` 合并为单次 IPC（按 role_id 维度）。 */
const pluginStateInflight = new Map<string, Promise<PluginStateGetResponse>>();

function pluginStateCacheKey(roleId: string): string {
  const t = roleId.trim();
  return t.length > 0 ? t : "__default__";
}

/** 角色包根目录 `ui.json`（与编写器 / 后端 `UiConfig` 一致）。 */
export interface SlotConfig {
  order: string[];
  visible: string[];
  /** 插件 id → 默认 `appearance_id`（该槽内） */
  appearance?: Record<string, string>;
}

export interface UiConfig {
  shell: string;
  slots: {
    chat_toolbar: SlotConfig;
    "settings.panel": SlotConfig;
    "role.detail": SlotConfig;
    sidebar: SlotConfig;
    "chat.header": SlotConfig;
    "settings.plugins": SlotConfig;
    "settings.advanced": SlotConfig;
    "overlay.floating": SlotConfig;
    "launcher.palette": SlotConfig;
    "debug.dock": SlotConfig;
  };
}

export interface UiSlotVariantInfo {
  slot: string;
  appearanceId: string;
  label?: string | null;
}

export interface DirectoryPluginCatalogEntry {
  id: string;
  version: string;
  pluginType?: string | null;
  /** manifest 是否含 `process`（可调试 RPC 子进程） */
  hasRpcProcess: boolean;
  isShell: boolean;
  uiSlotNames: string[];
  /** 每条 manifest `ui_slots`（嵌入槽）一条 */
  uiSlotVariants?: UiSlotVariantInfo[];
  provides: string[];
  /** `ok` / `missing` / `mismatch` */
  dependencyStatus: string;
  dependencyIssues: string[];
}

/** 并发 `get_directory_plugin_catalog` 合并为单次 IPC（无 role 参数，全局共用一个 in-flight）。 */
const directoryCatalogInflight = new Map<
  string,
  Promise<DirectoryPluginCatalogEntry[]>
>();
const DIRECTORY_CATALOG_COALESCE_KEY = "__global__";

export async function getDirectoryPluginCatalog(): Promise<DirectoryPluginCatalogEntry[]> {
  const existing = directoryCatalogInflight.get(DIRECTORY_CATALOG_COALESCE_KEY);
  if (existing) {
    return existing;
  }
  const p = invokeWithFriendlyError<DirectoryPluginCatalogEntry[]>(
    "get_directory_plugin_catalog",
    {},
  ).finally(() => {
    if (directoryCatalogInflight.get(DIRECTORY_CATALOG_COALESCE_KEY) === p) {
      directoryCatalogInflight.delete(DIRECTORY_CATALOG_COALESCE_KEY);
    }
  });
  directoryCatalogInflight.set(DIRECTORY_CATALOG_COALESCE_KEY, p);
  return p;
}

export async function getPluginState(
  roleId: string,
): Promise<PluginStateGetResponse> {
  const key = pluginStateCacheKey(roleId);
  const existing = pluginStateInflight.get(key);
  if (existing) {
    return existing;
  }
  const p = invokeWithFriendlyError<PluginStateGetResponse>("get_plugin_state", {
    roleId,
  }).finally(() => {
    if (pluginStateInflight.get(key) === p) {
      pluginStateInflight.delete(key);
    }
  });
  pluginStateInflight.set(key, p);
  return p;
}

export async function savePluginState(
  roleId: string,
  state: RolePluginState,
): Promise<void> {
  pluginStateInflight.delete(pluginStateCacheKey(roleId));
  return invokeWithFriendlyError<void>("save_plugin_state", {
    roleId,
    state,
  });
}

export async function saveGlobalPluginState(
  state: RolePluginState,
): Promise<void> {
  pluginStateInflight.clear();
  return invokeWithFriendlyError<void>("save_global_plugin_state", {
    state,
  });
}

/** 用磁盘上的 `ui.json` 覆盖该角色的本地插件 UI 状态。 */
export async function resetPluginStateToRoleDefault(
  roleId: string,
): Promise<void> {
  pluginStateInflight.delete(pluginStateCacheKey(roleId));
  return invokeWithFriendlyError<void>("reset_plugin_state_to_role_default", {
    roleId,
  });
}

export type HotkeyAction =
  | {
      type: "openPluginSlot";
      pluginId: string;
      slot: string;
      appearanceId?: string;
    }
  | { type: "openLauncherList" };

export interface HotkeyBinding {
  id: string;
  accelerator: string;
  enabled: boolean;
  action: HotkeyAction;
}

export interface HotkeyBindingsFile {
  schemaVersion: number;
  bindings: HotkeyBinding[];
}

export async function getHotkeyBindings(): Promise<HotkeyBindingsFile> {
  return invokeWithFriendlyError<HotkeyBindingsFile>("get_hotkey_bindings", {});
}

export async function saveHotkeyBindings(
  file: HotkeyBindingsFile,
): Promise<void> {
  return invokeWithFriendlyError<void>("save_hotkey_bindings", {
    bindings: file,
  });
}

/** B2：对指定目录插件懒启动后透传 JSON-RPC（方法名与 params 由插件定义）。 */
export async function directoryPluginInvoke(
  pluginId: string,
  method: string,
  params: unknown = {},
): Promise<unknown> {
  return invokeWithFriendlyError<unknown>("directory_plugin_invoke", {
    req: {
      pluginId,
      method,
      params,
    },
  });
}

/** 开发者调试：目录插件 RPC 子进程快照（与后端 `PluginProcessDebugInfo` 一致）。 */
export interface PluginProcessDebugInfo {
  pluginId: string;
  pid: number;
  rpcUrl: string;
  startedAtMs: number;
  cpuPercent?: number | null;
  memoryKb?: number | null;
}

export async function spawnPluginForTest(
  pluginId: string,
  configJson?: string | null,
): Promise<PluginProcessDebugInfo> {
  return invokeWithFriendlyError<PluginProcessDebugInfo>("spawn_plugin_for_test", {
    plugin_id: pluginId,
    config_json: configJson ?? null,
  });
}

export async function killPluginProcess(pluginId: string): Promise<void> {
  return invokeWithFriendlyError<void>("kill_plugin_process", { plugin_id: pluginId });
}

export async function listPluginProcesses(): Promise<PluginProcessDebugInfo[]> {
  return invokeWithFriendlyError<PluginProcessDebugInfo[]>("list_plugin_processes", {});
}

export async function getPluginLogs(
  pluginId: string,
  lines: number,
): Promise<string[]> {
  return invokeWithFriendlyError<string[]>("get_plugin_logs", {
    plugin_id: pluginId,
    lines,
  });
}

export async function clearPluginLogs(pluginId: string): Promise<void> {
  return invokeWithFriendlyError<void>("clear_plugin_logs", { plugin_id: pluginId });
}

export async function testPluginMethod(
  pluginId: string,
  method: string,
  params: unknown = {},
): Promise<unknown> {
  return invokeWithFriendlyError<unknown>("test_plugin_method", {
    req: {
      pluginId,
      method,
      params,
    },
  });
}

export async function discoverPluginMethods(pluginId: string): Promise<string[]> {
  return invokeWithFriendlyError<string[]>("discover_plugin_methods", {
    plugin_id: pluginId,
  });
}

/**
 * manifest `shell.bridge.invoke` 可声明 **命令名** 或 **权限别名**（后者用于 `get_conversation` → `read:conversation` 等）。
 * 敏感命令（聊天/角色切换）还要求 **`type`: `"ocliveplugin"`** 且来源为 **`shell.entry`** HTML 或 **`shell.vueEntry`** Vue 整壳。
 */
export type PluginBridgeManifestToken =
  | "send_message"
  | "read:conversation"
  | "switch_role"
  | "read:roles"
  | "read:current_role"
  | "get_role_info"
  | "list_roles"
  | "get_time_state"
  | "get_directory_plugin_bootstrap"
  | "get_conversation"
  | "get_roles"
  | "get_current_role"
  | "update_memory"
  | "delete_memory"
  | "update_emotion"
  | "update_event"
  | "update_prompt"
  | "write:memory"
  | "write:emotion"
  | "write:event"
  | "write:prompt"
  | "export_conversation"
  | "import_role"
  | "export:conversation"
  | "import:role"
  | "delete_role"
  | "update_settings"
  | "get_conversation_list"
  | "delete:role"
  | "write:settings"
  | "read:conversations";

/** 整壳 `OclivePluginBridge.invoke('update_memory', params)` */
export interface PluginBridgeUpdateMemoryParams {
  role_id: string;
  content: string;
  /** 0–1，默认 0.5 */
  importance?: number;
}

export interface PluginBridgeDeleteMemoryParams {
  role_id: string;
  memory_id: string;
}

export interface PluginBridgeUpdateEmotionParams {
  role_id: string;
  emotion: string;
}

export interface PluginBridgeUpdateEventParams {
  role_id: string;
  event_type: string;
  description?: string | null;
}

/** 预留；宿主未实现动态提示词片段时返回 `not_implemented`。 */
export interface PluginBridgeUpdatePromptParams {
  role_id: string;
  /** 由后续宿主契约定义 */
  fragment_key?: string;
  content?: string;
}

/** `plugin_bridge_invoke` → `send_message`（字段与 {@link SendMessageRequest} 一致；可提供 `text` 代替 `user_message`） */
export interface PluginBridgeSendMessageParams {
  role_id: string;
  user_message: string;
  scene_id?: string | null;
  session_id?: string | null;
  /** 与 `user_message` 二选一 */
  text?: string;
}

export interface PluginBridgeGetConversationParams {
  role_id: string;
  session_id?: string | null;
  limit?: number;
  offset?: number;
}

export interface PluginBridgeConversationTurn {
  user_input: string;
  bot_reply: string;
  emotion: string;
  scene: string | null;
  created_at: string;
}

export interface PluginBridgeGetConversationResult {
  role_id: string;
  session_namespace: string;
  total: number;
  limit: number;
  offset: number;
  items: PluginBridgeConversationTurn[];
}

/** `export_conversation` → 与 `export_chat_logs` 单角色导出一致（`format`: `json` | `txt`）。 */
export interface PluginBridgeExportConversationParams {
  role_id: string;
  format?: string;
  session_id?: string | null;
}

export interface PluginBridgeExportConversationResult {
  content: string;
  suggested_filename: string;
}

/** `import_role`：从 `.zip` / `.ocpak` 或已解压目录导入角色包。 */
export interface PluginBridgeImportRoleParams {
  path: string;
  /** 与 `src_path` 二选一 */
  src_path?: string;
  overwrite?: boolean;
}

export interface PluginBridgeImportRoleResult {
  role_id: string;
  ok: boolean;
}

/** `delete_role`：删除本地角色包及相关数据。 */
export interface PluginBridgeDeleteRoleParams {
  role_id?: string;
  roleId?: string;
}

export interface PluginBridgeDeleteRoleResult {
  ok: boolean;
  role_id: string;
}

/** `update_settings`：更新允许的应用设置（整壳白名单字段）。 */
export interface PluginBridgeUpdateSettingsParams {
  /** 与 `ui_theme` 二选一 */
  theme?: "light" | "dark" | "system";
  ui_theme?: "light" | "dark" | "system";
  interaction_mode?: string;
  [key: string]: unknown;
}

/** `get_conversation_list`：列出本地会话元数据。 */
export interface PluginBridgeConversationListItem {
  session_namespace: string;
  turn_count: number;
  last_at: string | null;
}

export interface PluginBridgeGetConversationListResult {
  items: PluginBridgeConversationListItem[];
}

/** 目录插件页 `OclivePluginBridge.invoke` 对应的后端入口（一般无需在主 UI 调用）。 */
export async function pluginBridgeInvoke(req: {
  pluginId: string;
  assetRel: string;
  command: string;
  params?: unknown;
}): Promise<unknown> {
  return invokeWithFriendlyError<unknown>("plugin_bridge_invoke", {
    req: {
      pluginId: req.pluginId,
      assetRel: req.assetRel,
      command: req.command,
      params: req.params ?? {},
    },
  });
}
