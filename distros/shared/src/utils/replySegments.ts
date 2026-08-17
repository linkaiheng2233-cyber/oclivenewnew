/** Presentation helpers for the `reply_mode` side channel (mirrors backend `domain/reply_mode.rs`). */

export type ReplyModeKind = 'single' | 'burst'
export type ReplyModeStreaming = 'live' | 'batch'

export interface ReplyModeConfig {
  mode: ReplyModeKind
  segments: number
  separator: string
  delays_ms: number[]
  streaming: ReplyModeStreaming
}

export interface ReplyPresentationDto {
  segments: string[]
  delays_ms: number[]
}

export interface ReplySegmentDraft {
  text: string
  delayMs: number
}

/** Mirror the backend line-only separator rule so live streaming and history stay identical. */
export function splitReplyBySeparatorLine(
  raw: string,
  separator: string,
  maxSegments: number,
): string[] {
  const sep = separator.trim()
  if (!sep || maxSegments <= 1) {
    const whole = raw.replace(/\r\n/g, '\n').replace(/\r/g, '\n').trim()
    return whole ? [whole] : []
  }
  const normalized = raw.replace(/\r\n/g, '\n').replace(/\r/g, '\n')
  const segments: string[] = []
  let current = ''
  for (const line of normalized.split('\n')) {
    if (line.trim() === sep) {
      const segment = current.trim()
      if (segment)
        segments.push(segment)
      current = ''
    }
    else {
      current += (current ? '\n' : '') + line
    }
  }
  const tail = current.trim()
  if (tail)
    segments.push(tail)
  if (segments.length > maxSegments) {
    const overflow = segments.slice(maxSegments - 1).join('\n\n')
    segments.length = maxSegments - 1
    segments.push(overflow)
  }
  return segments
}

export function draftsFromPresentation(
  reply: string,
  presentation: ReplyPresentationDto | null | undefined,
): ReplySegmentDraft[] {
  if (!presentation || presentation.segments.length <= 1) {
    return reply.trim() ? [{ text: reply.trim(), delayMs: 0 }] : []
  }
  return presentation.segments.map((text, index) => ({
    text: text.trim(),
    delayMs: presentation.delays_ms[index] ?? 0,
  }))
}

export function segmentMessageIds(baseId: string, count: number): string[] {
  if (count <= 1)
    return [baseId]
  return Array.from({ length: count }, (_, index) => `${baseId}#s${index}`)
}
