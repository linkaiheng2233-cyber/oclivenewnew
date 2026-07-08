const DEFAULT_SIDECAR = 'http://127.0.0.1:50000'
/** Abort sidecar stream fetch if no response within this window (fallback to RPC). */
export const SIDECAR_STREAM_TIMEOUT_MS = 30_000
/** Stop waiting for first PCM chunk during prefetch playback (fallback to RPC). */
export const SIDECAR_STREAM_FIRST_CHUNK_TIMEOUT_MS = 20_000
/** Max time to wait for prefetch playback loop before RPC fallback. */
export const SIDECAR_STREAM_PLAYBACK_TIMEOUT_MS = 45_000
/** Max time to read NDJSON body after HTTP headers arrive. */
export const SIDECAR_STREAM_BODY_READ_TIMEOUT_MS = 60_000
/** CosyVoice instruct2 fallback when director directive is not ready yet. */
export const DEFAULT_COSYVOICE_EMO_TEXT = '用自然平静的语气'

export type CosyvoiceDirective = {
  emo_text?: string
  ref_audio?: string
  ref_text?: string
  speed?: number
}

export type CosyvoiceStreamResult = {
  ok: boolean
  reason?: string
  message?: string
  chunks?: number
  ttfc_ms?: number
  elapsed_ms?: number
}

function logVoiceStreamTelemetry(result: CosyvoiceStreamResult): void {
  if (!import.meta.env.DEV || !result.ok)
    return
  if (result.ttfc_ms == null && result.elapsed_ms == null)
    return
  console.debug('[voice-tts] stream telemetry', {
    ttfc_ms: result.ttfc_ms,
    elapsed_ms: result.elapsed_ms,
    chunks: result.chunks,
  })
}

type PcmChunk = {
  pcm_base64: string
  sample_rate: number
}

type NdjsonEvent = {
  ok?: boolean
  event?: string
  reason?: string
  message?: string
  pcm_base64?: string
  sample_rate?: number
  chunks?: number
  ttfc_ms?: number
  elapsed_ms?: number
}

let sharedAudioContext: AudioContext | null = null

/** Resume Web Audio on user-adjacent paths so first auto-TTS chunk plays without delay. */
export async function ensureVoiceAudioReady(): Promise<void> {
  if (typeof window === 'undefined')
    return
  if (!sharedAudioContext)
    sharedAudioContext = new AudioContext()
  if (sharedAudioContext.state === 'suspended')
    await sharedAudioContext.resume().catch(() => {})
}

class PcmStreamScheduler {
  private nextStart = 0

  schedulePcm16(pcmBase64: string, sampleRate: number): void {
    const ctx = sharedAudioContext
    if (!ctx)
      return
    const binary = atob(pcmBase64)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++)
      bytes[i] = binary.charCodeAt(i)
    const int16 = new Int16Array(bytes.buffer, bytes.byteOffset, bytes.byteLength / 2)
    const float32 = new Float32Array(int16.length)
    for (let i = 0; i < int16.length; i++)
      float32[i] = int16[i] / 32768
    const buffer = ctx.createBuffer(1, float32.length, sampleRate)
    buffer.copyToChannel(float32, 0)
    const source = ctx.createBufferSource()
    source.buffer = buffer
    source.connect(ctx.destination)
    const now = ctx.currentTime
    const start = Math.max(now + 0.005, this.nextStart)
    source.start(start)
    this.nextStart = start + buffer.duration
  }

  async waitUntilFinished(): Promise<void> {
    const ctx = sharedAudioContext
    if (!ctx)
      return
    const waitMs = Math.max(0, (this.nextStart - ctx.currentTime) * 1000)
    if (waitMs > 0)
      await new Promise(resolve => window.setTimeout(resolve, waitMs))
  }

  resetSchedule(): void {
    const ctx = sharedAudioContext
    this.nextStart = ctx ? Math.max(ctx.currentTime, this.nextStart) : 0
  }
}

function sidecarStreamUrl(endpoint: string): string {
  const base = (endpoint || DEFAULT_SIDECAR).trim().replace(/\/+$/, '')
  return `${base}/synthesize/stream`
}

