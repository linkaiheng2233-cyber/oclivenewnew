/** Minimum graphemes before emitting a speakable chunk (avoid "嗯" alone). */
const MIN_CHUNK_CHARS = 3

/**
 * Speech boundaries shared by CJK, Latin, Arabic and Indic writing systems.
 * Punctuation remains attached to the preceding chunk so the TTS model can
 * render the intended prosody.
 */
const BREAK_CHARS = '。！？!?；;，、：:,.…\n؟؛،۔।॥｡､'

function isAsciiDigit(char: string | undefined): boolean {
  return char != null && char >= '0' && char <= '9'
}

function isNaturalBreak(text: string, index: number): boolean {
  const char = text[index]
  if (!BREAK_CHARS.includes(char))
    return false
  // Do not split decimal/grouped numbers. When the right-hand digit has not
  // arrived yet, wait for one more streaming token before deciding.
  if (
    (char === '.' || char === ',')
    && isAsciiDigit(text[index - 1])
    && (text[index + 1] == null || isAsciiDigit(text[index + 1]))
  ) {
    return false
  }
  return true
}

function firstValidBreak(text: string): string | null {
  for (let index = 0; index < text.length; index++) {
    if (!isNaturalBreak(text, index))
      continue
    let end = index + 1
    while (end < text.length && isNaturalBreak(text, end))
      end += 1
    const chunk = text.slice(0, end).trim()
    // Skip a leading one- or two-character clause ("喂，") and keep looking
    // for the next delimiter instead of splitting inside the following word.
    if (chunk.length >= MIN_CHUNK_CHARS)
      return chunk
  }
  return null
}

export interface SpeakableChunkOptions {
  /** Kept for source compatibility; all chunks now use natural boundaries. */
  isFirst?: boolean
}

/**
 * Extract the earliest speakable fragment from streaming LLM text so TTS can
 * start before the full reply/sentence completes (text–voice sync).
 */
export function extractFirstSpeakableChunk(
  accumulated: string,
  _options?: SpeakableChunkOptions,
): string | null {
  const text = accumulated.trimStart()
  if (text.length < MIN_CHUNK_CHARS)
    return null

  return firstValidBreak(text)
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
  return firstValidBreak(accumulated)
}
