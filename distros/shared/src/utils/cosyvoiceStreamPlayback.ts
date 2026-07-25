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

export interface CosyvoiceDirective {
  emo_text?: string
  ref_audio?: string
  ref_text?: string
  speed?: number
}

export interface CosyvoiceStreamResult {
  ok: boolean
  reason?: string
  message?: string
  chunks?: number
  ttfc_ms?: number
  elapsed_ms?: number
  stream_mode?: string
}

export interface CosyvoicePlaybackObserver {
  onFirstChunkScheduled?: () => void
}

function logVoiceStreamTelemetry(result: CosyvoiceStreamResult): void {
  if (!import.meta.env.DEV || !result.ok)
    return
  if (result.ttfc_ms == null && result.elapsed_ms == null)
    return
  // Development-only latency telemetry; production returns above.
  // eslint-disable-next-line no-console
  console.debug('[voice-tts] stream telemetry', {
    ttfc_ms: result.ttfc_ms,
    elapsed_ms: result.elapsed_ms,
    chunks: result.chunks,
    stream_mode: result.stream_mode,
  })
}

interface PcmChunk {
  pcm_base64: string
  sample_rate: number
}

interface NdjsonEvent {
  ok?: boolean
  event?: string
  reason?: string
  message?: string
  pcm_base64?: string
  sample_rate?: number
  chunks?: number
  ttfc_ms?: number
  elapsed_ms?: number
  stream_mode?: string
}

let sharedAudioContext: AudioContext | null = null
const activePcmSchedulers = new Set<PcmStreamScheduler>()
const activeStreamControllers = new Set<AbortController>()
const activePlaybackPrefetches = new Set<CosyvoiceStreamPrefetch>()

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
  private readonly sources = new Set<AudioBufferSourceNode>()
  private cancelWaiters: Array<() => void> = []
  private cancelled = false

  schedulePcm16(pcmBase64: string, sampleRate: number): void {
    if (this.cancelled)
      return
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
    this.sources.add(source)
    source.onended = () => this.sources.delete(source)
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
    if (waitMs > 0 && !this.cancelled) {
      await Promise.race([
        new Promise(resolve => window.setTimeout(resolve, waitMs)),
        new Promise<void>(resolve => this.cancelWaiters.push(resolve)),
      ])
    }
  }

  cancel(): void {
    this.cancelled = true
    for (const source of this.sources) {
      try {
        source.stop()
      }
      catch {
        // The source may already have ended between iteration and stop().
      }
    }
    this.sources.clear()
    const waiters = this.cancelWaiters
    this.cancelWaiters = []
    for (const resolve of waiters)
      resolve()
  }
}

/** Abort all active CosyVoice fetches/prefetches and stop scheduled PCM immediately. */
export function cancelVoiceAudioPlayback(): void {
  for (const controller of activeStreamControllers)
    controller.abort()
  for (const prefetch of activePlaybackPrefetches)
    prefetch.abort()
  for (const scheduler of activePcmSchedulers)
    scheduler.cancel()
}

function sidecarStreamUrl(endpoint: string): string {
  const base = (endpoint || DEFAULT_SIDECAR).trim().replace(/\/+$/, '')
  return `${base}/synthesize/stream`
}

async function fetchSidecarStream(
  endpoint: string,
  text: string,
  directive?: CosyvoiceDirective | null,
  controller = new AbortController(),
): Promise<Response> {
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
    reason: aborted ? 'stream_timeout' : 'stream_read_failed',
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

function readStreamChunkWithDeadline(
  reader: ReadableStreamDefaultReader<Uint8Array>,
  deadline: number,
  onTimeout?: () => void,
): Promise<ReadableStreamReadResult<Uint8Array>> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      onTimeout?.()
      reject(new Error('ndjson_read_timeout'))
    }, Math.max(0, deadline - Date.now()))
    void reader.read().then(
      (chunk) => {
        clearTimeout(timer)
        resolve(chunk)
      },
      (error) => {
        clearTimeout(timer)
        reject(error)
      },
    )
  })
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
      const chunk = deadline !== null
        ? await readStreamChunkWithDeadline(reader, deadline, options?.onTimeout)
        : await reader.read()
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

export interface CosyvoiceStreamPrefetch {
  key: string
  chunks: PcmChunk[]
  done: Promise<CosyvoiceStreamResult>
  waitForChunk: (consumedCount: number) => Promise<void>
  abort: () => void
}

export function abortCosyvoiceStreamPrefetch(
  prefetch: CosyvoiceStreamPrefetch | undefined,
): void {
  prefetch?.abort()
}

