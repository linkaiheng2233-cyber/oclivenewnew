import { describe, expect, it } from 'vitest'
import {
  DEFAULT_COSYVOICE_EMO_TEXT,
  resolveStreamDirective,
  shouldUseBundledSidecarStream,
  shouldUseDirectSidecarStream,
} from './cosyvoiceStreamPlayback'

describe('cosyvoiceStreamPlayback', () => {
  it('fills default emo_text for instruct2 fast path', () => {
    expect(resolveStreamDirective(undefined).emo_text).toBe(DEFAULT_COSYVOICE_EMO_TEXT)
    expect(resolveStreamDirective({ ref_audio: '/tmp/x.wav' }).emo_text).toBeUndefined()
  })

  it('detects bundled synth provider', () => {
    expect(shouldUseBundledSidecarStream('bundled')).toBe(true)
    expect(shouldUseBundledSidecarStream('cloud')).toBe(false)
  })

  it('enables direct sidecar stream only for bundled cosyvoice2', () => {
    expect(shouldUseDirectSidecarStream('bundled', 'cosyvoice2')).toBe(true)
    expect(shouldUseDirectSidecarStream('bundled')).toBe(true)
    expect(shouldUseDirectSidecarStream('', 'cosyvoice2')).toBe(true)
    expect(shouldUseDirectSidecarStream('cloud')).toBe(false)
    expect(shouldUseDirectSidecarStream('bundled', 'gpt-sovits-http')).toBe(false)
    expect(shouldUseDirectSidecarStream('local_http', 'cosyvoice2')).toBe(false)
  })

  it('exports bounded stream timeout constants', async () => {
    const mod = await import('./cosyvoiceStreamPlayback')
    expect(mod.SIDECAR_STREAM_TIMEOUT_MS).toBeLessThanOrEqual(30_000)
    expect(mod.SIDECAR_STREAM_FIRST_CHUNK_TIMEOUT_MS).toBeLessThanOrEqual(20_000)
  })
})
