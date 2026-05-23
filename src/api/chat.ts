import { invoke } from '@tauri-apps/api/tauri'
import { invokeWithFriendlyError } from './helpers'

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
  /** 共景 / 异地占位 / 异地心声 */
  presence_mode: PresenceMode
  /** 本回合结束后的关系阶段（�?`role_runtime.relation_state` 一致） */
  relation_state: string
  reply: string
  emotion: EmotionDto
  /** 本回�?bot 情绪标签（小写英文） */
  bot_emotion: string
  /** 立绘用（�?DB current_emotion 一致）；对话语气见 bot_emotion */
  portrait_emotion: string
  favorability_delta: number
  favorability_current: number
  events: DetectedEventDto[]
  scene_id: string
  /** 后端判定用户有前往/位移意图时置 true；实际切换仅通过 switch_scene */
  offer_destination_picker: boolean
  /** 检测到「一起去/跟我来」等邀请同行语义时�?true；确认后 `switch_scene`（同行）或仅叙事切换 */
  offer_together_travel: boolean
  /** �?LLM 失败时使用备用短回复 */
  reply_is_fallback?: boolean
  /** 本回合注�?Prompt 的知识块条数（共�?异地心声；占位为 0�?*/
  knowledge_chunks_in_prompt?: number
  timestamp: number
}

/** 身份下拉里「跟�?manifest 默认身份」选项的值（与后�?`OCLIVE_DEFAULT_RELATION_SENTINEL` 一致） */

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
  /** 虚拟时间规则是否将角�?current_scene �?from 切到 to */
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
  /** `true`：角色与用户同场景；`false`：仅更新用户叙事场景 */
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

/** `.ocpak`：ZIP 打包�?`roles/{id}/` 目录（与 `.zip` 相同容器；亦可导入已解压目录路径�?*/

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

/** 嵌入主界面插槽（`chat_toolbar` / `settings.panel`），�?bootstrap 返回�?*/

export interface PluginBridgeSendMessageParams {
  role_id: string
  user_message: string
  scene_id?: string | null
  session_id?: string | null
  /** �?`user_message` 二选一 */
  text?: string
}

