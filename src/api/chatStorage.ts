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
