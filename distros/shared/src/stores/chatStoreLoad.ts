import type { StoredMessage } from '@oclive/shared/api/chatStorage'
import type { RoleSceneMessageMap } from '@oclive/shared/utils/chatMessageDb'
import type { ChatMessage } from './chatStore'
import { fetchChatMessages, getChatStorageStats, listChatSessions } from '@oclive/shared/api/chatStorage'
import {
  loadBucketFromIdb,
  loadMessageMapFromIdb,

  saveBucketToIdb,
} from '@oclive/shared/utils/chatMessageDb'
import { isChatStorageMigrated } from '@oclive/shared/utils/chatStorageMigration'
import { conversationSessionId } from '@oclive/shared/utils/conversationSessionId'
import {
  applyAssistantSplit,
} from '@oclive/shared/utils/roleplayReplySplit'

export function parseMessageTimestamp(iso?: string | null): number {
  if (!iso)
    return Date.now()
  const ms = Date.parse(iso)
  return Number.isFinite(ms) ? ms : Date.now()
}

/** Convert one durable chat row into the shared UI message shape. */
export function storedMessageToChatMessage(m: StoredMessage): ChatMessage {
  let emotion: string | undefined
  let replyIsFallback: boolean | undefined
  if (m.metadata) {
    try {
      const meta = JSON.parse(m.metadata) as Record<string, unknown>
      if (m.sender === 'assistant') {
        if (typeof meta.bot_emotion === 'string')
          emotion = meta.bot_emotion
        if (typeof meta.reply_is_fallback === 'boolean')
          replyIsFallback = meta.reply_is_fallback
      }
    }
    catch {
      /* ignore */
    }
  }
  const role = m.sender === 'assistant'
    ? 'assistant'
    : m.sender === 'user'
      ? 'user'
      : 'system'
  return {
    id: m.id,
    role,
    content: m.content,
    timestamp: parseMessageTimestamp(m.created_at),
    emotion,
    replyIsFallback,
  }
}

export function storedMessageIsHidden(m: StoredMessage): boolean {
  if (!m.metadata)
    return false
  try {
    const meta = JSON.parse(m.metadata) as Record<string, unknown>
    return meta.hidden === true
  }
  catch {
    return false
  }
}

/** Normalize every loaded assistant transcript so old adult history keeps two bubbles. */
export function splitAssistantMessages(messages: ChatMessage[]): ChatMessage[] {
  if (messages.length === 0)
    return messages
  for (let i = 0; i < messages.length; i++) {
    const m = messages[i]
    if (m?.role === 'assistant' && !m.aside)
      messages[i] = applyAssistantSplit(m)
  }
  return messages
}

/** Scene probe order when the narrative-resolved bucket is empty (cross-scene / IDB recovery). */
export function buildSceneLoadCandidates(
  primarySceneId: string,
  packScenes: string[],
  scenesWithSessions: string[] = [],
): string[] {
  const seen = new Set<string>()
  const out: string[] = []
  const push = (raw: string | undefined | null) => {
    const id = (raw ?? '').trim() || 'default'
    if (seen.has(id))
      return
    seen.add(id)
    out.push(id)
  }
  for (const s of scenesWithSessions)
    push(s)
  push(primarySceneId)
  for (const s of packScenes)
    push(s)
  push('home')
  push('default')
  return out
}

async function loadBucketFromIdbOrMap(
  messageMap: RoleSceneMessageMap,
  roleId: string,
  sid: string,
  previousLocal: ChatMessage[],
): Promise<ChatMessage[] | null> {
  const cached = await loadBucketFromIdb(roleId, sid)
  if (cached?.length) {
    return splitAssistantMessages(
      mergeMessagesFromServer(cached, previousLocal),
    )
  }
  const roleBucket = messageMap[roleId]
  if (roleBucket && !Array.isArray(roleBucket) && roleBucket[sid]?.length)
    return roleBucket[sid]!
  return null
}

/** Resolve backend `session_id` for a role×scene bucket; null when the scene has no session yet. */
export async function resolveChatSessionId(
  roleId: string,
  sceneId: string,
): Promise<string | null> {
  const sid = sceneId || 'default'
  try {
    const sessions = await listChatSessions(roleId, sid, 10, 0)
    if (sessions.length > 0)
      return sessions[0]!.session_id
    return null
  }
  catch {
    /* offline / API error: legacy default-scene namespace only */
    if (sid === 'default')
      return conversationSessionId(roleId, null)
    return null
  }
}

