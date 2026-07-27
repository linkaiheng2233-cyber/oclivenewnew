import type { DisplayMetricsDto, RoleInfo } from './role'
import { invokeWithFriendlyError, toFriendlyErrorMessage } from './helpers'

export interface SendMessageRequest {
  role_id: string
  user_message: string
  scene_id?: string | null
  session_id?: string | null
  adult?: AdultInteractionRequest | null
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

export async function beginAdultStageGeneration(
  req: BeginAdultStageGenerationRequest,
): Promise<BeginAdultStageGenerationResponse> {
  return invokeWithFriendlyError<BeginAdultStageGenerationResponse>(
    'begin_adult_stage_generation',
    { req },
  )
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

function parseSseBlock(block: string): { eventName: string, data: string } {
  let eventName = 'message'
  const dataLines: string[] = []
  for (const line of block.split('\n')) {
    if (line.startsWith('event:'))
      eventName = line.slice(6).trim()
    else if (line.startsWith('data:'))
      dataLines.push(line.slice(5).trim())
  }
  return { eventName, data: dataLines.join('\n') }
}

export interface SendMessageStreamOptions {
  onToken?: (token: string, accumulated: string) => void
  signal?: AbortSignal
}

/** Stream chat tokens via kernel HTTP `POST /chat/stream` (SSE). */
export async function sendMessageStream(
  req: SendMessageRequest,
  options: SendMessageStreamOptions = {},
): Promise<SendMessageResponse> {
  const { getKernelConnectionStatus } = await import('./kernel')
  const status = await getKernelConnectionStatus()
  if (!status.healthy) {
    throw new Error('Kernel offline')
  }
  const rolePath = await invokeWithFriendlyError<string>('get_role_pack_path', {
    roleId: req.role_id,
  })
  const res = await fetch(`${status.baseUrl}/chat/stream`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      'accept': 'text/event-stream',
    },
    body: JSON.stringify({
      role_path: rolePath,
      message: req.user_message,
      scene_id: req.scene_id ?? null,
      session_id: req.session_id ?? null,
      adult: req.adult ?? null,
    }),
    signal: options.signal,
  })
  if (!res.ok) {
    const errText = await res.text()
    throw new Error(`stream HTTP ${res.status}: ${errText.slice(0, 400)}`)
  }
  const reader = res.body?.getReader()
  if (!reader)
    throw new Error('stream body unavailable')
  const decoder = new TextDecoder()
  let buffer = ''
  let accumulated = ''
  let finalResponse: SendMessageResponse | null = null
  while (true) {
    const { done, value } = await reader.read()
    if (done)
      break
    buffer += decoder.decode(value, { stream: true })
    let sep: number
    while ((sep = buffer.indexOf('\n\n')) !== -1) {
      const block = buffer.slice(0, sep)
      buffer = buffer.slice(sep + 2)
      const { eventName, data } = parseSseBlock(block)
      if (!data)
        continue
      if (eventName === 'token') {
        try {
          const token = JSON.parse(data).token ?? ''
          if (typeof token === 'string' && token.length > 0) {
            accumulated += token
            options.onToken?.(token, accumulated)
          }
        }
        catch {
          accumulated += data
          options.onToken?.(data, accumulated)
        }
      }
      else if (eventName === 'done') {
        try {
          const parsed = JSON.parse(data) as { data?: SendMessageResponse }
          finalResponse = parsed.data ?? (parsed as unknown as SendMessageResponse)
        }
        catch {
          throw new Error('stream done payload parse failed')
        }
      }
      else if (eventName === 'error') {
        throw new Error(toFriendlyErrorMessage(data.slice(0, 400)))
      }
    }
  }
  if (!finalResponse)
    throw new Error('stream ended without done event')
  return finalResponse
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
