/** Side-channel `voice.asr` — plugin custom event → host `send_message`. */
export const VOICE_ASR_PLUGIN_ID = 'com.oclive.voice.asr'
export const VOICE_ASR_SUBMIT_EVENT = 'com.oclive.voice.asr:submit'
/** Emitted from VoiceSettings after save; host invalidates cached voice config. */
export const VOICE_ASR_CONFIG_UPDATED_EVENT = 'com.oclive.voice.asr:config-updated'
/** First complete sentence during streaming reply — optional early TTS (Phase 6). */
export const VOICE_STREAM_SENTENCE_EVENT = 'com.oclive.voice:stream-sentence'

export interface VoiceAsrSubmitPayload {
  text?: string
  /** `send` (default) posts chat; `fill` writes the composer draft only. */
  mode?: 'send' | 'fill'
}
