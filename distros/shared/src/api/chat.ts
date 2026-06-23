import { invokeWithFriendlyError } from './helpers'
import type { RoleInfo } from './role'

export interface SendMessageRequest {
  role_id: string
  user_message: string
  scene_id?: string | null
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
  /** Favorability relation stage; from `role_runtime.relation_state` */
  relation_state: string
  reply: string
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
}


export interface MemoryItem {
  id: string
  role_id: string
  content: string
  memory_type: string
  timestamp: string
  importance: number
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

