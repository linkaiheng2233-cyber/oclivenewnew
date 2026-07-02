/** Side-channel `voice.asr` — plugin custom event → host `send_message`. */
export const VOICE_ASR_SUBMIT_EVENT = 'com.oclive.voice.asr:submit'

export interface VoiceAsrSubmitPayload {
  text?: string
  /** `send` (default) posts chat; `fill` writes the composer draft only. */
  mode?: 'send' | 'fill'
}
