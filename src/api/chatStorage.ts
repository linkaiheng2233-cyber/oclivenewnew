import { invokeWithFriendlyError } from './helpers'

export interface SessionMeta {
  session_id: string
  role_id: string
  scene_id: string
  created_at: string
  updated_at: string
  message_count: number
  last_message_snippet: string
}

export interface StoredMessage {
  id: string
  session_id: string
  turn_index: number
  sender: string
  content: string
  metadata?: string | null
  created_at: string
}

export interface ChatSearchResult {
  session_id: string
  role_id: string
  scene_id: string
  message: StoredMessage
  highlight_snippet: string
}

export interface ChatExportResponse {
  content: string
  suggested_filename: string
  mime_type: string
  content_encoding?: string | null
}

export interface RoleChatStorageConfig {
  /** Role pack `config.json` → `chat_storage.backend` (`hybrid` | `file` | `sqlite`). */
  backend?: 'hybrid' | 'file' | 'sqlite' | null
  max_messages_per_session?: number | null
  auto_cleanup_days?: number | null
  auto_cleanup_max_sessions?: number | null
}

export interface AutoCleanupResult {
  sessions_deleted: number
  bytes_freed: number
}

export interface ImportChatMessage {
  role: string
  content: string
  timestamp: number
  id?: string | null
}

export interface ImportChatBucket {
  role_id: string
  scene_id: string
  session_id?: string | null
  messages: ImportChatMessage[]
}

export interface ImportChatBucketsResult {
  buckets_imported: number
  turns_imported: number
}

export interface SceneStorageStat {
  scene_id: string
  session_count: number
  total_size_bytes: number
  last_active?: string | null
}

export interface RoleStorageStat {
  role_id: string
  total_size_bytes: number
  scene_count: number
  last_active?: string | null
  scenes: SceneStorageStat[]
}

export interface DeleteChatsResult {
  sessions_deleted: number
  bytes_freed: number
}

export async function listChatSessions(
  roleId: string,
  sceneId: string,
  limit = 50,
  offset = 0,
): Promise<SessionMeta[]> {
  return invokeWithFriendlyError<SessionMeta[]>('list_chat_sessions', {
    roleId,
    sceneId,
    limit,
    offset,
  })
}

export async function fetchChatMessages(
  sessionId: string,
  limit = 500,
  offset = 0,
): Promise<StoredMessage[]> {
  return invokeWithFriendlyError<StoredMessage[]>('fetch_chat_messages', {
    sessionId,
    limit,
    offset,
  })
}

export async function migrateIndexeddbToBackend(
  buckets: ImportChatBucket[],
): Promise<ImportChatBucketsResult> {
  return invokeWithFriendlyError<ImportChatBucketsResult>(
    'migrate_indexeddb_to_backend',
    { buckets },
  )
}

export async function getChatStorageStats(): Promise<RoleStorageStat[]> {
  return invokeWithFriendlyError<RoleStorageStat[]>('get_chat_storage_stats', {})
}

export async function deleteRoleChats(roleId: string): Promise<DeleteChatsResult> {
  return invokeWithFriendlyError<DeleteChatsResult>('delete_role_chats', { roleId })
}

export async function deleteSceneChats(
  roleId: string,
  sceneId: string,
): Promise<DeleteChatsResult> {
  return invokeWithFriendlyError<DeleteChatsResult>('delete_scene_chats', {
    roleId,
    sceneId,
  })
}

export async function exportChatSession(
  sessionId: string,
  format: 'markdown' | 'json',
): Promise<ChatExportResponse> {
  return invokeWithFriendlyError<ChatExportResponse>('export_chat_session', {
    sessionId,
    format,
  })
}

export async function exportRoleChats(
  roleId: string,
  format: 'markdown' | 'json',
): Promise<ChatExportResponse> {
  return invokeWithFriendlyError<ChatExportResponse>('export_role_chats', {
    roleId,
    format,
  })
}

export async function searchChatMessages(
  query: string,
  roleId?: string | null,
  limit = 100,
  offset = 0,
): Promise<ChatSearchResult[]> {
  return invokeWithFriendlyError<ChatSearchResult[]>('search_chat_messages', {
    query,
    roleId: roleId ?? null,
    limit,
    offset,
  })
}

export async function deleteChatMessage(messageId: string): Promise<void> {
  return invokeWithFriendlyError<void>('delete_chat_message', { messageId })
}

export async function editChatMessage(
  messageId: string,
  newContent: string,
): Promise<void> {
  return invokeWithFriendlyError<void>('edit_chat_message', {
    messageId,
    newContent,
  })
}

export async function getRoleChatStorageConfig(
  roleId: string,
): Promise<RoleChatStorageConfig> {
  return invokeWithFriendlyError<RoleChatStorageConfig>(
    'get_role_chat_storage_config',
    { roleId },
  )
}

export async function saveRoleChatStorageConfig(
  roleId: string,
  config: RoleChatStorageConfig,
): Promise<void> {
  return invokeWithFriendlyError<void>('save_role_chat_storage_config_cmd', {
    roleId,
    config,
  })
}

export async function runChatAutoCleanup(roleId: string): Promise<AutoCleanupResult> {
  return invokeWithFriendlyError<AutoCleanupResult>('run_chat_auto_cleanup', { roleId })
}

export interface ReplayTarget {
  role_id: string
  scene_id?: string | null
  session_id?: string | null
}

export interface ReplayProgress {
  task_id: string
  percent: number
  processed_turns: number
  total_turns: number
  new_memories: number
  updated_memories: number
  skipped_memories: number
  done: boolean
  errors: string[]
}

export interface ReplayResult {
  total_turns: number
  new_memories: number
  updated_memories: number
  skipped_memories: number
  errors: string[]
}

export async function replayMemoryExtraction(
  source: 'session' | 'scene' | 'role',
  target: ReplayTarget,
): Promise<string> {
  return invokeWithFriendlyError<string>('replay_memory_extraction', { source, target })
}

export async function getReplayProgress(taskId: string): Promise<ReplayProgress> {
  return invokeWithFriendlyError<ReplayProgress>('get_replay_progress', { taskId })
}

export interface ChatStorageCapabilities {
  supports_search: boolean
  supports_replay: boolean
  supports_cleanup: boolean
}

export async function getChatStorageCapabilities(): Promise<ChatStorageCapabilities> {
  return invokeWithFriendlyError<ChatStorageCapabilities>('get_chat_storage_capabilities', {})
}