async function fetchSidecarStream(
  endpoint: string,
  text: string,
  directive?: CosyvoiceDirective | null,
): Promise<Response> {
  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), SIDECAR_STREAM_TIMEOUT_MS)
  try {
    return await fetch(sidecarStreamUrl(endpoint), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json; charset=utf-8' },
      body: JSON.stringify(buildStreamPayload(text, directive)),
      signal: controller.signal,
    })
  }
  finally {
    clearTimeout(timer)
  }
}

function streamFetchError(err: unknown): CosyvoiceStreamResult {
  const msg = err instanceof Error ? err.message : String(err)
  const aborted = err instanceof Error && err.name === 'AbortError'
  const readTimeout = msg === 'ndjson_read_timeout'
  return {
    ok: false,
    reason: aborted ? 'stream_timeout' : readTimeout ? 'stream_read_failed' : 'stream_read_failed',
    message: readTimeout ? 'Sidecar stream body read timed out' : msg,
  }
}

export function resolveStreamDirective(
  directive?: CosyvoiceDirective | null,
): CosyvoiceDirective {
  const d = directive ?? {}
  const emo = typeof d.emo_text === 'string' ? d.emo_text.trim() : ''
  const ref = typeof d.ref_audio === 'string' ? d.ref_audio.trim() : ''
  if (emo || ref)
    return d
  return { ...d, emo_text: DEFAULT_COSYVOICE_EMO_TEXT }
}

function buildStreamPayload(text: string, directive?: CosyvoiceDirective | null) {
  const d = resolveStreamDirective(directive)
  return {
    text: text.trim(),
    emo_text: typeof d.emo_text === 'string' ? d.emo_text : '',
    ref_audio: typeof d.ref_audio === 'string' ? d.ref_audio : '',
    ref_text: typeof d.ref_text === 'string' ? d.ref_text : '',
    speed: typeof d.speed === 'number' ? d.speed : 1.0,
  }
}

function parseNdjsonLine(line: string): NdjsonEvent | null {
  const trimmed = line.trim()
  if (!trimmed)
    return null
  return JSON.parse(trimmed) as NdjsonEvent
}

async function readNdjsonStream(
  body: ReadableStream<Uint8Array>,
  onEvent: (evt: NdjsonEvent) => void,
  options?: { deadlineMs?: number, onTimeout?: () => void },
): Promise<void> {
  const reader = body.getReader()
  const decoder = new TextDecoder()
  let buffer = ''
  const deadline = options?.deadlineMs
    ? Date.now() + options.deadlineMs
    : null
  try {
    while (true) {
      if (deadline !== null && Date.now() > deadline) {
        options?.onTimeout?.()
        throw new Error('ndjson_read_timeout')
      }
      const readPromise = reader.read()
      const chunk = deadline !== null
        ? await Promise.race([
            readPromise,
            new Promise<ReadableStreamReadResult<Uint8Array>>((_, reject) => {
              const wait = deadline - Date.now()
              setTimeout(
                () => reject(new Error('ndjson_read_timeout')),
                Math.max(0, wait),
              )
            }),
          ])
        : await readPromise
      const { done, value } = chunk
      if (done)
        break
      buffer += decoder.decode(value, { stream: true })
      let newline = buffer.indexOf('\n')
      while (newline >= 0) {
        const line = buffer.slice(0, newline)
        buffer = buffer.slice(newline + 1)
        const evt = parseNdjsonLine(line)
        if (evt)
          onEvent(evt)
        newline = buffer.indexOf('\n')
      }
    }
    const tail = parseNdjsonLine(buffer)
    if (tail)
      onEvent(tail)
  }
  finally {
    reader.releaseLock()
  }
}

export type CosyvoiceStreamPrefetch = {
  key: string
  chunks: PcmChunk[]
  done: Promise<CosyvoiceStreamResult>
  waitForChunk(afterIndex: number): Promise<void>
  abort: () => void
}

