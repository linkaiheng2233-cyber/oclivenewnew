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

export function toFriendlyErrorMessage(err: unknown): string {
  const { code, raw } = parseBackendError(err);
  if (!code) return raw;
  if (code === "INVALID_PARAMETER" && raw.includes("角色包格式错误")) {
    const bracket = raw.indexOf("]");
    if (bracket !== -1) return raw.slice(bracket + 1).trim();
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

/** 与 `settings.json` → `plugin_backends` 一致（snake_case，与后端 serde 对齐） */
export interface PluginBackends {
  memory: "builtin" | "builtin_v2" | "remote" | "local";
  /** `memory === "local"` 时可选：与 `_local_plugins` 中 descriptor 的 `provider_id` 一致 */
  local_memory_provider_id?: string | null;
  emotion: "builtin" | "builtin_v2" | "remote";
  event: "builtin" | "builtin_v2" | "remote";
  prompt: "builtin" | "builtin_v2" | "remote";
  llm: "ollama" | "remote";
}

export interface PluginBackendsOverride {
  memory?: PluginBackends["memory"] | null;
  local_memory_provider_id?: string | null;
  emotion?: PluginBackends["emotion"] | null;
  event?: PluginBackends["event"] | null;
  prompt?: PluginBackends["prompt"] | null;
  llm?: PluginBackends["llm"] | null;
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
  llm_env_override?: "ollama" | "remote" | null;
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
  backend?: string | null,
  localMemoryProviderId?: string,
  sessionId?: string | null,
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
  return invokeWithFriendlyError<RoleInfo>("set_session_plugin_backend", {
    req,
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

/** 目录插件启动引导（整壳 URL、已扫描插件 id、开发者模式）。 */
export interface DirectoryPluginBootstrap {
  shellUrl?: string | null;
  shellPluginId?: string | null;
  pluginIds: string[];
  developerMode: boolean;
}

export async function getDirectoryPluginBootstrap(): Promise<DirectoryPluginBootstrap> {
  return invokeWithFriendlyError<DirectoryPluginBootstrap>(
    "get_directory_plugin_bootstrap",
    {},
  );
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
