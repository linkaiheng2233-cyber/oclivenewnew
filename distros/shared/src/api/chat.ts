import type { DisplayMetricsDto, RoleInfo } from './role'
import { Channel, invoke } from '@tauri-apps/api/core'
import { invokeWithFriendlyError, toFriendlyErrorMessage } from './helpers'

export interface SendMessageRequest {
  /** One logical user send; streaming and result recovery reuse this UUID. */
  client_request_id?: string | null
  role_id: string
  user_message: string
  scene_id?: string | null
  session_id?: string | null
  adult?: AdultInteractionRequest | null
}

/** Reference Host local converter output; this is not a universal package layout. */
export interface MinimalRoleLocalSource {
  role_id: string
  asset_root: string
  definition_reference: string
}

export interface MinimalRoleLocalMessageRequest {
  source: MinimalRoleLocalSource
  message: { user_message: string, requirements?: string }
  /** Completed turns of this temporary binding; never saved/rich memory IDs. */
  conversation?: readonly { user_message: string, reply: string }[]
}

/** No synthetic product metrics, stored row IDs, history or recovery guarantee. */
export interface MinimalRoleMessageResponse {
  role_id: string
  reply: string
  product_extensions: 'unavailable'
}

export type AdultInteractionAction = 'message' | 'continue' | 'exit'
export type AdultInteractionState = 'inactive' | 'active' | 'ended'

export interface AdultInteractionRequest {
  confirmed_adult: boolean
  global_enabled: boolean
  role_enabled: boolean
  interaction_active: boolean
  action?: AdultInteractionAction
  stage?: AdultStageDirective | null
}

export interface AdultStageDirective {
  generation_id: string
  sequence: number
}

export interface AdultBeatDto {
  dialogue: string
  narration: string
  interaction_state: AdultInteractionState
  next_beat_interval_ms?: number | null
}

export interface BeginAdultStageGenerationRequest {
  role_id: string
  scene_id?: string | null
  session_id?: string | null
  adult: AdultInteractionRequest
}

export interface BeginAdultStageGenerationResponse {
  generation_id: string
  next_sequence: number
}

export interface StageAdultBeatRequest {
  role_id: string
  scene_id?: string | null
  session_id?: string | null
  generation_id: string
  sequence: number
  adult: AdultInteractionRequest
}

export interface AdultStagedBeatDto {
  generation_id: string
  sequence: number
  response: SendMessageResponse
}

export interface CommitAdultStagedBeatRequest {
  role_id: string
  scene_id?: string | null
  session_id?: string | null
  generation_id: string
  sequence: number
}

export interface CancelAdultStageGenerationRequest {
  role_id: string
  scene_id?: string | null
  session_id?: string | null
  generation_id: string
}

export type ListAdultStagedBeatsRequest = CancelAdultStageGenerationRequest

export interface ListAdultStagedBeatsResponse {
  generation_id: string
  active: boolean
  next_sequence: number
  beats: AdultStagedBeatDto[]
}

export interface EmotionDto {
  joy: number
  sadness: number
  anger: number
  fear: number
  surprise: number
  disgust: number
  neutral: number
}

export interface DetectedEventDto {
  event_type: string
  confidence: number
}

export type PresenceMode = 'co_present' | 'remote_stub' | 'remote_life'

export interface ReplyPresentationDto {
  segments: string[]
  delays_ms: number[]
}

export interface SendMessageResponse {
  api_version: number
  schema: number
  /** co-present / remote stub / remote life */
  presence_mode: PresenceMode
  display_metrics?: DisplayMetricsDto | null
  /** @deprecated prefer `display_metrics.relation_summary` */
  relation_state: string
  reply: string
  adult_beat?: AdultBeatDto | null
  /** Optional segmented presentation for the `reply_mode` side channel */
  reply_presentation?: ReplyPresentationDto | null
  emotion: EmotionDto
  /** Bot emotion label (lowercase) for this turn */
  bot_emotion: string
  /** Portrait DB `current_emotion`; dialogue styling uses `bot_emotion` */
  portrait_emotion: string
  /** Closed-set catalog asset id when portrait_catalog.enabled */
  visual_state_id?: string | null
  /** Visual presentation facility render directive */
  performance_directive?: {
    visual_state_id: string
    kind: string
    path?: string | null
    fallback_image?: string | null
    live2d_model?: string | null
    rig3d_model?: string | null
    context?: string | null
  } | null
  favorability_delta: number
  favorability_current: number
  events: DetectedEventDto[]
  scene_id: string
  /** Frontend shows destination picker on movement intent; confirm via `switch_scene` */
  offer_destination_picker: boolean
  /** User invited character to travel together; confirm via `switch_scene` instead of scene-only switch */
  offer_together_travel: boolean
  /** Fallback short reply when primary LLM failed */
  reply_is_fallback?: boolean
  llm_fallback_reason?: string | null
  /** Knowledge chunks injected into Prompt this turn (remote stub placeholder is 0) */
  knowledge_chunks_in_prompt?: number
  timestamp: number
  user_message_id?: string | null
  assistant_message_id?: string | null
  user_message_timestamp?: string | null
  assistant_message_timestamp?: string | null
  /** True when CoPresent chat row persistence failed (SQLite authoritative store). */
  chat_persist_failed?: boolean | null
  /** Human-readable chat persistence error when `chat_persist_failed` is set. */
  chat_persist_error?: string | null
}

