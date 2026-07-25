/** Side-channel `voice.asr` — plugin custom event → host `send_message`. */
export const VOICE_ASR_PLUGIN_ID = 'com.oclive.voice.asr'
export const VOICE_ASR_SUBMIT_EVENT = 'com.oclive.voice.asr:submit'
/** Emitted from VoiceSettings after save; host invalidates cached voice config. */
export const VOICE_ASR_CONFIG_UPDATED_EVENT = 'com.oclive.voice.asr:config-updated'
/** Incremental speakable fragment during streaming reply — early TTS side channel. */
export const VOICE_STREAM_SENTENCE_EVENT = 'com.oclive.voice:stream-sentence'

export interface VoiceAsrSubmitPayload {
  text?: string
  /** `send` (default) posts chat; `fill` writes the composer draft only. */
  mode?: 'send' | 'fill'
  /** Per-transcription id used to make host submission idempotent. */
  submissionId?: string
}

const MAX_SEEN_VOICE_SUBMISSIONS = 128
const LEGACY_DUPLICATE_WINDOW_MS = 1_500

/** Guards both current id-bearing events and legacy Voice plugin duplicates. */
export class VoiceAsrSubmitDeduper {
  private readonly seenIds = new Set<string>()
  private lastLegacyKey = ''
  private lastLegacyAt = 0

  accept(payload: VoiceAsrSubmitPayload, now = Date.now()): boolean {
    const id = payload.submissionId?.trim()
    if (id) {
      if (this.seenIds.has(id))
        return false
      if (this.seenIds.size >= MAX_SEEN_VOICE_SUBMISSIONS) {
        const oldest = this.seenIds.values().next().value
        if (oldest)
          this.seenIds.delete(oldest)
      }
      this.seenIds.add(id)
      return true
    }

    const key = `${payload.mode === 'fill' ? 'fill' : 'send'}\u001F${payload.text?.trim() ?? ''}`
    if (key === this.lastLegacyKey && now - this.lastLegacyAt <= LEGACY_DUPLICATE_WINDOW_MS)
      return false
    this.lastLegacyKey = key
    this.lastLegacyAt = now
    return true
  }
}