export function abortCosyvoiceStreamPrefetch(
  prefetch: CosyvoiceStreamPrefetch | undefined,
): void {
  prefetch?.abort()
}

export function startCosyvoiceSidecarPrefetch(
  key: string,
  endpoint: string,
  text: string,
  directive?: CosyvoiceDirective | null,
): CosyvoiceStreamPrefetch {
  const cleaned = text.trim()
  const chunks: PcmChunk[] = []
  let ttfc_ms: number | undefined
  let elapsed_ms: number | undefined
  let errorResult: CosyvoiceStreamResult | undefined
  let chunkWaiters: Array<(value: void) => void> = []

  function notifyChunk(): void {
    const waiters = chunkWaiters
    chunkWaiters = []
    for (const w of waiters)
      w()
  }

  function waitForChunk(afterIndex: number): Promise<void> {
    if (chunks.length > afterIndex || errorResult)
      return Promise.resolve()
    return new Promise((resolve) => {
      chunkWaiters.push(resolve)
    })
  }

  const abortController = new AbortController()
  function abort(): void {
    abortController.abort()
    notifyChunk()
  }

  const done = (async (): Promise<CosyvoiceStreamResult> => {
    if (!cleaned)
      return { ok: false, reason: 'empty_text' }
    try {
      const timer = setTimeout(() => abortController.abort(), SIDECAR_STREAM_TIMEOUT_MS)
      let res: Response
      try {
        res = await fetch(sidecarStreamUrl(endpoint), {
          method: 'POST',
          headers: { 'Content-Type': 'application/json; charset=utf-8' },
          body: JSON.stringify(buildStreamPayload(cleaned, directive)),
          signal: abortController.signal,
        })
      }
      finally {
        clearTimeout(timer)
      }
      if (!res.ok || !res.body)
        return { ok: false, reason: 'http_error', message: `HTTP ${res.status}` }

      let totalChunks = 0
      await readNdjsonStream(res.body, (evt) => {
        if (evt.event === 'chunk' && evt.pcm_base64) {
          chunks.push({
            pcm_base64: evt.pcm_base64,
            sample_rate: evt.sample_rate || 22050,
          })
          totalChunks += 1
          notifyChunk()
        }
        else if (evt.event === 'done') {
          ttfc_ms = evt.ttfc_ms
          elapsed_ms = evt.elapsed_ms
          totalChunks = evt.chunks ?? totalChunks
          notifyChunk()
        }
        else if (evt.ok === false) {
          errorResult = {
            ok: false,
            reason: evt.reason || 'stream_error',
            message: evt.message,
          }
          notifyChunk()
        }
      }, { deadlineMs: SIDECAR_STREAM_BODY_READ_TIMEOUT_MS })

      if (errorResult)
        return errorResult
      if (chunks.length === 0)
        return { ok: false, reason: 'cosyvoice_empty', message: 'No audio chunks received' }
      return { ok: true, chunks: totalChunks, ttfc_ms, elapsed_ms }
    }
    catch (err) {
      if (errorResult)
        return errorResult
      return streamFetchError(err)
    }
    finally {
      notifyChunk()
    }
  })()

  return { key, chunks, done, waitForChunk, abort }
}