/** Identity dropdown sentinel for manifest default identity option; value is `OCLIVE_DEFAULT_RELATION_SENTINEL` */

export type SwitchSceneResponse = RoleInfo & {
  scene_welcome?: string | null
}

export interface TimeStateResponse {
  virtual_time_ms: number
  iso_datetime: string
}

export interface JumpTimeResponse {
  virtual_time_ms: number
  iso_datetime: string
  monologues: string[]
  favorability_delta: number
  favorability_current: number
  /** Autonomous scene switch after time jump: `current_scene` from → to */
  autonomous_scene_from?: string | null
  autonomous_scene_to?: string | null
}

export interface ExportChatLogsResponse {
  content: string
  suggested_filename: string
}

export interface QueryMemoriesRequest {
  role_id: string
  limit: number
  offset: number
  content_scope?: 'ordinary' | 'adult' | null
}

export interface MemoryItem {
  id: string
  role_id: string
  content: string
  memory_type: string
  timestamp: string
  importance: number
  content_scope: 'ordinary' | 'adult'
}

export interface QueryEventsRequest {
  role_id: string
  limit: number
  offset: number
}

export interface EventItem {
  id: number
  role_id: string
  event_type: string
  user_emotion?: string | null
  bot_emotion?: string | null
  timestamp: string
  description?: string | null
}

export interface CreateEventRequest {
  role_id: string
  event_type: string
  description?: string | null
}

export interface CreateEventResponse {
  id: number
  role_id: string
  event_type: string
  timestamp: string
  description?: string | null
}

export async function sendMessage(
  req: SendMessageRequest,
): Promise<SendMessageResponse> {
  return invokeWithFriendlyError<SendMessageResponse>('send_message', { req })
}

/** One basic call through the authenticated Rust client; never retries as rich chat. */
export async function sendMinimalMessage(
  req: MinimalRoleLocalMessageRequest,
): Promise<MinimalRoleMessageResponse> {
  return invokeWithFriendlyError<MinimalRoleMessageResponse>('send_minimal_message', { req })
}

export async function beginAdultStageGeneration(
  req: BeginAdultStageGenerationRequest,
): Promise<BeginAdultStageGenerationResponse> {
  return invokeWithFriendlyError<BeginAdultStageGenerationResponse>(
    'begin_adult_stage_generation',
    { req },
  )
}

/** Recover an existing send; never fall back to a command that starts a new turn. */
export async function recoverMessage(req: SendMessageRequest): Promise<SendMessageResponse> {
  try {
    return await invokeWithFriendlyError<SendMessageResponse>('recover_message', { req })
  }
  catch (error) {
    if (error instanceof Error && 'code' in error && error.code === 'CHAT_REQUEST_CONFLICT')
      throw error
    throw new Error(toFriendlyErrorMessage(JSON.stringify({ code: 'CHAT_REQUEST_UNCONFIRMED', message: 'Chat request outcome is unconfirmed' })), { cause: error })
  }
}

export async function generateAdultStagedBeat(
  req: StageAdultBeatRequest,
): Promise<AdultStagedBeatDto> {
  return invokeWithFriendlyError<AdultStagedBeatDto>(
    'generate_adult_staged_beat',
    { req },
  )
}

export async function commitAdultStagedBeat(
  req: CommitAdultStagedBeatRequest,
): Promise<SendMessageResponse> {
  return invokeWithFriendlyError<SendMessageResponse>(
    'commit_adult_staged_beat',
    { req },
  )
}

export async function cancelAdultStageGeneration(
  req: CancelAdultStageGenerationRequest,
): Promise<void> {
  return invokeWithFriendlyError<void>('cancel_adult_stage_generation', { req })
}

export async function listAdultStagedBeats(
  req: ListAdultStagedBeatsRequest,
): Promise<ListAdultStagedBeatsResponse> {
  return invokeWithFriendlyError<ListAdultStagedBeatsResponse>(
    'list_adult_staged_beats',
    { req },
  )
}

/**
 * Stream one ordinary chat turn through the desktop host's **authenticated** Rust transport.
 *
 * The loopback kernel requires `x-oclive-api-token` on `/chat/stream`, and only the Rust client holds
 * it. Calling this from a non-Tauri host is therefore unsupported by design rather than silently
 * falling back to an unauthenticated `fetch` (which the Host rejects with `401 KERNEL_AUTH_REQUIRED`).
 */
