import type { StoredMessage } from '../api/chatStorage'
import { fetchChatMessages } from '../api/chatStorage'
import {
  assistantDialogueFromSplit,
  splitRoleplayReply,
} from '../utils/roleplayReplySplit'
import { conversationSessionId } from '../utils/conversationSessionId'
import {
  loadBucketFromIdb,
  saveBucketToIdb,
  type RoleSceneMessageMap,
} from '../utils/chatMessageDb'
import type { ChatMessage } from './chatStore'

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

/** Load messages for a role+scene bucket from backend (SQLite) with IDB fallback. */
export async function loadRoleSceneMessages(
  messageMap: RoleSceneMessageMap,
  roleId: string,
  sceneId: string,
): Promise<ChatMessage[]> {
  const sid = sceneId || 'default'
  const sessionId = conversationSessionId(roleId, null)
  try {
    const stored = await fetchChatMessages(sessionId, 500, 0)
    const messages = stored
      .filter(m => m.sender === 'user' || m.sender === 'assistant')
      .map(storedMessageToChatMessage)
    if (!messageMap[roleId])
      messageMap[roleId] = {}
    messageMap[roleId]![sid] = messages
    await saveBucketToIdb(roleId, sid, messages)
    return messages
  }
  catch (err) {
    console.warn('[chatStore] fetch_chat_messages failed; using IDB cache', err)
    const cached = await loadBucketFromIdb(roleId, sid)
    if (cached) {
      if (!messageMap[roleId])
        messageMap[roleId] = {}
      messageMap[roleId]![sid] = cached
      return cached
    }
    return []
  }
}
