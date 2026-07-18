/** Minimum graphemes before emitting a speakable chunk (avoid "嗯" alone). */
const MIN_CHUNK_CHARS = 3

/** If no punctuation yet, cap wait — start TTS after this many chars. */
const MAX_CHARS_WITHOUT_BREAK = 12

/** First chunk: emit sooner to reduce time-to-first-sound. */
const FIRST_MAX_CHARS_WITHOUT_BREAK = 8

const STRONG_BREAK = /^[\s\S]*?[。！？!?；;\n]/
const WEAK_BREAK = /^[\s\S]*?[，、：:]/

export interface SpeakableChunkOptions {
  /** First streaming chunk: lower char cap, weak punctuation exits earlier. */
  isFirst?: boolean
}

/**
 * Extract the earliest speakable fragment from streaming LLM text so TTS can
 * start before the full reply/sentence completes (text–voice sync).
 */
export function extractFirstSpeakableChunk(
  accumulated: string,
  options?: SpeakableChunkOptions,
): string | null {
  const isFirst = options?.isFirst ?? false
  const text = accumulated.trimStart()
  if (text.length < MIN_CHUNK_CHARS)
    return null

  const maxWithoutBreak = isFirst
    ? FIRST_MAX_CHARS_WITHOUT_BREAK
    : MAX_CHARS_WITHOUT_BREAK

  const candidates: string[] = []
  const strong = text.match(STRONG_BREAK)
  if (strong) {
    const chunk = strong[0].trim()
    if (chunk.length >= MIN_CHUNK_CHARS)
      candidates.push(chunk)
  }
  const weak = text.match(WEAK_BREAK)
  if (weak) {
    const chunk = weak[0].trim()
    if (isFirst && chunk.length >= MIN_CHUNK_CHARS) {
      return chunk
    }
    if (chunk.length >= MIN_CHUNK_CHARS)
      candidates.push(chunk)
  }
  if (candidates.length > 0)
    return candidates.reduce((shortest, cur) => (cur.length < shortest.length ? cur : shortest))

  if (text.length >= maxWithoutBreak)
    return text.slice(0, maxWithoutBreak).trim()

  if (text.length >= MIN_CHUNK_CHARS)
    return text

  return null
}

/**
 * Next speakable fragment after `fromIndex` in streaming accumulated text.
 * Returns the chunk and its exclusive end index in `accumulated`.
 */
export function extractSpeakableChunkFrom(
  accumulated: string,
  fromIndex: number,
  options?: SpeakableChunkOptions,
): { chunk: string, endIndex: number } | null {
  if (fromIndex >= accumulated.length)
    return null
  const chunk = extractFirstSpeakableChunk(accumulated.slice(fromIndex), options)
  if (!chunk)
    return null
  const slice = accumulated.slice(fromIndex)
  const trimmedStart = slice.length - slice.trimStart().length
  const searchFrom = fromIndex + trimmedStart
  const pos = accumulated.indexOf(chunk, searchFrom)
  if (pos < 0) {
    const slice = accumulated.slice(fromIndex)
    const local = slice.trimStart()
    const localPos = local.indexOf(chunk)
    if (localPos >= 0) {
      const endIndex = fromIndex + (slice.length - local.length) + localPos + chunk.length
      return { chunk, endIndex }
    }
    return { chunk, endIndex: Math.min(searchFrom + chunk.length, accumulated.length) }
  }
  return { chunk, endIndex: pos + chunk.length }
}

/** Text in `fullText` that was not covered by streamed speakable chunks. */
export function remainderAfterSpokenPrefix(fullText: string, spokenPrefix: string): string {
  const full = fullText.trim()
  const prefix = spokenPrefix.trim()
  if (!prefix)
    return full
  if (full.startsWith(prefix))
    return full.slice(prefix.length).trim()
  const idx = full.indexOf(prefix)
  if (idx >= 0)
    return full.slice(idx + prefix.length).trim()
  const normFull = full.replace(/\s+/g, ' ')
  const normPrefix = prefix.replace(/\s+/g, ' ')
  if (normPrefix && normFull.startsWith(normPrefix))
    return normFull.slice(normPrefix.length).trim()
  return full
}

/** @deprecated Use extractFirstSpeakableChunk; kept for grep/back-compat. */
export function extractFirstSentence(accumulated: string): string | null {
  const m = accumulated.match(STRONG_BREAK)
  return m ? m[0].trim() : null
}