function assertAuthenticatedHost(): void {
  const internals = (globalThis as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__
  if (!internals) {
    throw new Error(
      'sendMessageStream requires the desktop host: /chat/stream is token-protected and only the Rust client holds the token',
    )
  }
}

function streamAbortError(): DOMException {
  return new DOMException('The operation was aborted.', 'AbortError')
}

export interface SendMessageStreamOptions {
  onToken?: (token: string, accumulated: string) => void
  signal?: AbortSignal
}

interface ChatStreamIpcEvent {
  kind: 'token' | 'canceled' | 'failed'
  token?: string
  accumulated?: string
  reason?: string
  code?: string
  message?: string
}

/**
 * Stream chat tokens for one turn over IPC; the command returns the authoritative DTO, which by
 * construction resolves strictly after the last token. The request keeps one
 * `client_request_id` across this stream and any later same-identity recovery.
 */
export async function sendMessageStream(
  req: SendMessageRequest,
  options: SendMessageStreamOptions = {},
): Promise<SendMessageResponse> {
  assertAuthenticatedHost()
  if (options.signal?.aborted)
    throw streamAbortError()

  // A per-stream transport handle for cancellation. Deliberately not `req.client_request_id`: that
  // identity belongs to the business turn, not to this renderer transport instance.
  const transportId = `chat-stream-${crypto.randomUUID()}`
  let cancelled = false
  const onAbort = () => {
    cancelled = true
    void invoke('cancel_message_stream', { transportId }).catch(() => {})
  }
  options.signal?.addEventListener('abort', onAbort, { once: true })

  const onEvent = new Channel<ChatStreamIpcEvent>()
  onEvent.onmessage = (event) => {
    // Late tokens must never reach an already-cancelled or already-recovered UI.
    if (cancelled || options.signal?.aborted)
      return
    if (event.kind === 'token' && typeof event.token === 'string')
      options.onToken?.(event.token, event.accumulated ?? '')
  }

  try {
    return await invokeWithFriendlyError<SendMessageResponse>('send_message_stream', {
      req,
      transportId,
      onEvent,
    })
  }
  catch (error) {
    if (cancelled || options.signal?.aborted)
      throw streamAbortError()
    throw error
  }
  finally {
    options.signal?.removeEventListener('abort', onAbort)
  }
}

export async function queryMemories(
  req: QueryMemoriesRequest,
): Promise<MemoryItem[]> {
  return invokeWithFriendlyError<MemoryItem[]>('query_memories', { req })
}

export async function queryEvents(req: QueryEventsRequest): Promise<EventItem[]> {
  return invokeWithFriendlyError<EventItem[]>('query_events', { req })
}

export async function createEvent(
  req: CreateEventRequest,
): Promise<CreateEventResponse> {
  return invokeWithFriendlyError<CreateEventResponse>('create_event', { req })
}

export async function reloadPolicyPlugins(): Promise<string> {
  return invokeWithFriendlyError<string>('reload_policy_plugins', {})
}

export async function switchScene(
  roleId: string,
  sceneId: string,
  /** `true`: write `current_scene` and co-present with role; `false`: update `user_presence_scene` only (solo narrative) */
  together: boolean = true,
): Promise<SwitchSceneResponse> {
  return invokeWithFriendlyError<SwitchSceneResponse>('switch_scene', {
    req: { role_id: roleId, scene_id: sceneId, together },
  })
}

export async function setUserPresenceScene(
  roleId: string,
  sceneId: string,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>('set_user_presence_scene', {
    req: { role_id: roleId, scene_id: sceneId },
  })
}

export async function getTimeState(roleId: string): Promise<TimeStateResponse> {
  return invokeWithFriendlyError<TimeStateResponse>('get_time_state', {
    roleId,
  })
}

export async function jumpTime(
  roleId: string,
  timestampMs?: number,
  preset?: '+2h' | '+6h' | 'next_morning' | 'skip_idle_time',
): Promise<JumpTimeResponse> {
  return invokeWithFriendlyError<JumpTimeResponse>('jump_time', {
    req: { role_id: roleId, timestamp_ms: timestampMs ?? null, preset: preset ?? null },
  })
}

export async function generateMonologue(roleId: string): Promise<string> {
  const res = await invokeWithFriendlyError<{ text: string }>(
    'generate_monologue',
    { req: { role_id: roleId } },
  )
  return res.text
}

/** `.ocpak` is ZIP packaging `roles/{id}/`; `.zip` with same layout also works; can export from extracted directory path */

export async function exportChatLogs(params: {
  roleId?: string
  allRoles?: boolean
  format: 'json' | 'txt'
  includePluginResolutionDebug?: boolean
  sessionId?: string | null
}): Promise<ExportChatLogsResponse> {
  return invokeWithFriendlyError<ExportChatLogsResponse>('export_chat_logs', {
    req: {
      role_id: params.roleId ?? null,
      all_roles: params.allRoles ?? false,
      format: params.format,
      include_plugin_resolution_debug: params.includePluginResolutionDebug ?? false,
      session_id: params.sessionId ?? null,
    },
  })
}

/** Main UI embed slots (`chat_toolbar` / `settings.panel`, etc.) from bootstrap DTO */

export interface PluginBridgeSendMessageParams {
  role_id: string
  user_message: string
  scene_id?: string | null
  session_id?: string | null
  /** Alias for `user_message` */
  text?: string
}
