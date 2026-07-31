/** Mirrors Rust `conversation_state_role_id` (manifest id or `role__sess__*` namespace). */
const MAX_SUFFIX_CHARS = 64
const MAX_TOTAL_CHARS = 256

export function conversationSessionId(
  manifestRoleId: string,
  sessionId?: string | null,
): string {
  const sid = sessionId?.trim()
  if (!sid)
    return manifestRoleId.slice(0, MAX_TOTAL_CHARS)
  const safe = sid
    .replace(/[^\w-]/g, '_')
    .slice(0, MAX_SUFFIX_CHARS)
  const out = `${manifestRoleId}__sess__${safe}`
  return out.slice(0, MAX_TOTAL_CHARS)
}
