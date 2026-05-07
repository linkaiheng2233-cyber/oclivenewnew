import { invoke } from "@tauri-apps/api/tauri";
import { i18n } from "../i18n";
import {
  TAURI_INVOKE_CAPABILITIES,
  capabilityKeyForCommand,
} from "../lib/tauriInvokeCapabilities";

function t(key: string, params?: Record<string, unknown>): string {
  return String(i18n.global.t(key as any, params as any));
}

const TransactionErrorMessageKeys: Record<string, string> = {
  TXN_BEGIN_FAILED: "apiErrors.txn.TXN_BEGIN_FAILED",
  TXN_RUNTIME_ENSURE_FAILED: "apiErrors.txn.TXN_RUNTIME_ENSURE_FAILED",
  TXN_PERSONALITY_INSERT_FAILED: "apiErrors.txn.TXN_PERSONALITY_INSERT_FAILED",
  TXN_FAVORABILITY_UPDATE_FAILED: "apiErrors.txn.TXN_FAVORABILITY_UPDATE_FAILED",
  TXN_FAVORABILITY_HISTORY_INSERT_FAILED:
    "apiErrors.txn.TXN_FAVORABILITY_HISTORY_INSERT_FAILED",
  TXN_MEMORY_INSERT_FAILED: "apiErrors.txn.TXN_MEMORY_INSERT_FAILED",
  TXN_SHORT_TERM_INSERT_FAILED: "apiErrors.txn.TXN_SHORT_TERM_INSERT_FAILED",
  TXN_SHORT_TERM_TRIM_FAILED: "apiErrors.txn.TXN_SHORT_TERM_TRIM_FAILED",
  TXN_EVENT_INSERT_FAILED: "apiErrors.txn.TXN_EVENT_INSERT_FAILED",
  TXN_FAVORABILITY_READ_FAILED: "apiErrors.txn.TXN_FAVORABILITY_READ_FAILED",
  TXN_COMMIT_FAILED: "apiErrors.txn.TXN_COMMIT_FAILED",
  TXN_ROLLBACK_FAILED: "apiErrors.txn.TXN_ROLLBACK_FAILED",
};