/** Scenes that have at least one backend session for `roleId` (storage stats). */
export async function listScenesWithBackendSessions(roleId: string): Promise<string[]> {
  try {
    const stats = await getChatStorageStats()
    const row = stats.find(s => s.role_id === roleId)
    if (!row)
      return []
    return row.scenes
      .filter(s => s.session_count > 0)
      .sort((a, b) => (b.last_active ?? '').localeCompare(a.last_active ?? ''))
      .map(s => s.scene_id)
  }
  catch {
    return []
  }
}

/**
 * Load the best non-empty bucket for a role: primary narrative scene first, then other
 * scenes with backend sessions, then pack scenes / home / default, then IDB index scan.
 */
export async function loadRoleSceneMessagesWithSceneFallback(
  messageMap: RoleSceneMessageMap,
  roleId: string,
  primarySceneId: string,
  packScenes: string[],
): Promise<string> {
  const scenesWithSessions = await listScenesWithBackendSessions(roleId)
  const candidates = buildSceneLoadCandidates(
    primarySceneId,
    packScenes,
    scenesWithSessions,
  )
  for (const sceneId of candidates) {
    const messages = await loadRoleSceneMessages(messageMap, roleId, sceneId)
    if (messages.length > 0)
      return sceneId
  }
  const idbMap = await loadMessageMapFromIdb()
  const idbRole = idbMap?.[roleId]
  if (idbRole && !Array.isArray(idbRole)) {
    let bestScene: string | null = null
    let bestCount = 0
    for (const [sceneId, msgs] of Object.entries(idbRole)) {
      if (msgs.length > bestCount) {
        bestCount = msgs.length
        bestScene = sceneId
      }
    }
    if (bestScene) {
      await loadRoleSceneMessages(messageMap, roleId, bestScene)
      return bestScene
    }
  }
  return primarySceneId
}

/** Merge server-fetched messages with local bucket, keeping optimistic rows absent from server. */
export function mergeMessagesFromServer(
  serverMessages: ChatMessage[],
  localMessages: ChatMessage[],
): ChatMessage[] {
  if (localMessages.length === 0)
    return serverMessages
  const serverIds = new Set(serverMessages.map(m => m.id))
  const localOnly = localMessages.filter(m => !serverIds.has(m.id))
  if (localOnly.length === 0)
    return serverMessages
  const merged = [...serverMessages, ...localOnly]
  merged.sort((a, b) => {
    const dt = a.timestamp - b.timestamp
    return dt !== 0 ? dt : a.id.localeCompare(b.id)
  })
  return merged
}

function writeBucket(
  messageMap: RoleSceneMessageMap,
  roleId: string,
  sid: string,
  messages: ChatMessage[],
): ChatMessage[] {
  if (!messageMap[roleId])
    messageMap[roleId] = {}
  messageMap[roleId]![sid] = messages
  return messages
}

/** Load messages for a role+scene bucket from backend (SQLite) with IDB fallback. */
export async function loadRoleSceneMessages(
  messageMap: RoleSceneMessageMap,
  roleId: string,
  sceneId: string,
): Promise<ChatMessage[]> {
  const sid = sceneId || 'default'
  const previousLocal = messageMap[roleId]?.[sid] ?? []
  const sessionId = await resolveChatSessionId(roleId, sid)
  if (!sessionId) {
    const fromIdb = await loadBucketFromIdbOrMap(
      messageMap,
      roleId,
      sid,
      previousLocal,
    )
    if (fromIdb) {
      writeBucket(messageMap, roleId, sid, fromIdb)
      return fromIdb
    }
    return writeBucket(messageMap, roleId, sid, previousLocal)
  }
  try {
    const stored = await fetchChatMessages(sessionId, 500, 0)
    const serverMessages = stored
      .filter(m =>
        (m.sender === 'user' || m.sender === 'assistant')
        && !storedMessageIsHidden(m),
      )
      .map(storedMessageToChatMessage)
    const messages = splitAssistantMessages(
      mergeMessagesFromServer(serverMessages, previousLocal),
    )
    writeBucket(messageMap, roleId, sid, messages)
    if (!isChatStorageMigrated())
      await saveBucketToIdb(roleId, sid, messages)
    return messages
  }
  catch (err) {
    console.warn('[chatStore] fetch_chat_messages failed; using IDB cache', err)
    const cached = await loadBucketFromIdb(roleId, sid)
    if (cached) {
      const messages = splitAssistantMessages(
        mergeMessagesFromServer(cached, previousLocal),
      )
      writeBucket(messageMap, roleId, sid, messages)
      return messages
    }
    if (previousLocal.length > 0)
      return previousLocal
    return []
  }
}
