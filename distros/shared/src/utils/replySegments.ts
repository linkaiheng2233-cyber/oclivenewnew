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

const SEPARATOR_TRAILING_PUNCTUATION = new Set([
  '。',
  '，',
  '！',
  '？',
  '…',
  '、',
  '.',
  ',',
  '!',
  '?',
  ';',
  '~',
])
const TRAILING_MARKER_PRECEDERS = new Set(['。', '！', '？', '!', '?', '…', '；', ';'])

function isSeparatorBoundary(line: string, separator: string): boolean {
  const trimmed = line.trim()
  if (trimmed === separator)
    return true
  if (!trimmed.startsWith(separator))
    return false
  const suffix = trimmed.slice(separator.length)
  return suffix.length > 0 && [...suffix].every(char => SEPARATOR_TRAILING_PUNCTUATION.has(char))
}

function trailingMarkerPrefix(line: string, separator: string): string | null {
  const trimmedEnd = line.trimEnd()
  if (!trimmedEnd.endsWith(separator))
    return null
  const prefix = trimmedEnd.slice(0, -separator.length).trimEnd()
  const last = prefix[prefix.length - 1]
  return prefix && last && TRAILING_MARKER_PRECEDERS.has(last) ? prefix : null
}

/** Mirror the backend's primary separator protocol for safe live presentation. */
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
    const prefix = trailingMarkerPrefix(line, sep)
    if (prefix !== null) {
      current += (current ? '\n' : '') + prefix
      const segment = current.trim()
      if (segment)
        segments.push(segment)
      current = ''
    }
    else if (isSeparatorBoundary(line, sep)) {
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
