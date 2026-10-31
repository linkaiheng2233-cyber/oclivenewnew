import type { SessionMeta } from '@oclive/shared/api/chatStorage'

const canonicalUuid = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/

/** Only transcript IDs issued in this role's namespace are eligible for continuation. */
export function chatThreadForSession(roleId: string, session: SessionMeta): string | null {
  if (session.role_id !== roleId)
    throw new Error('Conversation does not belong to this role')
  if (session.session_id === roleId)
    return null
  const suffix = `:chat:${roleId}`
  const thread = session.session_id.endsWith(suffix) ? session.session_id.slice(0, -suffix.length) : ''
  if (!canonicalUuid.test(thread) || thread === '00000000-0000-0000-0000-000000000000')
    throw new Error('This historical conversation uses a separate runtime; open it from data and records')
  return thread
}

export function chatThreadStorageId(roleId: string, threadId: string | null): string {
  return threadId ? `${threadId}:chat:${roleId}` : roleId
}