const CommonErrorMessageKeys: Record<string, string> = {
  DB_ERROR: "apiErrors.common.DB_ERROR",
  IO_ERROR: "apiErrors.common.IO_ERROR",
  API_PLUGIN_NOT_FOUND: "apiErrors.common.API_PLUGIN_NOT_FOUND",
  API_PERMISSION_DENIED: "apiErrors.common.API_PERMISSION_DENIED",
  API_INVALID_MANIFEST: "apiErrors.common.API_INVALID_MANIFEST",
  LLM_ERROR: "apiErrors.common.LLM_ERROR",
  ROLE_NOT_FOUND: "apiErrors.common.ROLE_NOT_FOUND",
  ROLE_PACK_EXISTS: "apiErrors.common.ROLE_PACK_EXISTS",
  INVALID_PARAMETER: "apiErrors.common.INVALID_PARAMETER",
  OLLAMA_TIMEOUT: "apiErrors.common.OLLAMA_TIMEOUT",
  TXN_ROLLBACK: "apiErrors.common.TXN_ROLLBACK",
  SERDE_ERROR: "apiErrors.common.SERDE_ERROR",
  UNKNOWN_ERROR: "apiErrors.common.UNKNOWN_ERROR",
  PLUGIN_PINNED_VERSION: "apiErrors.common.PLUGIN_PINNED_VERSION",
  PLUGIN_PUBKEY_REVOKED: "apiErrors.common.PLUGIN_PUBKEY_REVOKED",
  PLUGIN_PUBKEY_NOT_FOUND: "apiErrors.common.PLUGIN_PUBKEY_NOT_FOUND",
  PLUGIN_SIGNATURE_VERIFY_FAILED: "apiErrors.common.PLUGIN_SIGNATURE_VERIFY_FAILED",
  PLUGIN_SIGNATURE_BASE64_INVALID: "apiErrors.common.PLUGIN_SIGNATURE_BASE64_INVALID",
  PLUGIN_SIGNATURE_SIZE_INVALID: "apiErrors.common.PLUGIN_SIGNATURE_SIZE_INVALID",
  PLUGIN_SIGNATURE_ALGO_UNSUPPORTED:
    "apiErrors.common.PLUGIN_SIGNATURE_ALGO_UNSUPPORTED",
  PLUGIN_SIGNATURE_ID_MISMATCH: "apiErrors.common.PLUGIN_SIGNATURE_ID_MISMATCH",
  PLUGIN_ARCHIVE_TOO_MANY_FILES: "apiErrors.common.PLUGIN_ARCHIVE_TOO_MANY_FILES",
  PLUGIN_ARCHIVE_SINGLE_FILE_TOO_LARGE:
    "apiErrors.common.PLUGIN_ARCHIVE_SINGLE_FILE_TOO_LARGE",
  PLUGIN_ARCHIVE_TOTAL_TOO_LARGE: "apiErrors.common.PLUGIN_ARCHIVE_TOTAL_TOO_LARGE",
  PLUGIN_ARCHIVE_ILLEGAL_PATH: "apiErrors.common.PLUGIN_ARCHIVE_ILLEGAL_PATH",
  ZIP_TOO_MANY_FILES: "apiErrors.common.ZIP_TOO_MANY_FILES",
  ZIP_SINGLE_FILE_TOO_LARGE: "apiErrors.common.ZIP_SINGLE_FILE_TOO_LARGE",
  ZIP_TOTAL_TOO_LARGE: "apiErrors.common.ZIP_TOTAL_TOO_LARGE",
  PLUGIN_PERMISSION_NOT_GRANTED: "apiErrors.common.PLUGIN_PERMISSION_NOT_GRANTED",
  CHAT_GENERATION_CANCELLED: "apiErrors.common.CHAT_GENERATION_CANCELLED",
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

// ===== Module 9: Expert Models =====

export type ExpertConfigSource =
  | "pack_default"
  | "role_default"
  | "session_override";

export interface PromptStyleOverride {
  replyQualityAnchor?: string | null;
  corePersonality?: string | null;
  description?: string | null;
}

export type ExpertNode =
  | {
      type: "base_model";
      id: string;
      ggufPath: string;
      ui?: { x: number; y: number } | null;
    }
  | {
      type: "lora_adapter";
      id: string;
      ggufPath: string;
      strength: number;
      enabled: boolean;
      order: number;
      ui?: { x: number; y: number } | null;
    }
  | {
      type: "prompt_style";
      id: string;
      style: PromptStyleOverride;
      ui?: { x: number; y: number } | null;
    }
  | {
      type: "cloud_model";
      id: string;
      /** Only `"host"` is supported (global host cloud LLM). */
      hostSource?: string;
      /** Overrides OpenAI-compatible `model` id for this session when set. */
      model?: string | null;
      enabled: boolean;
      ui?: { x: number; y: number } | null;
    }
  | {
      type: "event_trigger";
      id: string;
      matchSubstring: string;
      memoryContent: string;
      importance: number;
      enabled: boolean;
      ui?: { x: number; y: number } | null;
    };

export interface ExpertEdge {
  from: string;
  to: string;
}

export interface ExpertGraph {
  /** Graph schema revision (backend accepts 1+; v2 reserved for future). */
  version: number;
  nodes: ExpertNode[];
  edges: ExpertEdge[];
}

/** Result of `expert_models_apply_to_session` (camelCase from Tauri). */
export interface ExpertModelsApplyResult {
  ok: boolean;
  llamaPluginId: string;
  modelPath?: string | null;
  llamaArgs?: string | null;
  sidecarNotice?: string | null;
  useRemoteLlm?: boolean;
  remoteModelOverride?: string | null;
}

export interface ExpertModelsEffectiveResponse {
  graph: ExpertGraph;
  promptStyle?: PromptStyleOverride | null;
  graphSource: ExpertConfigSource;
  promptStyleSource: ExpertConfigSource;
  canRollbackLastRun?: boolean;
  /** DB 中的角色级默认图（会话覆盖生效时仍返回，便于「加载角色默认」）。 */
  roleDefaultGraph?: ExpertGraph | null;
  roleDefaultPromptStyle?: PromptStyleOverride | null;
}

export interface LocalModelFileDto {
  name: string;
  path: string;
  /** 本地仓库备注（`models/gguf/.oclive_gguf_repo.json`） */
  repoNotes?: string;
  repoSourceUrl?: string;
  repoTags?: string[];
}

export interface ExpertWorkflowSummaryDto {
  id: string;
  name: string;
  updatedAtMs: number;
}

export interface ExpertWorkflowDto {
  id: string;
  name: string;
  updatedAtMs: number;
  graph: ExpertGraph;
  promptStyle?: PromptStyleOverride | null;
}

export async function expertWorkflowsList(): Promise<{ items: ExpertWorkflowSummaryDto[] }> {
  return invokeWithFriendlyError<{ items: ExpertWorkflowSummaryDto[] }>(
    "expert_workflows_list",
    {},
  );
}

export async function expertWorkflowsGet(id: string): Promise<ExpertWorkflowDto> {
  return invokeWithFriendlyError<ExpertWorkflowDto>("expert_workflows_get", { req: { id } });
}

export async function expertWorkflowsSave(params: {
  id?: string | null;
  name: string;
  graph: ExpertGraph;
  promptStyle?: PromptStyleOverride | null;
}): Promise<ExpertWorkflowDto> {
  return invokeWithFriendlyError<ExpertWorkflowDto>("expert_workflows_save", {
    req: {
      id: params.id ?? null,
      name: params.name,
      graph: params.graph,
      promptStyle: params.promptStyle ?? null,
    },
  });
}

export async function expertWorkflowsDelete(id: string): Promise<void> {
  return invokeWithFriendlyError<void>("expert_workflows_delete", { req: { id } });
}

export async function expertModelsGetEffective(params: {
  roleId: string;
  sessionId?: string | null;
}): Promise<ExpertModelsEffectiveResponse> {
  return invokeWithFriendlyError<ExpertModelsEffectiveResponse>(
    "expert_models_get_effective",
    {
      req: { roleId: params.roleId, sessionId: params.sessionId ?? null },
    },
  );
}

export async function expertModelsSetSessionOverride(params: {
  roleId: string;
  sessionId?: string | null;
  graph: ExpertGraph;
  promptStyle?: PromptStyleOverride | null;
}): Promise<void> {
  return invokeWithFriendlyError<void>("expert_models_set_session_override", {
    req: {
      roleId: params.roleId,
      sessionId: params.sessionId ?? null,
      graph: params.graph,
      promptStyle: params.promptStyle ?? null,
    },
  });
}

export async function expertModelsClearSessionOverride(params: {
  roleId: string;
  sessionId?: string | null;
}): Promise<void> {
  return invokeWithFriendlyError<void>("expert_models_clear_session_override", {
    req: { roleId: params.roleId, sessionId: params.sessionId ?? null },
  });
}

export async function expertModelsSetRoleDefault(params: {
  roleId: string;
  graph: ExpertGraph;
  promptStyle?: PromptStyleOverride | null;
}): Promise<void> {
  return invokeWithFriendlyError<void>("expert_models_set_role_default", {
    req: {
      roleId: params.roleId,
      graph: params.graph,
      promptStyle: params.promptStyle ?? null,
    },
  });
}

export async function expertModelsClearRoleDefault(params: {
  roleId: string;
}): Promise<void> {
  return invokeWithFriendlyError<void>("expert_models_clear_role_default", {
    req: { roleId: params.roleId },
  });
}

export async function expertModelsApplyToSession(params: {
  roleId: string;
  sessionId?: string | null;
}): Promise<ExpertModelsApplyResult> {
  return invokeWithFriendlyError<ExpertModelsApplyResult>("expert_models_apply_to_session", {
    req: { roleId: params.roleId, sessionId: params.sessionId ?? null },
  });
}

export async function expertModelsRollbackLastRun(params: {
  roleId: string;
  sessionId?: string | null;
}): Promise<ExpertModelsApplyResult> {
  return invokeWithFriendlyError<ExpertModelsApplyResult>("expert_models_rollback_last_run", {
    req: { roleId: params.roleId, sessionId: params.sessionId ?? null },
  });
}

export interface ExpertModelsRunSummaryDto {
  indexFromLatest: number;
  atMs: number;
  pinned?: boolean | null;
  targetBaseName: string;
  targetLoraCount: number;
  targetHasPromptStyle: boolean;
  applyOk?: boolean | null;
  applyError?: string | null;
  applyDurationMs?: number | null;
  applySidecarNotice?: string | null;
}

export async function expertModelsListRuns(params: {
  roleId: string;
  sessionId?: string | null;
}): Promise<{ items: ExpertModelsRunSummaryDto[] }> {
  return invokeWithFriendlyError<{ items: ExpertModelsRunSummaryDto[] }>(
    "expert_models_list_runs",
    { req: { roleId: params.roleId, sessionId: params.sessionId ?? null } },
  );
}

export async function expertModelsClearRuns(params: {
  roleId: string;
  sessionId?: string | null;
  mode?: "all" | "ok" | "failed" | "unpinned";
  keepPinned?: boolean;
}): Promise<void> {
  return invokeWithFriendlyError<void>("expert_models_clear_runs", {
    req: {
      roleId: params.roleId,
      sessionId: params.sessionId ?? null,
      mode: params.mode ?? "all",
      keepPinned: params.keepPinned ?? null,
    },
  });
}

export async function expertModelsRollbackToRun(params: {
  roleId: string;
  sessionId?: string | null;
  indexFromLatest: number;
}): Promise<ExpertModelsApplyResult> {
  return invokeWithFriendlyError<ExpertModelsApplyResult>("expert_models_rollback_to_run", {
    req: {
      roleId: params.roleId,
      sessionId: params.sessionId ?? null,
      indexFromLatest: params.indexFromLatest,
    },
  });
}

export interface ExpertModelsRunDetailDto {
  indexFromLatest: number;
  atMs: number;
  pinned?: boolean | null;
  snapshotGraph: ExpertGraph;
  snapshotPromptStyle?: PromptStyleOverride | null;
  snapshotBaseName: string;
  snapshotLoraCount: number;
  snapshotHasPromptStyle: boolean;
  targetGraph?: ExpertGraph | null;
  targetPromptStyle?: PromptStyleOverride | null;
  targetBaseName: string;
  targetLoraCount: number;
  targetHasPromptStyle: boolean;
  applyOk?: boolean | null;
  applyError?: string | null;
  applyModelPath?: string | null;
  applyLlamaArgs?: string | null;
  applyDurationMs?: number | null;
  applySidecarNotice?: string | null;
}

export async function expertModelsSetRunPinned(params: {
  roleId: string;
  sessionId?: string | null;
  indexFromLatest: number;
  pinned: boolean;
}): Promise<void> {
  return invokeWithFriendlyError<void>("expert_models_set_run_pinned", {
    req: {
      roleId: params.roleId,
      sessionId: params.sessionId ?? null,
      indexFromLatest: params.indexFromLatest,
      pinned: params.pinned,
    },
  });
}

export async function expertModelsGetRunDetail(params: {
  roleId: string;
  sessionId?: string | null;
  indexFromLatest: number;
}): Promise<{ item: ExpertModelsRunDetailDto }> {
  return invokeWithFriendlyError<{ item: ExpertModelsRunDetailDto }>(
    "expert_models_get_run_detail",
    {
      req: {
        roleId: params.roleId,
        sessionId: params.sessionId ?? null,
        indexFromLatest: params.indexFromLatest,
      },
    },
  );
}

export async function expertModelsListLocalBaseModels(): Promise<LocalModelFileDto[]> {
  return invokeWithFriendlyError<LocalModelFileDto[]>(
    "expert_models_list_local_base_models",
    {},
  );
}

export async function expertModelsListLocalLoras(): Promise<LocalModelFileDto[]> {
  return invokeWithFriendlyError<LocalModelFileDto[]>(
    "expert_models_list_local_loras",
    {},
  );
}

export async function expertModelsImportBaseGguf(
  sourcePath: string,
): Promise<LocalModelFileDto> {
  return invokeWithFriendlyError<LocalModelFileDto>("expert_models_import_base_gguf", {
    req: { sourcePath },
  });
}

export async function expertModelsImportLoraGguf(
  sourcePath: string,
): Promise<LocalModelFileDto> {
  return invokeWithFriendlyError<LocalModelFileDto>("expert_models_import_lora_gguf", {
    req: { sourcePath },
  });
}

export async function expertModelsDeleteLocalBaseModel(path: string): Promise<void> {
  return invokeWithFriendlyError<void>("expert_models_delete_local_base_model", {
    req: { path },
  });
}

export async function expertModelsRenameLocalBaseModel(
  path: string,
  newFileName: string,
): Promise<LocalModelFileDto> {
  return invokeWithFriendlyError<LocalModelFileDto>("expert_models_rename_local_base_model", {
    req: { path, newFileName },
  });
}

export async function expertModelsSetGgufRepoMeta(params: {
  path: string;
  notes: string;
  sourceUrl: string;
  tags: string[];
}): Promise<LocalModelFileDto> {
  return invokeWithFriendlyError<LocalModelFileDto>("expert_models_set_gguf_repo_meta", {
    req: {
      path: params.path,
      notes: params.notes,
      sourceUrl: params.sourceUrl,
      tags: params.tags,
    },
  });
}

/** 本机是否出现 Ollama / llama.cpp 系进程（供前端轮询刷新模型列表）。 */
export interface LocalLlmRuntimeProbeDto {
  ollamaProcess: boolean;
  llamaLikeProcess: boolean;
}

export async function probeLocalLlmRuntime(): Promise<LocalLlmRuntimeProbeDto> {
  return invokeWithFriendlyError<LocalLlmRuntimeProbeDto>("probe_local_llm_runtime", {});
}

export async function ollamaModelsHealth(): Promise<boolean> {
  return invokeWithFriendlyError<boolean>("ollama_models_health", {});
}

export async function ollamaModelsListNames(): Promise<string[]> {
  return invokeWithFriendlyError<string[]>("ollama_models_list_names", {});
}

export async function ollamaModelsDelete(name: string): Promise<void> {
  return invokeWithFriendlyError<void>("ollama_models_delete", {
    req: { name },
  });
}

/** 纯聊模式：聊天等路径用大白话，避免堆栈与 HTTP 细节。 */
export function toPureChatPlainErrorMessage(err: unknown): string {
  const { code, raw } = parseBackendError(err);
  if (code === "CHAT_GENERATION_CANCELLED") {
    return t("apiErrors.common.CHAT_GENERATION_CANCELLED");
  }
  if (code === "LLM_ERROR" || code === "OLLAMA_TIMEOUT") {
    return t("app.pureChatErrors.llm");
  }
  if (code === "INVALID_PARAMETER") {
    return t("app.pureChatErrors.invalid");
  }
  if (code === "ROLE_NOT_FOUND") {
    return t("app.pureChatErrors.noRole");
  }
  if (code === "ROLE_PACK_EXISTS") {
    return t("app.pureChatErrors.packExists");
  }
  if (code === "DB_ERROR") {
    return t("app.pureChatErrors.db");
  }
  const message = err instanceof Error ? err.message : raw;
  if (
    code === "API_PERMISSION_DENIED" ||
    code === "PLUGIN_PERMISSION_NOT_GRANTED" ||
    message === t("apiErrors.common.PLUGIN_PERMISSION_NOT_GRANTED") ||
    message === t("apiErrors.common.API_PERMISSION_DENIED") ||
    message.includes("尚未被授予所需权限")
  ) {
    return t("app.pureChatErrors.permission");
  }
  return t("app.pureChatErrors.generic");
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
        return t("apiErrors.special.roleNotFoundWithDetail", {
          detail: detail.slice("Role not found:".length).trim(),
        });
      }
    }
  }
  if (code === "IO_ERROR" && raw.includes("host json")) {
    return t("apiErrors.special.hostJsonSerdeFailed");
  }
  const txnKey = TransactionErrorMessageKeys[code];
  if (txnKey) return t(txnKey);
  const commonKey = CommonErrorMessageKeys[code];
  if (commonKey) return t(commonKey);
  return raw;
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
  const cap = capabilityKeyForCommand(command);
  if (cap && !TAURI_INVOKE_CAPABILITIES[cap]) {
    const friendly: FriendlyError = {
      code: "INVOKE_NOT_COMPILED",
      message: t("apiErrors.invoke.notCompiled", {
        command,
        group: cap,
      }),
      raw: `invoke ${command} (${cap} disabled)`,
    };
    console.error(`[tauri:${command}]`, friendly.code ?? "?", friendly.raw);
    if (errorReporter) {
      errorReporter(friendly);
    }
    throw new Error(friendly.message);
  }
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
  agent?: string | null;
}