async function playBufferedOrLiveStream(
  prefetch: CosyvoiceStreamPrefetch | undefined,
  endpoint: string,
  text: string,
  directive?: CosyvoiceDirective | null,
): Promise<CosyvoiceStreamResult> {
  await ensureVoiceAudioReady()
  const player = new PcmStreamScheduler()

  if (prefetch) {
    let played = 0
    const waitStarted = Date.now()
    while (true) {
      if (Date.now() - waitStarted > SIDECAR_STREAM_PLAYBACK_TIMEOUT_MS) {
        abortCosyvoiceStreamPrefetch(prefetch)
        return {
          ok: false,
          reason: 'stream_playback_timeout',
          message: 'Timed out waiting for sidecar stream playback',
        }
      }
      while (played < prefetch.chunks.length) {
        const chunk = prefetch.chunks[played]
        player.schedulePcm16(chunk.pcm_base64, chunk.sample_rate)
        played += 1
      }
      if (played === 0 && Date.now() - waitStarted > SIDECAR_STREAM_FIRST_CHUNK_TIMEOUT_MS) {
        abortCosyvoiceStreamPrefetch(prefetch)
        return {
          ok: false,
          reason: 'stream_first_chunk_timeout',
          message: 'Timed out waiting for first audio chunk from sidecar',
        }
      }
      const meta = await Promise.race([
        prefetch.done,
        prefetch.waitForChunk(Math.max(0, played - 1)).then(() => null),
      ])
      if (meta !== null) {
        while (played < prefetch.chunks.length) {
          const chunk = prefetch.chunks[played]
          player.schedulePcm16(chunk.pcm_base64, chunk.sample_rate)
          played += 1
        }
        await player.waitUntilFinished()
        if (!meta.ok && played === 0)
          return meta
        if (played === 0)
          return { ok: false, reason: 'cosyvoice_empty', message: 'No audio chunks received' }
        const out = {
          ok: true as const,
          chunks: played,
          ttfc_ms: meta.ttfc_ms,
          elapsed_ms: meta.elapsed_ms,
        }
        logVoiceStreamTelemetry(out)
        return out
      }
    }
  }

  let chunks = 0
  let meta: CosyvoiceStreamResult = { ok: true, chunks: 0 }
  const cleaned = text.trim()
  if (!cleaned)
    return { ok: false, reason: 'empty_text' }

  try {
    const res = await fetchSidecarStream(endpoint, cleaned, directive)
    if (!res.ok || !res.body)
      return { ok: false, reason: 'http_error', message: `HTTP ${res.status}` }

    await readNdjsonStream(res.body, (evt) => {
      if (evt.event === 'chunk' && evt.pcm_base64) {
        player.schedulePcm16(evt.pcm_base64, evt.sample_rate || 22050)
        chunks += 1
      }
      else if (evt.event === 'done') {
        meta = {
          ok: true,
          chunks: evt.chunks ?? chunks,
          ttfc_ms: evt.ttfc_ms,
          elapsed_ms: evt.elapsed_ms,
        }
      }
      else if (evt.ok === false) {
        meta = {
          ok: false,
          reason: evt.reason || 'stream_error',
          message: evt.message,
        }
      }
    }, { deadlineMs: SIDECAR_STREAM_BODY_READ_TIMEOUT_MS })
  }
  catch (err) {
    return streamFetchError(err)
  }
  await player.waitUntilFinished()
  if (!meta.ok)
    return meta
  if (chunks === 0)
    return { ok: false, reason: 'cosyvoice_empty', message: 'No audio chunks received' }
  const out = { ok: true as const, chunks, ttfc_ms: meta.ttfc_ms, elapsed_ms: meta.elapsed_ms }
  logVoiceStreamTelemetry(out)
  return out
}

/**
 * Stream CosyVoice2 PCM chunks from the local sidecar and play with Web Audio.
 */
export async function playCosyvoiceSidecarStream(
  endpoint: string,
  text: string,
  directive?: CosyvoiceDirective | null,
  prefetch?: CosyvoiceStreamPrefetch,
): Promise<CosyvoiceStreamResult> {
  return playBufferedOrLiveStream(prefetch, endpoint, text, directive)
}

export function resolveBundledSidecarEndpoint(
  localSynthEndpoint: string | undefined,
): string {
  const trimmed = localSynthEndpoint?.trim()
  return trimmed || DEFAULT_SIDECAR
}

export function shouldUseBundledSidecarStream(synthProvider: string | undefined): boolean {
  const p = (synthProvider || 'bundled').trim().toLowerCase()
  return p === '' || p === 'bundled'
}

/** Stream PCM from local CosyVoice sidecar when engine is cosyvoice2 and provider is bundled. */
export function shouldUseDirectSidecarStream(
  synthProvider: string | undefined,
  engine?: string,
): boolean {
  const eng = (engine || '').trim().toLowerCase()
  if (eng && eng !== 'cosyvoice2')
    return false
  return shouldUseBundledSidecarStream(synthProvider)
}
