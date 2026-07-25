import type {
  CosyvoiceStreamPrefetch,
  CosyvoiceStreamResult,
} from './cosyvoiceStreamPlayback'
import { describe, expect, it, vi } from 'vitest'
import {
  CosyvoiceStreamPrefetchRegistry,
  DEFAULT_COSYVOICE_EMO_TEXT,
  playCosyvoiceSidecarStream,
  resolveStreamDirective,
  shouldUseBundledSidecarStream,
  shouldUseDirectSidecarStream,
} from './cosyvoiceStreamPlayback'

function fakePrefetch(
  key: string,
  abort = vi.fn(),
): CosyvoiceStreamPrefetch {
  return {
    key,
    chunks: [],
    done: Promise.resolve({ ok: true, chunks: 1 }),
    waitForChunk: async () => {},
    abort,
  }
}

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

  it('waits for and consumes an in-flight prefetch instead of duplicating it', async () => {
    const registry = new CosyvoiceStreamPrefetchRegistry()
    let resolvePending: (prefetch: CosyvoiceStreamPrefetch) => void = () => {}
    const pending = new Promise<CosyvoiceStreamPrefetch>((resolve) => {
      resolvePending = resolve
    })
    registry.setPending('segment-2', pending)

    const taken = registry.take('segment-2')
    const prefetch = fakePrefetch('segment-2')
    registry.setReady('segment-2', prefetch)
    resolvePending(prefetch)
    registry.clearPending('segment-2', pending)

    await expect(taken).resolves.toBe(prefetch)
    expect(registry.readyFor('segment-2')).toBeUndefined()
    expect(registry.busy).toBe(false)
  })

  it('keeps a completed prefetch until its queue item consumes it', async () => {
    const registry = new CosyvoiceStreamPrefetchRegistry()
    const prefetch = fakePrefetch('segment-2')
    registry.setReady('segment-2', prefetch)

    await prefetch.done
    expect(registry.readyFor('segment-2')).toBe(prefetch)
    await expect(registry.take('segment-2')).resolves.toBe(prefetch)
  })

  it('aborts a pending prefetch when its queue item is cancelled', async () => {
    const registry = new CosyvoiceStreamPrefetchRegistry()
    const abort = vi.fn()
    let resolvePending: (prefetch: CosyvoiceStreamPrefetch) => void = () => {}
    const pending = new Promise<CosyvoiceStreamPrefetch>((resolve) => {
      resolvePending = resolve
    })
    registry.setPending('segment-2', pending)
    registry.cancel('segment-2')

    resolvePending(fakePrefetch('segment-2', abort))
    await pending
    await Promise.resolve()
    expect(abort).toHaveBeenCalledOnce()
    expect(registry.busy).toBe(false)
  })

  it('waits for a chunk beyond the consumed count without microtask spinning', async () => {
    const chunks = [{ pcm_base64: 'AA==', sample_rate: 22050 }]
    let releaseChunkWait = () => {}
    let finishStream: (result: CosyvoiceStreamResult) => void = () => {}
    const done = new Promise<CosyvoiceStreamResult>((resolve) => {
      finishStream = resolve
    })
    const waitForChunk = vi.fn((consumedCount: number) => {
      if (consumedCount !== chunks.length)
        return Promise.reject(new Error(`invalid consumed count: ${consumedCount}`))
      return new Promise<void>((resolve) => {
        releaseChunkWait = resolve
      })
    })
    const prefetch: CosyvoiceStreamPrefetch = {
      key: 'segment-1',
      chunks,
      done,
      waitForChunk,
      abort: vi.fn(),
    }

    const playback = playCosyvoiceSidecarStream(
      'http://127.0.0.1:50000',
      '你好呀',
      undefined,
      prefetch,
    )
    await vi.waitFor(() => {
      expect(waitForChunk).toHaveBeenCalledWith(1)
      expect(waitForChunk).toHaveBeenCalledTimes(1)
    })

    chunks.push({ pcm_base64: 'AA==', sample_rate: 22050 })
    releaseChunkWait()
    finishStream({ ok: true, chunks: 2 })

    await expect(playback).resolves.toMatchObject({ ok: true, chunks: 2 })
  })
})