/** 与 `settings.json` → `plugin_backends` 一致（snake_case，与后端 serde 对齐） */
export interface PluginBackends {
  memory: "builtin" | "builtin_v2" | "remote" | "local" | "directory" | "none";
  /** `memory === "local"` 时可选：与 `_local_plugins` 中 descriptor 的 `provider_id` 一致 */
  local_memory_provider_id?: string | null;
  emotion: "builtin" | "builtin_v2" | "remote" | "directory" | "none";
  event: "builtin" | "builtin_v2" | "remote" | "directory" | "none";
  prompt: "builtin" | "builtin_v2" | "remote" | "directory" | "none";
  llm: "ollama" | "remote" | "directory" | "none";
  agent: "builtin" | "remote" | "directory" | "none";
  complex_emotion: "builtin" | "remote" | "directory" | "none";
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
  agent?: PluginBackends["agent"] | null;
  complex_emotion?: PluginBackends["complex_emotion"] | null;
  /** 会话级与包内按槽合并（当前 UI 未编辑；仅展示与调试） */
  directory_plugins?: DirectoryPluginSlots | null;
  /** 合并后强制清空 `directory_plugins.llm`（专家图云端路径） */
  forceClearDirectoryLlmSlot?: boolean;
}

export type PluginBackendSource = "pack_default" | "session_override" | "env_override";

