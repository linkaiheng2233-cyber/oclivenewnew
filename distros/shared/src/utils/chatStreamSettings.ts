const CHAT_STREAM_ENABLED_KEY = 'oclive.chat.streamEnabled'

/** Whether Chat Pro uses SSE `/chat/stream` (default true). */
export function isChatStreamEnabled(): boolean {
  if (typeof localStorage === 'undefined')
    return true
  const raw = localStorage.getItem(CHAT_STREAM_ENABLED_KEY)
  if (raw === null)
    return true
  return raw !== 'false'
}

export function setChatStreamEnabled(enabled: boolean): void {
  if (typeof localStorage === 'undefined')
    return
  localStorage.setItem(CHAT_STREAM_ENABLED_KEY, enabled ? 'true' : 'false')
}
