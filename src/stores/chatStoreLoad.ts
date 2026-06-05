import type { StoredMessage } from '../api/chatStorage'
import type { RoleSceneMessageMap } from '../utils/chatMessageDb'
import type { ChatMessage } from './chatStore'
import { fetchChatMessages, listChatSessions } from '../api/chatStorage'
import {
  loadBucketFromIdb,

  saveBucketToIdb,
} from '../utils/chatMessageDb'
import { isChatStorageMigrated } from '../utils/chatStorageMigration'
import { conversationSessionId } from '../utils/conversationSessionId'
import {
  assistantDialogueFromSplit,
  splitRoleplayReply,
} from '../utils/roleplayReplySplit'

export function parseMessageTimestamp(iso?: string | null): number {
  if (!iso)
    return Date.now()
  const ms = Date.parse(iso)
  return Number.isFinite(ms) ? ms : Date.now()
}

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
  const base: ChatMessage = {
    id: m.id,
    role,
    content: m.content,
    timestamp: parseMessageTimestamp(m.created_at),
    emotion,
    replyIsFallback,
  }
  if (role === 'assistant') {
    const split = splitRoleplayReply(m.content)
    return {
      ...base,
      content: assistantDialogueFromSplit(m.content, split),
      ...(split.aside.trim() ? { aside: split.aside.trim() } : {}),
    }
  }
  return base
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
    const empty: ChatMessage[] = []
    return writeBucket(messageMap, roleId, sid, empty)
  }
  try {
    const stored = await fetchChatMessages(sessionId, 500, 0)
    const serverMessages = stored
      .filter(m => m.sender === 'user' || m.sender === 'assistant')
      .map(storedMessageToChatMessage)
    const messages = mergeMessagesFromServer(serverMessages, previousLocal)
    writeBucket(messageMap, roleId, sid, messages)
    if (!isChatStorageMigrated())
      await saveBucketToIdb(roleId, sid, messages)
    return messages
  }
  catch (err) {
    console.warn('[chatStore] fetch_chat_messages failed; using IDB cache', err)
    const cached = await loadBucketFromIdb(roleId, sid)
    if (cached) {
      const messages = mergeMessagesFromServer(cached, previousLocal)
      writeBucket(messageMap, roleId, sid, messages)
      return messages
    }
    if (previousLocal.length > 0)
      return previousLocal
    return []
  }
}