export interface PluginBackendsSourceMap {
  memory: PluginBackendSource;
  emotion: PluginBackendSource;
  event: PluginBackendSource;
  prompt: PluginBackendSource;
  llm: PluginBackendSource;
  agent: PluginBackendSource;
  complex_emotion: PluginBackendSource;
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
  /** manifest `creator_message_to_downloader` */
  creator_message_to_downloader?: string | null;
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
  /** manifest `creator_message_to_downloader` */
  creator_message_to_downloader?: string | null;
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

export interface CreateRoleFeedbackRequest {
  role_id: string;
  session_id?: string | null;
  mood_tag?: string | null;
  scene_id?: string | null;
  presence_mode?: string | null;
  role_version?: string | null;
  client_version?: string | null;
  message: string;
}

export interface CreateRoleFeedbackResponse {
  id: number;
}

export interface QueryRoleFeedbackRequest {
  role_id: string;
  limit: number;
  offset: number;
}

export interface RoleFeedbackItem {
  id: number;
  role_id: string;
  session_id?: string | null;
  mood_tag?: string | null;
  message: string;
  timestamp: string;
  status?: string | null;
  read_at?: string | null;
  handled_at?: string | null;
  handled_note?: string | null;
  scene_id?: string | null;
  presence_mode?: string | null;
  role_version?: string | null;
  runtime_version?: string | null;
  client_version?: string | null;
  source?: string | null;
}

export interface PluginReviewEntryDto {
  id: string;
  plugin_id: string;
  pubkey_id?: string | null;
  version?: string | null;
  rating: number;
  title?: string | null;
  body?: string | null;
  created_at: string;
  author_github?: string | null;
}

export interface PluginReviewsIndexDto {
  schema_version: number;
  generated_at?: string | null;
  reviews: PluginReviewEntryDto[];
}

export type LocalImportKind =
  | "role_pack"
  | "plugin_archive"
  | "plugin_dir"
  | "module_json"
  | "profile_json";

export interface LocalImportCandidateDto {
  kind: LocalImportKind;
  path: string;
  fileName: string;
  relatedSignaturePath?: string | null;
  sizeBytes?: number | null;
  modifiedMs?: number | null;
}

export interface ListLocalImportCandidatesResponseDto {
  items: LocalImportCandidateDto[];
  rootDir: string;
}

export async function listLocalImportCandidates(): Promise<ListLocalImportCandidatesResponseDto> {
  return invokeWithFriendlyError<ListLocalImportCandidatesResponseDto>(
    "list_local_import_candidates_command",
    {},
  );
}

export async function readLocalImportText(path: string): Promise<string> {
  const r = await invokeWithFriendlyError<{ content: string }>(
    "read_local_import_text_command",
    { req: { path } },
  );
  return r.content;
}

export interface PreviewLocalPluginArchiveResponseDto {
  pluginId: string;
  declaredPermissions: string[];
  signatureVerified: boolean;
  signatureMessage?: string | null;
}

export async function previewLocalPluginArchive(params: {
  archivePath: string;
  signaturePath?: string | null;
}): Promise<PreviewLocalPluginArchiveResponseDto> {
  return invokeWithFriendlyError<PreviewLocalPluginArchiveResponseDto>(
    "preview_local_plugin_archive_command",
    {
      req: {
        archivePath: params.archivePath,
        signaturePath: params.signaturePath ?? null,
      },
    },
  );
}

export async function installLocalPluginArchive(params: {
  archivePath: string;
  signaturePath?: string | null;
  overwrite: boolean;
  acceptedPermissions?: string[] | null;
}): Promise<string> {
  return invokeWithFriendlyError<string>("install_local_plugin_archive_command", {
    req: {
      archivePath: params.archivePath,
      signaturePath: params.signaturePath ?? null,
      overwrite: params.overwrite,
      acceptedPermissions: params.acceptedPermissions ?? null,
    },
  });
}

export async function syncPluginReviewsIndex(
  sourceUrl?: string | null,
): Promise<PluginReviewsIndexDto> {
  return invokeWithFriendlyError<PluginReviewsIndexDto>(
    "sync_plugin_reviews_index",
    { req: { sourceUrl: sourceUrl ?? null } },
  );
}

export async function getCachedPluginReviewsIndex(): Promise<PluginReviewsIndexDto> {
  return invokeWithFriendlyError<PluginReviewsIndexDto>(
    "get_cached_plugin_reviews_index",
    {},
  );
}

export async function sendMessage(
  req: SendMessageRequest,
): Promise<SendMessageResponse> {
  return invokeWithFriendlyError<SendMessageResponse>("send_message", { req });
}

export async function cancelChatGeneration(): Promise<void> {
  await invoke("cancel_chat_generation");
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

export async function createRoleFeedback(
  req: CreateRoleFeedbackRequest,
): Promise<CreateRoleFeedbackResponse> {
  return invokeWithFriendlyError<CreateRoleFeedbackResponse>("create_role_feedback", { req });
}

export async function queryRoleFeedback(
  req: QueryRoleFeedbackRequest,
): Promise<RoleFeedbackItem[]> {
  return invokeWithFriendlyError<RoleFeedbackItem[]>("query_role_feedback", { req });
}

export async function markRoleFeedbackRead(params: {
  roleId: string;
  ids: number[];
}): Promise<number> {
  return invokeWithFriendlyError<number>("mark_role_feedback_read", {
    req: { roleId: params.roleId, ids: params.ids },
  });
}

export async function setRoleFeedbackHandled(params: {
  roleId: string;
  id: number;
  handled: boolean;
  note?: string | null;
}): Promise<void> {
  return invokeWithFriendlyError<void>("set_role_feedback_handled", {
    req: {
      roleId: params.roleId,
      id: params.id,
      handled: params.handled,
      note: params.note ?? null,
    },
  });
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
  module:
    | "memory"
    | "emotion"
    | "event"
    | "prompt"
    | "llm"
    | "agent"
    | "complex_emotion",
  /** 与后端 `parse_backend_wire` 一致，如 `builtin_v2`、`directory`、`remote` */
  backend?: string | null,
  localMemoryProviderId?: string,
  sessionId?: string | null,
  directoryPluginId?: string | null,
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
  if (directoryPluginId !== undefined) {
    req.directory_plugin_id = directoryPluginId;
  }
  return invokeWithFriendlyError<RoleInfo>("set_session_plugin_backend", {
    req,
  });
}

export async function setSessionPluginBackendsOverride(
  roleId: string,
  overrideBackends: Record<string, unknown>,
  sessionId?: string | null,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>("set_session_plugin_backends_override", {
    req: {
      role_id: roleId,
      session_id: sessionId ?? null,
      override_backends: overrideBackends,
    },
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
  creator_message_to_downloader?: string | null;
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

/** 读取 `roles/{roleId}/creator_message.txt`：每个非空行视为一条寄语。 */
export async function readRoleCreatorMessageLines(
  roleId: string,
): Promise<string[]> {
  return invokeWithFriendlyError<string[]>(
    "read_role_creator_message_lines_command",
    { role_id: roleId },
  );
}

export interface RoleMarketDownloadDto {
  label: string;
  kind: "direct" | "page" | "pan" | string;
  url: string;
  sha256: string;
  note?: string | null;
  trust?: "official" | "verified" | "community" | "unknown" | string | null;
}

export interface RoleMarketEntryDto {
  type: "role" | string;
  id: string;
  name: string;
  description?: string;
  author?: string;
  version: string;
  minRuntimeVersion?: string | null;
  tags?: string[];
  downloads: RoleMarketDownloadDto[];
}

export interface RoleMarketIndexDto {
  generatedAt?: string | null;
  roles: RoleMarketEntryDto[];
  warning?: string | null;
}

export async function syncRoleMarketIndex(sourceUrl?: string | null): Promise<RoleMarketIndexDto> {
  return invokeWithFriendlyError<RoleMarketIndexDto>("sync_role_market_index", {
    req: { sourceUrl: sourceUrl ?? null },
  });
}

export async function installRolePackFromMarket(params: {
  roleId: string;
  downloadUrl: string;
  sha256: string;
  overwrite: boolean;
}): Promise<string> {
  return invokeWithFriendlyError<string>("install_role_pack_from_market", {
    req: {
      roleId: params.roleId,
      downloadUrl: params.downloadUrl,
      sha256: params.sha256,
      overwrite: params.overwrite,
    },
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
  /** 前端壳（Module 8）支持的官方插槽名清单（稳定契约）。 */
  supportedUiSlots: string[];
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
  acceptedPermissions?: string[] | null,
): Promise<void> {
  return invokeWithFriendlyError<void>("extract_plugin_zip", {
    zip_path: zipPath,
    plugin_id: pluginId,
    accepted_permissions: acceptedPermissions ?? null,
  });
}

export interface PluginZipPermissionPreviewDto {
  pluginId: string;
  permissions: string[];
}

export interface PluginDirPermissionPreviewDto {
  pluginId: string;
  permissions: string[];
}

export async function previewPluginZipPermissions(
  zipPath: string,
): Promise<PluginZipPermissionPreviewDto> {
  return invokeWithFriendlyError<PluginZipPermissionPreviewDto>(
    "preview_plugin_zip_permissions",
    { zip_path: zipPath },
  );
}

export async function previewPluginDirPermissions(
  dirPath: string,
): Promise<PluginDirPermissionPreviewDto> {
  return invokeWithFriendlyError<PluginDirPermissionPreviewDto>(
    "preview_plugin_dir_permissions",
    { dir_path: dirPath },
  );
}

export async function installPluginDir(
  dirPath: string,
  pluginId: string,
  acceptedPermissions?: string[] | null,
): Promise<void> {
  return invokeWithFriendlyError<void>("install_plugin_dir", {
    dir_path: dirPath,
    plugin_id: pluginId,
    accepted_permissions: acceptedPermissions ?? null,
  });
}

export interface PluginAuditLogRowDto {
  createdAt: string;
  action: string;
  permission?: string | null;
  allowed: boolean;
  metaJson: string;
}

export interface GetPluginAuditLogsResponseDto {
  logs: PluginAuditLogRowDto[];
}

export async function getPluginAuditLogs(
  pluginId: string,
  limit?: number,
): Promise<GetPluginAuditLogsResponseDto> {
  return invokeWithFriendlyError<GetPluginAuditLogsResponseDto>(
    "get_plugin_audit_logs",
    { req: { pluginId, limit: limit ?? 50 } },
  );
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
  installMeta?: {
    installMethod: string;
    gitUrl?: string | null;
    pinnedTag?: string | null;
    declaredPermissions?: string[];
    grantedPermissions?: string[];
  } | null;
  /** manifest 含 `uiTemplate` 或 `uiSchema.fields` */
  hasUiSettings?: boolean;
  /** manifest 是否含 `process`（可在此面板启动 JSON-RPC 子进程） */
  hasRpcProcess: boolean;
  /** manifest 是否声明 `rpcMethods`（调试面板可预填方法名） */
  declaresRpcMethods?: boolean;
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

/** 网页索引中的单条插件（与 `plugin_installer::PluginIndexEntry` 一致，camelCase）。 */
export interface PluginIndexEntryDto {
  /** plugins.json 字段名就是 `type`（与 Rust serde rename 对齐） */
  type?: "plugin" | "module" | "profile";
  id: string;
  name: string;
  description: string;
  author: string;
  version: string;
  /** `type=plugin` 必填；`module` 可能为空字符串 */
  git: string;
  permissions: string[];
  tags: string[];
  category?: string | null;
  source?: string | null;
  changelog?: string | null;
  dependencies: Record<string, string>;
  publisher?: string | null;
  publicKeys?: {
    pubkeyId: string;
    publicKey: string;
    status?: string | null;
    rotatedTo?: string | null;
  }[];
  versions?: {
    version: string;
    downloadUrl?: string | null;
    signatureUrl?: string | null;
    gitTag?: string | null;
  }[];

  /** `type=module` 时可选：模块声明（无代码） */
  module?: {
    plugins: { id: string; version?: string | null; source?: string | null }[];
    backends?: Record<string, unknown> | null;
  } | null;

  /** `type=profile` 时可选：Profile 声明（无代码） */
  profile?: {
    plugins: { id: string; version?: string | null; source?: string | null }[];
    backends?: Record<string, unknown> | null;
    predeclaredPermissions?: string[] | null;
  } | null;
}

export interface PluginMarketEntryDto extends PluginIndexEntryDto {
  installed: boolean;
  installedVersion?: string | null;
  hasUpdate: boolean;
  missingDependencies: string[];
}

export interface PluginMarketSnapshotDto {
  plugins: PluginMarketEntryDto[];
  offlineMode: boolean;
  source: string;
  warning?: string | null;
}

export interface PendingProtocolInstallDto {
  gitUrl: string;
}

export interface PluginMarketSourcesConfigDto {
  developerMode: boolean;
  pluginIndexSources: string[];
}

export interface PluginPermissionGrantDto {
  permission: string;
  enabled: boolean;
}

export interface GetPluginPermissionGrantsResponseDto {
  pluginId: string;
  grants: PluginPermissionGrantDto[];
}

export interface InstallPluginFromMarketResponseDto {
  installedPluginId: string;
}

export async function syncPluginIndexCommand(
  indexUrl?: string | null,
): Promise<PluginMarketSnapshotDto> {
  return invokeWithFriendlyError<PluginMarketSnapshotDto>(
    "sync_plugin_index_command",
    { indexUrl: indexUrl ?? null },
  );
}

export async function getCachedPluginIndex(): Promise<PluginMarketSnapshotDto> {
  return invokeWithFriendlyError<PluginMarketSnapshotDto>(
    "get_cached_plugin_index",
    {},
  );
}

export async function installPluginFromMarket(
  pluginId: string,
  gitUrl?: string | null,
  acceptedPermissions?: string[] | null,
): Promise<InstallPluginFromMarketResponseDto> {
  return invokeWithFriendlyError<InstallPluginFromMarketResponseDto>(
    "install_plugin_from_market",
    {
      pluginId,
      gitUrl: gitUrl ?? null,
      consent: { acceptedPermissions: acceptedPermissions ?? [] },
    },
  );
}

export async function installPluginVersionFromMarket(
  pluginId: string,
  version: string,
  acceptedPermissions?: string[] | null,
): Promise<InstallPluginFromMarketResponseDto> {
  return invokeWithFriendlyError<InstallPluginFromMarketResponseDto>(
    "install_plugin_version_from_market",
    { req: { pluginId, version, acceptedPermissions: acceptedPermissions ?? [] } },
  );
}

export async function installPluginFromGit(
  gitUrl: string,
): Promise<InstallPluginFromMarketResponseDto> {
  return invokeWithFriendlyError<InstallPluginFromMarketResponseDto>(
    "install_plugin_from_git",
    { req: { gitUrl } },
  );
}

export async function updatePluginFromMarket(pluginId: string): Promise<void> {
  return invokeWithFriendlyError<void>("update_plugin_from_market", {
    pluginId,
  });
}

export async function uninstallPluginFromMarket(pluginId: string): Promise<void> {
  return invokeWithFriendlyError<void>("uninstall_plugin_from_market", {
    pluginId,
  });
}

export async function batchUpdatePlugins(pluginIds: string[]): Promise<void> {
  return invokeWithFriendlyError<void>("batch_update_plugins", { pluginIds });
}

export async function batchUninstallPlugins(pluginIds: string[]): Promise<void> {
  return invokeWithFriendlyError<void>("batch_uninstall_plugins", { pluginIds });
}

export async function consumePendingProtocolInstalls(): Promise<
  PendingProtocolInstallDto[]
> {
  return invokeWithFriendlyError<PendingProtocolInstallDto[]>(
    "consume_pending_protocol_installs",
    {},
  );
}

export async function getPluginMarketSourcesConfig(): Promise<PluginMarketSourcesConfigDto> {
  return invokeWithFriendlyError<PluginMarketSourcesConfigDto>(
    "get_plugin_market_sources_config",
    {},
  );
}

export async function setPluginMarketDeveloperMode(
  enabled: boolean,
): Promise<PluginMarketSourcesConfigDto> {
  return invokeWithFriendlyError<PluginMarketSourcesConfigDto>(
    "set_plugin_market_developer_mode",
    { req: { enabled } },
  );
}

export async function setPluginIndexSources(
  sources: string[],
): Promise<PluginMarketSourcesConfigDto> {
  return invokeWithFriendlyError<PluginMarketSourcesConfigDto>(
    "set_plugin_index_sources",
    { req: { sources } },
  );
}

export async function getPluginPermissionGrants(
  pluginId: string,
): Promise<GetPluginPermissionGrantsResponseDto> {
  return invokeWithFriendlyError<GetPluginPermissionGrantsResponseDto>(
    "get_plugin_permission_grants",
    { pluginId },
  );
}

export async function setPluginPermissionGrant(
  pluginId: string,
  permission: string,
  enabled: boolean,
): Promise<void> {
  return invokeWithFriendlyError<void>("set_plugin_permission_grant", {
    req: { pluginId, permission, enabled },
  });
}

export type PermissionRisk = "low" | "medium" | "high";

export interface PermissionTokenInfoDto {
  token: string;
  title: string;
  description: string;
  risk: PermissionRisk;
}

export interface ListPermissionTokensResponseDto {
  tokens: PermissionTokenInfoDto[];
}

export async function listPermissionTokens(): Promise<ListPermissionTokensResponseDto> {
  return invokeWithFriendlyError<ListPermissionTokensResponseDto>(
    "list_permission_tokens",
    {},
  );
}

export interface ProfilePluginSpecDto {
  id: string;
  version?: string | null;
  source?: string | null;
}

export interface ProfilePermissionsDto {
  predeclared: string[];
  requireConfirm: string[];
}

export interface ProfileBackendsDto {
  memory?: string | null;
  emotion?: string | null;
  event?: string | null;
  prompt?: string | null;
  llm?: string | null;
  agent?: string | null;
  complexEmotion?: string | null;
}

export interface ProfilePreviewDto {
  id: string;
  name: string;
  version: string;
  developerMode: boolean;
  marketSources: string[];
  plugins: ProfilePluginSpecDto[];
  permissions?: ProfilePermissionsDto | null;
  backends?: ProfileBackendsDto | null;
}

export async function previewProfileFromPath(path: string): Promise<ProfilePreviewDto> {
  return invokeWithFriendlyError<ProfilePreviewDto>("preview_profile_from_path", {
    req: { path },
  });
}

export interface UiSchemaFieldDto {
  key: string;
  label: string;
  type: string;
  required: boolean;
  default?: unknown;
}

export interface PluginUiSettingsDto {
  uiTemplate?: string | null;
  fields: UiSchemaFieldDto[];
  config: Record<string, unknown>;
}

export async function getPluginSettingsUi(
  pluginId: string,
): Promise<PluginUiSettingsDto> {
  return invokeWithFriendlyError<PluginUiSettingsDto>("get_plugin_settings_ui", {
    pluginId,
  });
}

export async function setPluginSettingsConfig(
  pluginId: string,
  config: Record<string, unknown>,
): Promise<void> {
  return invokeWithFriendlyError<void>("set_plugin_settings_config", {
    pluginId,
    config,
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

/** 扁平 Tauri command 参数在 IPC 上为 camelCase（与 Rust 侧 `snake_case` 形参对应）。 */
export async function spawnPluginForTest(
  pluginId: string,
  configJson?: string | null,
): Promise<PluginProcessDebugInfo> {
  return invokeWithFriendlyError<PluginProcessDebugInfo>("spawn_plugin_for_test", {
    pluginId,
    configJson: configJson ?? null,
  });
}

export async function killPluginProcess(pluginId: string): Promise<void> {
  return invokeWithFriendlyError<void>("kill_plugin_process", { pluginId });
}

export async function listPluginProcesses(): Promise<PluginProcessDebugInfo[]> {
  return invokeWithFriendlyError<PluginProcessDebugInfo[]>("list_plugin_processes", {});
}

export async function getPluginLogs(
  pluginId: string,
  lines: number,
): Promise<string[]> {
  return invokeWithFriendlyError<string[]>("get_plugin_logs", {
    pluginId,
    lines,
  });
}

export async function clearPluginLogs(pluginId: string): Promise<void> {
  return invokeWithFriendlyError<void>("clear_plugin_logs", { pluginId });
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
    pluginId,
  });
}

export interface McpToolManifest {
  name: string;
  description?: string | null;
}

export interface McpServerManifest {
  id: string;
  name: string;
  transport?: string;
  url?: string | null;
  command?: string | null;
  args?: string[];
  tools?: McpToolManifest[];
}

export interface McpToolCallResult {
  server_id: string;
  tool_name: string;
  result: unknown;
}

export interface AgentToolCallTrace {
  server_id: string;
  tool_name: string;
  params: unknown;
  result: unknown;
}

export interface AgentDebugTrace {
  timestamp_ms: number;
  role_id: string;
  session_namespace: string;
  message: string;
  plan: string;
  tool_calls: AgentToolCallTrace[];
  reply: string;
  error?: string | null;
}

export async function listMcpServers(): Promise<McpServerManifest[]> {
  return invokeWithFriendlyError<McpServerManifest[]>("list_mcp_servers", {});
}

export async function listMcpTools(serverId: string): Promise<McpToolManifest[]> {
  return invokeWithFriendlyError<McpToolManifest[]>("list_mcp_tools", {
    req: { server_id: serverId },
  });
}

export async function callMcpTool(
  serverId: string,
  toolName: string,
  params: unknown = {},
): Promise<McpToolCallResult> {
  return invokeWithFriendlyError<McpToolCallResult>("call_mcp_tool", {
    req: {
      server_id: serverId,
      tool_name: toolName,
      params,
    },
  });
}

export async function getAgentDebugTraces(): Promise<AgentDebugTrace[]> {
  return invokeWithFriendlyError<AgentDebugTrace[]>("get_agent_debug_traces", {});
}

export async function clearAgentDebugTraces(): Promise<void> {
  return invokeWithFriendlyError<void>("clear_agent_debug_traces", {});
}

export interface McpServerImportPreview {
  serverId: string;
  name: string;
  transport: string;
  requiredPermission: string;
}

export async function previewMcpServerImport(
  path: string,
): Promise<McpServerImportPreview> {
  return invokeWithFriendlyError<McpServerImportPreview>("preview_mcp_server_import", {
    req: { path },
  });
}

export async function importMcpServerFromPath(
  path: string,
  grantRequiredPermission: boolean,
): Promise<McpServerImportPreview> {
  return invokeWithFriendlyError<McpServerImportPreview>(
    "import_mcp_server_from_path",
    {
      req: { path, grantRequiredPermission },
    },
  );
}

export interface CreatePluginScaffoldRequest {
  pluginId: string;
  pluginName: string;
  language: "node" | "python" | "rust";
  pluginType: "skill" | "agent" | "module_ext";
  baseDir?: string;
}

export interface CreatePluginScaffoldResponse {
  plugin_dir: string;
}

export async function createPluginScaffold(
  req: CreatePluginScaffoldRequest,
): Promise<CreatePluginScaffoldResponse> {
  return invokeWithFriendlyError<CreatePluginScaffoldResponse>(
    "create_plugin_scaffold",
    {
      req: {
        plugin_id: req.pluginId,
        plugin_name: req.pluginName,
        language: req.language,
        plugin_type: req.pluginType,
        base_dir: req.baseDir ?? null,
      },
    },
  );
}

export interface PackPluginResponse {
  archive_path: string;
  signature_path: string;
  sha256: string;
}

export async function packPlugin(
  pluginId: string,
  outputDir?: string | null,
): Promise<PackPluginResponse> {
  return invokeWithFriendlyError<PackPluginResponse>("pack_plugin", {
    req: {
      plugin_id: pluginId,
      output_dir: outputDir ?? null,
    },
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

/** 设置页 / 云端：不含密钥的公开字段（`get_host_cloud_llm_public`）。 */
export interface HostCloudLlmPublicDto {
  baseUrl: string;
  model?: string | null;
  timeoutMs?: number | null;
  hasApiKey: boolean;
}

/** 保存应用内云端 OpenAI 兼容配置（`set_host_cloud_llm`）。 */
export interface HostCloudLlmSaveDto {
  baseUrl: string;
  apiKey: string;
  model?: string | null;
  timeoutMs?: number | null;
}

export async function getHostCloudLlmPublic(): Promise<HostCloudLlmPublicDto> {
  return invokeWithFriendlyError<HostCloudLlmPublicDto>("get_host_cloud_llm_public", {});
}

export async function setHostCloudLlm(dto: HostCloudLlmSaveDto): Promise<void> {
  return invokeWithFriendlyError<void>("set_host_cloud_llm", { dto });
}

/** 全局对话模型 id（Ollama 与云端路径共用）。 */
export async function getHostChatModel(): Promise<string> {
  return invokeWithFriendlyError<string>("get_host_chat_model", {});
}

export async function setHostChatModel(model: string): Promise<void> {
  return invokeWithFriendlyError<void>("set_host_chat_model", { model });
}

/** 在系统文件管理器中打开 `roles/{roleId}/`（编辑 settings.json、manifest 等）。 */
export async function revealRolePackFolder(roleId: string): Promise<void> {
  return invokeWithFriendlyError<void>("reveal_role_pack_folder", { roleId });
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