/**
 * Owns one-segment look-ahead state.
 *
 * A pending entry is registered before endpoint probing, so the playback path
 * can await and reuse that exact request instead of starting a duplicate.
 */
export class CosyvoiceStreamPrefetchRegistry {
  private readonly readyByKey = new Map<string, CosyvoiceStreamPrefetch>()
  private readonly pendingByKey
    = new Map<string, Promise<CosyvoiceStreamPrefetch | undefined>>()

  get busy(): boolean {
    return this.readyByKey.size > 0 || this.pendingByKey.size > 0
  }

  readyFor(key: string): CosyvoiceStreamPrefetch | undefined {
    return this.readyByKey.get(key)
  }

  pendingFor(
    key: string,
  ): Promise<CosyvoiceStreamPrefetch | undefined> | undefined {
    return this.pendingByKey.get(key)
  }

  setPending(
    key: string,
    promise: Promise<CosyvoiceStreamPrefetch | undefined>,
  ): void {
    this.pendingByKey.set(key, promise)
  }

  clearPending(
    key: string,
    promise: Promise<CosyvoiceStreamPrefetch | undefined>,
  ): void {
    if (this.pendingByKey.get(key) === promise)
      this.pendingByKey.delete(key)
  }

  setReady(key: string, prefetch: CosyvoiceStreamPrefetch): void {
    this.readyByKey.set(key, prefetch)
  }

  async take(key: string): Promise<CosyvoiceStreamPrefetch | undefined> {
    let prefetch = this.readyByKey.get(key)
    if (!prefetch) {
      const pending = this.pendingByKey.get(key)
      if (pending)
        prefetch = await pending
    }
    if (prefetch && this.readyByKey.get(key) === prefetch)
      this.readyByKey.delete(key)
    return prefetch
  }

  cancel(key: string): void {
    const ready = this.readyByKey.get(key)
    if (ready) {
      abortCosyvoiceStreamPrefetch(ready)
      this.readyByKey.delete(key)
    }
    const pending = this.pendingByKey.get(key)
    if (pending) {
      this.pendingByKey.delete(key)
      void pending.then((resolved) => {
        abortCosyvoiceStreamPrefetch(resolved)
        if (resolved && this.readyByKey.get(key) === resolved)
          this.readyByKey.delete(key)
      }).catch(() => {})
    }
  }

  reset(): void {
    for (const ready of this.readyByKey.values())
      abortCosyvoiceStreamPrefetch(ready)
    for (const pending of this.pendingByKey.values()) {
      void pending.then(prefetch => abortCosyvoiceStreamPrefetch(prefetch))
        .catch(() => {})
    }
    this.readyByKey.clear()
    this.pendingByKey.clear()
  }
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
  let stream_mode: string | undefined
  let errorResult: CosyvoiceStreamResult | undefined
  let chunkWaiters: Array<(value: void) => void> = []

  function notifyChunk(): void {
    const waiters = chunkWaiters
    chunkWaiters = []
    for (const w of waiters)
      w()
  }

  function waitForChunk(consumedCount: number): Promise<void> {
    if (chunks.length > consumedCount || errorResult)
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
          stream_mode = evt.stream_mode
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
      }, {
        deadlineMs: SIDECAR_STREAM_BODY_READ_TIMEOUT_MS,
        onTimeout: () => abortController.abort(),
      })

      if (errorResult)
        return errorResult
      if (chunks.length === 0)
        return { ok: false, reason: 'cosyvoice_empty', message: 'No audio chunks received' }
      return { ok: true, chunks: totalChunks, ttfc_ms, elapsed_ms, stream_mode }
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

