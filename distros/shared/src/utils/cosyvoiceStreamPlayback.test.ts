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

  it('enables direct sidecar stream for bundled provider (including Tauri)', () => {
    expect(shouldUseDirectSidecarStream('bundled')).toBe(true)
    expect(shouldUseDirectSidecarStream('')).toBe(true)
    expect(shouldUseDirectSidecarStream('cloud')).toBe(false)
  })

  it('exports bounded stream timeout constants', async () => {
    const mod = await import('./cosyvoiceStreamPlayback')
    expect(mod.SIDECAR_STREAM_TIMEOUT_MS).toBeLessThanOrEqual(30_000)
    expect(mod.SIDECAR_STREAM_FIRST_CHUNK_TIMEOUT_MS).toBeLessThanOrEqual(20_000)
  })
})
