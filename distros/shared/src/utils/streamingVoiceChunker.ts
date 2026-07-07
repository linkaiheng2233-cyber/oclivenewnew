import {
  extractSpeakableChunkFrom,
} from '@oclive/shared/utils/extractFirstSpeakableChunk'
import { voiceDialogueFromRaw } from '@oclive/shared/utils/voiceDialogueFromRaw'

/** Truncate streaming text before an unclosed `（` or half-line `【…】`. */
export function stableStreamingPrefix(raw: string): string {
  let end = raw.length
  const lastOpen = raw.lastIndexOf('（')
  if (lastOpen >= 0) {
    const closeAfter = raw.indexOf('）', lastOpen)
    if (closeAfter < 0)
      end = lastOpen
  }
  const lastNl = raw.lastIndexOf('\n', Math.max(0, end - 1))
  const tailStart = lastNl >= 0 ? lastNl + 1 : 0
  const tail = raw.slice(tailStart, end)
  if (/^\s*【[^】]*$/.test(tail))
    end = tailStart
  return raw.slice(0, end)
}

/**
 * Stateful incremental extractor: filters roleplay aside/narration before
 * emitting speakable chunks (matches final `voiceDialogueFromRaw` semantics).
 */
export class StreamingVoiceChunker {
  private spokenDialogueLen = 0
  private spokenRawEndIndex = 0
  private emittedFirst = false

  /** Exclusive end index in the raw accumulated stream buffer. */
  get rawEndIndex(): number {
    return this.spokenRawEndIndex
  }

  push(accumulated: string): string[] {
    const stable = stableStreamingPrefix(accumulated)
    if (stable.length <= this.spokenRawEndIndex)
      return []
    const dialogue = voiceDialogueFromRaw(stable)
    const chunks: string[] = []
    while (true) {
      const next = extractSpeakableChunkFrom(
        dialogue,
        this.spokenDialogueLen,
        { isFirst: !this.emittedFirst },
      )
      if (!next)
        break
      this.spokenDialogueLen = next.endIndex
      this.emittedFirst = true
      chunks.push(next.chunk)
    }
    this.spokenRawEndIndex = stable.length
    return chunks
  }

  /** Final pass: emit remaining dialogue tail after stream completes. */
  flush(accumulated: string): string[] {
    const chunks = this.push(accumulated)
    const dialogue = voiceDialogueFromRaw(accumulated)
    if (this.spokenDialogueLen < dialogue.length) {
      const tail = dialogue.slice(this.spokenDialogueLen).trim()
      if (tail) {
        chunks.push(tail)
        this.spokenDialogueLen = dialogue.length
        this.emittedFirst = true
      }
    }
    this.spokenRawEndIndex = accumulated.length
    return chunks
  }
}
