import { describe, expect, it } from 'vitest'
import { formatVoiceSpeakFailure, shouldFallbackStreamToRpc } from './voiceSpeakErrors'

describe('voiceSpeakErrors', () => {
  it('formats stream timeout in Chinese', () => {
    expect(formatVoiceSpeakFailure('stream', { reason: 'stream_timeout' }))
      .toContain('流式')
    expect(formatVoiceSpeakFailure('stream', { reason: 'stream_timeout' }))
      .toContain('超时')
  })

  it('allows RPC fallback for stream failures', () => {
    expect(shouldFallbackStreamToRpc({ ok: false, reason: 'stream_timeout' })).toBe(true)
    expect(shouldFallbackStreamToRpc({ ok: true })).toBe(false)
    expect(shouldFallbackStreamToRpc({ ok: false, reason: 'tts_expansion_disabled' })).toBe(false)
  })

  it('routes stream GPU denial through the coordinated RPC fallback', () => {
    expect(shouldFallbackStreamToRpc({
      ok: false,
      reason: 'gpu_admission_denied',
    })).toBe(true)
    expect(formatVoiceSpeakFailure('rpc', {
      reason: 'gpu_admission_denied',
      message: 'headroom below threshold',
    })).toContain('显存安全余量不足')
  })
})
