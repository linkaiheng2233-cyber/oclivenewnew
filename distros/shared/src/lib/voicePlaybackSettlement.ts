export type VoicePlaybackSettlement = 'complete' | 'disabled' | 'error' | 'timeout'

const MAX_SETTLED = 128
const settled = new Map<string, VoicePlaybackSettlement>()
const waiters = new Map<string, Set<(status: VoicePlaybackSettlement) => void>>()

function trimSettled(): void {
  while (settled.size > MAX_SETTLED) {
    const first = settled.keys().next().value
    if (typeof first !== 'string')
      return
    settled.delete(first)
  }
}

export function markVoicePlaybackSettled(
  turnId: string,
  status: VoicePlaybackSettlement,
): void {
  const id = turnId.trim()
  if (!id)
    return
  settled.set(id, status)
  trimSettled()
  const pending = waiters.get(id)
  if (!pending)
    return
  waiters.delete(id)
  for (const resolve of pending)
    resolve(status)
}

export function waitForVoicePlaybackSettled(
  turnId: string,
  timeoutMs = 120_000,
): Promise<VoicePlaybackSettlement> {
  const id = turnId.trim()
  const known = settled.get(id)
  if (known) {
    settled.delete(id)
    return Promise.resolve(known)
  }
  return new Promise((resolve) => {
    const pending = waiters.get(id) ?? new Set()
    let finished = false
    let timer: ReturnType<typeof setTimeout> | undefined
    const finish = (status: VoicePlaybackSettlement) => {
      if (finished)
        return
      finished = true
      if (timer)
        clearTimeout(timer)
      pending.delete(finish)
      if (pending.size === 0)
        waiters.delete(id)
      settled.delete(id)
      resolve(status)
    }
    pending.add(finish)
    waiters.set(id, pending)
    timer = globalThis.setTimeout(finish, timeoutMs, 'timeout')
  })
}

export function clearVoicePlaybackSettlements(): void {
  settled.clear()
  for (const pending of waiters.values()) {
    for (const resolve of pending)
      resolve('disabled')
  }
  waiters.clear()
}