async function playBufferedOrLiveStreamCore(
  prefetch: CosyvoiceStreamPrefetch | undefined,
  endpoint: string,
  text: string,
  player: PcmStreamScheduler,
  controller: AbortController,
  directive?: CosyvoiceDirective | null,
  observer?: CosyvoicePlaybackObserver,
): Promise<CosyvoiceStreamResult> {
  await ensureVoiceAudioReady()
  let firstChunkScheduled = false
  function scheduleChunk(chunk: PcmChunk): void {
    player.schedulePcm16(chunk.pcm_base64, chunk.sample_rate)
    if (!firstChunkScheduled) {
      firstChunkScheduled = true
      observer?.onFirstChunkScheduled?.()
    }
  }

  if (prefetch) {
    const activePrefetch = prefetch
    let played = 0
    const waitStarted = Date.now()
    function scheduleAvailableChunks(): void {
      while (played < activePrefetch.chunks.length) {
        const chunk = activePrefetch.chunks[played]
        scheduleChunk(chunk)
        // The AudioBuffer now owns decoded samples. Drop the much larger
        // base64 payload immediately instead of retaining every streamed PCM
        // chunk until the complete utterance finishes.
        chunk.pcm_base64 = ''
        played += 1
      }
    }
    while (true) {
      if (Date.now() - waitStarted > SIDECAR_STREAM_PLAYBACK_TIMEOUT_MS) {
        abortCosyvoiceStreamPrefetch(activePrefetch)
        return {
          ok: false,
          reason: 'stream_playback_timeout',
          message: 'Timed out waiting for sidecar stream playback',
        }
      }
      scheduleAvailableChunks()
      if (played === 0 && Date.now() - waitStarted > SIDECAR_STREAM_FIRST_CHUNK_TIMEOUT_MS) {
        abortCosyvoiceStreamPrefetch(activePrefetch)
        return {
          ok: false,
          reason: 'stream_first_chunk_timeout',
          message: 'Timed out waiting for first audio chunk from sidecar',
        }
      }
      const meta = await Promise.race([
        activePrefetch.done,
        // `played` is a consumed count, not the last consumed index. Waiting
        // for `played - 1` resolves immediately while no new chunk exists and
        // creates a microtask spin that can exhaust the WebView heap.
        activePrefetch.waitForChunk(played).then(() => null),
      ])
      if (meta !== null) {
        scheduleAvailableChunks()
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
          stream_mode: meta.stream_mode,
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
    const res = await fetchSidecarStream(endpoint, cleaned, directive, controller)
    if (!res.ok || !res.body)
      return { ok: false, reason: 'http_error', message: `HTTP ${res.status}` }

    await readNdjsonStream(res.body, (evt) => {
      if (evt.event === 'chunk' && evt.pcm_base64) {
        scheduleChunk({
          pcm_base64: evt.pcm_base64,
          sample_rate: evt.sample_rate || 22050,
        })
        chunks += 1
      }
      else if (evt.event === 'done') {
        meta = {
          ok: true,
          chunks: evt.chunks ?? chunks,
          ttfc_ms: evt.ttfc_ms,
          elapsed_ms: evt.elapsed_ms,
          stream_mode: evt.stream_mode,
        }
      }
      else if (evt.ok === false) {
        meta = {
          ok: false,
          reason: evt.reason || 'stream_error',
          message: evt.message,
        }
      }
    }, {
      deadlineMs: SIDECAR_STREAM_BODY_READ_TIMEOUT_MS,
      onTimeout: () => controller.abort(),
    })
  }
  catch (err) {
    return streamFetchError(err)
  }
  await player.waitUntilFinished()
  if (!meta.ok)
    return meta
  if (chunks === 0)
    return { ok: false, reason: 'cosyvoice_empty', message: 'No audio chunks received' }
  const out = {
    ok: true as const,
    chunks,
    ttfc_ms: meta.ttfc_ms,
    elapsed_ms: meta.elapsed_ms,
    stream_mode: meta.stream_mode,
  }
  logVoiceStreamTelemetry(out)
  return out
}

async function playBufferedOrLiveStream(
  prefetch: CosyvoiceStreamPrefetch | undefined,
  endpoint: string,
  text: string,
  directive?: CosyvoiceDirective | null,
  observer?: CosyvoicePlaybackObserver,
): Promise<CosyvoiceStreamResult> {
  const player = new PcmStreamScheduler()
  const controller = new AbortController()
  activePcmSchedulers.add(player)
  activeStreamControllers.add(controller)
  if (prefetch)
    activePlaybackPrefetches.add(prefetch)
  try {
    return await playBufferedOrLiveStreamCore(
      prefetch,
      endpoint,
      text,
      player,
      controller,
      directive,
      observer,
    )
  }
  finally {
    activePcmSchedulers.delete(player)
    activeStreamControllers.delete(controller)
    if (prefetch)
      activePlaybackPrefetches.delete(prefetch)
  }
}

/**
 * Stream CosyVoice2 PCM chunks from the local sidecar and play with Web Audio.
 */
export async function playCosyvoiceSidecarStream(
  endpoint: string,
  text: string,
  directive?: CosyvoiceDirective | null,
  prefetch?: CosyvoiceStreamPrefetch,
  observer?: CosyvoicePlaybackObserver,
): Promise<CosyvoiceStreamResult> {
  return playBufferedOrLiveStream(prefetch, endpoint, text, directive, observer)
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
