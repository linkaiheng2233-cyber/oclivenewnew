import { invoke } from '@tauri-apps/api/tauri'
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
  /** �?��?� / �?�?�占位 / �?�?��?声 */
  presence_mode: PresenceMode
  /** �?��??�?�?�?�?�??�?�系�?�段�?�?`role_runtime.relation_state` �?�?��? */
  relation_state: string
  reply: string
  emotion: EmotionDto
  /** �?��??�?bot �??绪�?签�?小�??�?��??�? */
  bot_emotion: string
  /** �?�?�?��?�?DB current_emotion �?�?��?�?对话语�?见 bot_emotion */
  portrait_emotion: string
  favorability_delta: number
  favorability_current: number
  events: DetectedEventDto[]
  scene_id: string
  /** �?端�?��?�?��?��??�?��?/位移�?��?��?�置 true�?�?�??�??换�?�??�? switch_scene */
  offer_destination_picker: boolean
  /** �?�?�?��??�?起�?�/�?�??来�?��?�??请�?�?语�?�?��?true�?确认�? `switch_scene`�?�?�?�?�??�?�?�?�??换 */
  offer_together_travel: boolean
  /** �?LLM 失败�?�使�?��?�?��?��??复 */
  reply_is_fallback?: boolean
  llm_fallback_reason?: string | null
  /** �?��??�?注�??Prompt �??�?��?�?条�?��?�?��??�?�?��?声�?占位为 0�?*/
  knowledge_chunks_in_prompt?: number
  timestamp: number
  user_message_id?: string | null
  assistant_message_id?: string | null
  user_message_timestamp?: string | null
  assistant_message_timestamp?: string | null
}

/** 身份�?�??�??�??�?�??manifest �?认身份�?��??项�??�?��?�?�?�?`OCLIVE_DEFAULT_RELATION_SENTINEL` �?�?��? */

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
  /** �??�??�?��?��?�??�?�否�?�?�??current_scene �?from �??�?� to */
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
  /** `true`�?�?�?��?�?��?��?�?��?��?`false`�?�?�?��?��?��?��?�?�?��?� */
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

/** `.ocpak`�?ZIP �??�??�??`roles/{id}/` �?��?�?�? `.zip` �?��?容�?��?亦可导�?�已解�??�?��?路�?�?*/

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

/** �?�?�主�??面�?槽�?`chat_toolbar` / `settings.panel`�?�?�??bootstrap �?�??�??*/

export interface PluginBridgeSendMessageParams {
  role_id: string
  user_message: string
  scene_id?: string | null
  session_id?: string | null
  /** �?`user_message` �?�??�? */
  text?: string
}

