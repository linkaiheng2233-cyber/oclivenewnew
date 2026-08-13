import type { VoiceRuntimeConfig } from './voiceAutoTtsConfig'

export interface VoiceLatencyTrace {
  submittedAtMs: number
  firstTextAtMs?: number
  firstSynthesisAtMs?: number
  firstAudioAtMs?: number
}

export interface SpeakJob {
  key: string
  text: string
  payload: { bot_emotion?: string, role_id?: string }
  streamId?: string
  cfg: VoiceRuntimeConfig
  directive: Record<string, unknown>
  forceRpc: boolean
}

export interface RpcSpeakResult {
  ok?: boolean
  audio_base64?: string
  audio_mime?: string
  reason?: string
  message?: string
}

export interface ActiveStreamLookahead {
  currentJobKey: string
  generation: number
}

export function resolveRoleTtsProfile(
  directive: Record<string, unknown> | undefined,
  globalProfile: string,
): string {
  const stamped = typeof directive?.synth_profile === 'string'
    ? directive.synth_profile.trim()
    : ''
  return stamped && stamped !== globalProfile ? stamped : globalProfile
}

export function speakJobKey(text: string, payload: SpeakJob['payload'], cfg: VoiceRuntimeConfig): string {
  return `${payload.role_id || ''}|${payload.bot_emotion || ''}|${cfg.tts_profile}|${text}`
}
