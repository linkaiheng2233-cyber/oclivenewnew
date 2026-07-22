import type { AppToastFn } from '@oclive/shared/composables/useAppToast'
import type { VoiceTtsProfileRouting } from '@oclive/shared/lib/voiceTtsRouting'
import type { CosyvoiceStreamPrefetch } from '@oclive/shared/utils/cosyvoiceStreamPlayback'
import { directoryPluginInvoke, getPluginSettingsUi } from '@oclive/shared/api'
import { invokeWithFriendlyError } from '@oclive/shared/api/helpers'
import {
  resetVoiceExpansionWarmSchedule,
  resolveVoiceSidecarEndpoint,
  scheduleVoiceExpansionWarm,
} from '@oclive/shared/composables/useVoiceExpansionWarm'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import {
  VOICE_ASR_CONFIG_UPDATED_EVENT,
  VOICE_ASR_PLUGIN_ID,
  VOICE_STREAM_SENTENCE_EVENT,
} from '@oclive/shared/lib/voiceAsrEvents'
import {
  resolveVoiceTtsRouting,

} from '@oclive/shared/lib/voiceTtsRouting'
import { usePluginStore } from '@oclive/shared/stores/pluginStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import {
  abortCosyvoiceStreamPrefetch,
  cancelVoiceAudioPlayback,

  DEFAULT_COSYVOICE_EMO_TEXT,
  ensureVoiceAudioReady,
  playCosyvoiceSidecarStream,
  resolveBundledSidecarEndpoint,
  resolveStreamDirective,
  shouldUseDirectSidecarStream,
  startCosyvoiceSidecarPrefetch,
} from '@oclive/shared/utils/cosyvoiceStreamPlayback'
import { remainderAfterSpokenPrefix } from '@oclive/shared/utils/extractFirstSpeakableChunk'
import { voiceDialogueFromRaw } from '@oclive/shared/utils/voiceDialogueFromRaw'
import { VoiceSpeakDeduper } from '@oclive/shared/utils/voiceSpeakDeduper'
import { formatVoiceSpeakFailure, shouldFallbackStreamToRpc } from '@oclive/shared/utils/voiceSpeakErrors'
import { onBeforeUnmount, onMounted } from 'vue'

const DEFAULT_TTS_PROFILE = 'bundled-cosyvoice2-zh'

interface MessageSentPayload {
  reply?: string
  bot_emotion?: string
  role_id?: string
  stream_id?: string
  stream_spoken_prefix?: string
  stream_full_raw?: string
  stream_spoken_end_index?: number
}

interface StreamSentencePayload {
  reply?: string
  bot_emotion?: string
  role_id?: string
  sentence?: string
  stream_id?: string
}

interface VoiceRuntimeConfig {
  tts_expansion_enabled: boolean
  auto_tts: boolean
  tts_profile: string
  tts_engine: string
  director_profile: string
  synth_provider: string
  local_synth_endpoint: string
}

interface SpeakOptions {
  /** Stream chunks normally hit the directive cache populated at message submit. */
  fastPath?: boolean
}

interface SpeakJob {
  key: string
  text: string
  payload: { bot_emotion?: string, role_id?: string }
  streamId?: string
  cfg: VoiceRuntimeConfig
  directive: Record<string, unknown>
}

interface RpcSpeakResult {
  ok?: boolean
  audio_base64?: string
  audio_mime?: string
  reason?: string
  message?: string
}

let cachedConfig: VoiceRuntimeConfig | null = null
let cachedConfigAt = 0
const CONFIG_TTL_MS = 30_000

const streamSpokenPrefixById = new Map<string, string>()
const streamPendingById = new Map<string, Promise<void>>()
const streamPlaybackFailed = new Set<string>()
const rolePathCache = new Map<string, string>()
const directiveCache = new Map<string, Record<string, unknown>>()

const speakQueue: SpeakJob[] = []
const preparedRpcSpeak = new Map<string, Promise<RpcSpeakResult>>()
const streamPrefetchByKey = new Map<string, CosyvoiceStreamPrefetch>()
const speakDeduper = new VoiceSpeakDeduper()
let drainingSpeakQueue = false
let speakIdleWaiters: Array<() => void> = []
let speakGeneration = 0
let cachedTtsProfiles: Map<string, VoiceTtsProfileRouting> | null = null
let activeRpcAudio: HTMLAudioElement | null = null
let cancelActiveRpcPlayback: (() => void) | null = null

function resetSpeakPipeline(): void {
  speakGeneration += 1
  speakQueue.length = 0
  preparedRpcSpeak.clear()
  for (const prefetch of streamPrefetchByKey.values())
    abortCosyvoiceStreamPrefetch(prefetch)
  streamPrefetchByKey.clear()
  cancelVoiceAudioPlayback()
  cancelActiveRpcPlayback?.()
  cancelActiveRpcPlayback = null
  if (activeRpcAudio) {
    activeRpcAudio.pause()
    activeRpcAudio.removeAttribute('src')
    activeRpcAudio.load()
  }
  speakDeduper.reset()
  streamSpokenPrefixById.clear()
  streamPendingById.clear()
  streamPlaybackFailed.clear()
  const waiters = speakIdleWaiters
  speakIdleWaiters = []
  for (const resolve of waiters)
    resolve()
}

function waitForSpeakQueueIdle(): Promise<void> {
  if (!drainingSpeakQueue && speakQueue.length === 0)
    return Promise.resolve()
  return new Promise(resolve => speakIdleWaiters.push(resolve))
}

function directiveCacheKey(roleId: string, director: string, emotion: string): string {
  return `${roleId}|${director}|${emotion}`
}

function speakJobKey(text: string, payload: SpeakJob['payload'], cfg: VoiceRuntimeConfig): string {
  return `${payload.role_id || ''}|${payload.bot_emotion || ''}|${cfg.tts_profile}|${text}`
}

export function invalidateVoiceRuntimeConfig(): void {
  cachedConfig = null
  cachedConfigAt = 0
  cachedTtsProfiles = null
}

async function loadTtsProfiles(): Promise<Map<string, VoiceTtsProfileRouting>> {
  if (cachedTtsProfiles)
    return cachedTtsProfiles
  try {
    const list = (await directoryPluginInvoke(
      VOICE_ASR_PLUGIN_ID,
      'voice.list_profiles',
      {},
    )) as {
      profiles?: Array<{
        id: string
        engine?: string
        synth_provider?: string
        sidecar_endpoint?: string
      }>
    }
    const map = new Map<string, VoiceTtsProfileRouting>()
    for (const row of list.profiles || []) {
      if (row.id) {
        map.set(row.id, {
          engine: row.engine,
          synth_provider: row.synth_provider,
          sidecar_endpoint: row.sidecar_endpoint,
        })
      }
    }
    cachedTtsProfiles = map
    return map
  }
  catch {
    return new Map()
  }
}

async function loadVoiceRuntimeConfig(
  isPluginDisabled: (id: string) => boolean,
): Promise<VoiceRuntimeConfig | null> {
  if (isPluginDisabled(VOICE_ASR_PLUGIN_ID))
    return null
  const now = Date.now()
  if (cachedConfig && now - cachedConfigAt < CONFIG_TTL_MS)
    return cachedConfig
  try {
    const ui = await getPluginSettingsUi(VOICE_ASR_PLUGIN_ID)
    const cfg = ui.config ?? {}
    const ttsProfile
      = typeof cfg.tts_profile === 'string' && cfg.tts_profile.trim()
        ? cfg.tts_profile.trim()
        : DEFAULT_TTS_PROFILE
    const profiles = await loadTtsProfiles()
    cachedConfig = {
      tts_expansion_enabled: cfg.tts_expansion_enabled === true,
      auto_tts: cfg.auto_tts === true,
      tts_profile: ttsProfile,
      tts_engine: profiles.get(ttsProfile)?.engine || 'cosyvoice2',
      director_profile:
        typeof cfg.director_profile === 'string'
          ? cfg.director_profile.trim() || 'none'
          : 'rules-v1',
      synth_provider:
        typeof cfg.synth_provider === 'string' ? cfg.synth_provider.trim() : 'bundled',
      local_synth_endpoint:
        typeof cfg.local_synth_endpoint === 'string'
          ? cfg.local_synth_endpoint.trim()
          : '',
    }
    cachedConfigAt = now
    return cachedConfig
  }
  catch {
    return null
  }
}

async function resolveRolePackPath(roleId: string): Promise<string> {
  const rid = roleId.trim()
  if (!rid)
    return ''
  const cached = rolePathCache.get(rid)
  if (cached !== undefined)
    return cached
  try {
    const path = (await invokeWithFriendlyError<string>('get_role_pack_path', {
      roleId: rid,
    })).trim()
    rolePathCache.set(rid, path)
    return path
  }
  catch {
    rolePathCache.set(rid, '')
    return ''
  }
}

async function prefetchVoiceDirective(
  roleId: string,
  director: string,
  emotion: string,
): Promise<void> {
  const cacheKey = directiveCacheKey(roleId, director, emotion)
  if (directiveCache.has(cacheKey))
    return
  const rolePath = roleId ? await resolveRolePackPath(roleId) : ''
  try {
    const built = (await directoryPluginInvoke(
      VOICE_ASR_PLUGIN_ID,
      'voice.build_directive',
      {
        profile: director,
        bot_emotion: emotion,
        role_path: rolePath,
      },
    )) as { ok?: boolean, directive?: Record<string, unknown> }
    if (built.ok && built.directive)
      directiveCache.set(cacheKey, built.directive)
  }
  catch {
    /* prefetch is best-effort */
  }
}

/**
 * Host-side auto TTS for `com.oclive.voice.asr`.
 * Requires voice expansion enabled; reads config on each speak.
 */
export function useVoiceAutoTts(options: { showToast: AppToastFn }) {
  const pluginStore = usePluginStore()
  const roleStore = useRoleStore()
  async function resolveDirective(
    payload: { bot_emotion?: string, role_id?: string },
    cfg: VoiceRuntimeConfig,
    speakOpts: SpeakOptions,
  ): Promise<Record<string, unknown> | undefined> {
    const director = cfg.director_profile
    const emotion = payload.bot_emotion || 'neutral'
    const roleId = payload.role_id?.trim() || ''
    const cacheKey = directiveCacheKey(roleId, director, emotion)
    const cachedDirective = directiveCache.get(cacheKey)
    if (cachedDirective)
      return cachedDirective
    // The first streamed phrase must not wait for a second plugin RPC. The
    // message-submit prefetch normally populated the cache already; otherwise
    // use the selected global profile for this first phrase.
    if (speakOpts.fastPath)
      return undefined
    const rolePath = roleId ? await resolveRolePackPath(roleId) : ''
    try {
      const built = (await directoryPluginInvoke(
        VOICE_ASR_PLUGIN_ID,
        'voice.build_directive',
        {
          profile: director,
          bot_emotion: emotion,
          role_path: rolePath,
        },
      )) as { ok?: boolean, directive?: Record<string, unknown> }
      if (built.ok && built.directive) {
        directiveCache.set(cacheKey, built.directive)
        return built.directive
      }
    }
    catch {
      /* speak without directive */
    }
    return undefined
  }

  function finalizeSpeakDirective(
    directive: Record<string, unknown> | undefined,
    cfg: VoiceRuntimeConfig,
    payload: { bot_emotion?: string },
  ): Record<string, unknown> {
    const resolved = resolveStreamDirective({
      emo_text: typeof directive?.emo_text === 'string' ? directive.emo_text : undefined,
      ref_audio: typeof directive?.ref_audio === 'string' ? directive.ref_audio : undefined,
      ref_text: typeof directive?.ref_text === 'string' ? directive.ref_text : undefined,
      speed: typeof directive?.speed === 'number' ? directive.speed : undefined,
    })
    return {
      schema_version: 1,
      emotion_tag: payload.bot_emotion || directive?.emotion_tag || 'neutral',
      speed: typeof directive?.speed === 'number' ? directive.speed : 1.0,
      energy: directive?.energy || 'normal',
      synth_profile:
        (directive?.synth_profile as string | undefined) || cfg.tts_profile,
      ...directive,
      emo_text: resolved.emo_text || DEFAULT_COSYVOICE_EMO_TEXT,
    }
  }

  async function playRpcAudio(res: RpcSpeakResult): Promise<void> {
    if (!activeRpcAudio)
      activeRpcAudio = new Audio()
    const audio = activeRpcAudio
    const mime = res.audio_mime || 'audio/wav'
    audio.src = `data:${mime};base64,${res.audio_base64}`
    await new Promise<void>((resolve, reject) => {
      let settled = false
      function onEnded() {
        finish()
      }
      function onError() {
        finish(new Error('RPC audio playback failed'))
      }
      function onCancel() {
        finish(new DOMException('RPC audio playback cancelled', 'AbortError'))
      }
      function finish(error?: unknown) {
        if (settled)
          return
        settled = true
        audio.removeEventListener('ended', onEnded)
        audio.removeEventListener('error', onError)
        if (cancelActiveRpcPlayback === onCancel)
          cancelActiveRpcPlayback = null
        if (error)
          reject(error)
        else
          resolve()
      }
      audio.addEventListener('ended', onEnded, { once: true })
      audio.addEventListener('error', onError, { once: true })
      cancelActiveRpcPlayback = onCancel
      void audio.play().catch(finish)
    })
  }

  async function rpcSpeak(job: SpeakJob): Promise<RpcSpeakResult> {
    const rolePath = job.payload.role_id?.trim()
      ? await resolveRolePackPath(job.payload.role_id)
      : ''
    return (await directoryPluginInvoke(
      VOICE_ASR_PLUGIN_ID,
      'voice.speak',
      {
        text: job.text,
        profile: job.cfg.tts_profile,
        directive: job.directive,
        bot_emotion: job.payload.bot_emotion || 'neutral',
        role_path: rolePath,
      },
    )) as RpcSpeakResult
  }

  function prepareRpcSpeak(job: SpeakJob): Promise<RpcSpeakResult> {
    const existing = preparedRpcSpeak.get(job.key)
    if (existing)
      return existing
    const promise = rpcSpeak(job)
    preparedRpcSpeak.set(job.key, promise)
    const cleanupPrepared = () => {
      if (preparedRpcSpeak.get(job.key) === promise)
        preparedRpcSpeak.delete(job.key)
    }
    void promise.then(cleanupPrepared, cleanupPrepared)
    return promise
  }

  function takeStreamPrefetch(key: string): CosyvoiceStreamPrefetch | undefined {
    const pf = streamPrefetchByKey.get(key)
    if (pf)
      streamPrefetchByKey.delete(key)
    return pf
  }

  function cancelStreamPrefetchForKey(key: string): void {
    const pf = streamPrefetchByKey.get(key)
    if (pf) {
      abortCosyvoiceStreamPrefetch(pf)
      streamPrefetchByKey.delete(key)
    }
  }

  async function sidecarEndpointFor(cfg: VoiceRuntimeConfig): Promise<string | null> {
    return resolveVoiceSidecarEndpoint(
      cfg.tts_profile,
      resolveBundledSidecarEndpoint(cfg.local_synth_endpoint),
      id => pluginStore.isPluginDisabled(id),
    )
  }

  async function ensureStreamPrefetch(job: SpeakJob): Promise<void> {
    if (!shouldUseDirectSidecarStream(job.cfg.synth_provider, job.cfg.tts_engine))
      return
    if (streamPrefetchByKey.has(job.key))
      return
    /** One sidecar synthesis at a time — avoid GPU contention (cosyvoice_empty). */
    if (streamPrefetchByKey.size > 0)
      return
    const endpoint = await sidecarEndpointFor(job.cfg)
    if (!endpoint)
      return
    const prefetch = startCosyvoiceSidecarPrefetch(
      job.key,
      endpoint,
      job.text,
      job.directive,
    )
    streamPrefetchByKey.set(job.key, prefetch)
    void prefetch.done.finally(() => {
      if (streamPrefetchByKey.get(job.key) === prefetch)
        streamPrefetchByKey.delete(job.key)
    })
  }

  async function runSpeakJob(job: SpeakJob, generation: number): Promise<void> {
    const reply = job.text.trim()
    if (!reply || generation !== speakGeneration)
      return

    let spoken = false
    try {
      await ensureVoiceAudioReady()
      if (generation !== speakGeneration)
        return

      const wantsStream = shouldUseDirectSidecarStream(job.cfg.synth_provider, job.cfg.tts_engine)
      const endpoint = wantsStream ? await sidecarEndpointFor(job.cfg) : null
      if (endpoint) {
        if (generation !== speakGeneration)
          return
        const prefetch = takeStreamPrefetch(job.key)
        const streamRes = await playCosyvoiceSidecarStream(
          endpoint,
          reply,
          job.directive,
          prefetch,
        )
        if (streamRes.ok && (streamRes.chunks ?? 0) > 0) {
          spoken = true
          return
        }
        cancelStreamPrefetchForKey(job.key)
        abortCosyvoiceStreamPrefetch(prefetch)
        if (shouldFallbackStreamToRpc(streamRes)) {
          console.warn('[voice-auto-tts] stream failed, falling back to RPC', streamRes)
          options.showToast(
            'info',
            `${formatVoiceSpeakFailure('stream', streamRes)}，正在改用 RPC…`,
          )
        }
        else {
          return
        }
      }

      if (generation !== speakGeneration)
        return
      const res = await prepareRpcSpeak(job)
      if (generation !== speakGeneration)
        return
      if (!res.ok || !res.audio_base64) {
        if (res.reason === 'tts_expansion_disabled')
          return
        console.warn('[voice-auto-tts] RPC speak failed', res)
        options.showToast('warning', formatVoiceSpeakFailure('rpc', res))
        return
      }
      await playRpcAudio(res)
      spoken = true
    }
    catch (err) {
      if (generation !== speakGeneration)
        return
      handleSpeakError(err)
    }
    finally {
      if (generation === speakGeneration) {
        speakDeduper.finish(job.key, spoken)
        if (job.streamId) {
          if (spoken && !streamPlaybackFailed.has(job.streamId)) {
            streamSpokenPrefixById.set(
              job.streamId,
              (streamSpokenPrefixById.get(job.streamId) || '') + job.text,
            )
          }
          else if (!spoken) {
            streamPlaybackFailed.add(job.streamId)
          }
        }
      }
    }
  }

  async function drainSpeakQueue(): Promise<void> {
    if (drainingSpeakQueue)
      return
    drainingSpeakQueue = true
    const generation = speakGeneration
    try {
      while (speakQueue.length > 0) {
        if (generation !== speakGeneration)
          break
        const job = speakQueue[0]
        const next = speakQueue[1]
        if (next && generation === speakGeneration) {
          if (shouldUseDirectSidecarStream(next.cfg.synth_provider, next.cfg.tts_engine))
            void ensureStreamPrefetch(next)
          else
            void prepareRpcSpeak(next)
        }
        await runSpeakJob(job, generation)
        if (generation !== speakGeneration)
          break
        speakQueue.shift()
      }
    }
    finally {
      drainingSpeakQueue = false
      if (speakQueue.length > 0) {
        void drainSpeakQueue()
      }
      else {
        const waiters = speakIdleWaiters
        speakIdleWaiters = []
        for (const resolve of waiters) {
          resolve()
        }
      }
    }
  }

  function enqueueSpeakJob(job: SpeakJob): void {
    speakQueue.push(job)
    void drainSpeakQueue()
  }

  async function queueSpeakText(
    text: string,
    payload: { bot_emotion?: string, role_id?: string, stream_id?: string },
    cfg: VoiceRuntimeConfig,
    speakOpts: SpeakOptions = {},
    generation = speakGeneration,
  ): Promise<void> {
    const rawDirective = await resolveDirective(payload, cfg, speakOpts)
    if (generation !== speakGeneration)
      return
    // build_directive always stamps synth_profile (global fill when the pack
    // omits one). Only a pack id that differs from the user's settings profile
    // is a true task override.
    const stamped = typeof rawDirective?.synth_profile === 'string'
      ? rawDirective.synth_profile.trim()
      : ''
    const roleProfile = stamped && stamped !== cfg.tts_profile ? stamped : undefined
    const profiles = await loadTtsProfiles()
    if (generation !== speakGeneration)
      return
    const routing = resolveVoiceTtsRouting(
      cfg,
      roleProfile,
      profiles,
    )
    const effectiveCfg: VoiceRuntimeConfig = { ...cfg, ...routing }
    const directive = finalizeSpeakDirective(rawDirective, effectiveCfg, payload)
    const key = speakJobKey(text, payload, effectiveCfg)
    if (!speakDeduper.markQueued(key))
      return
    enqueueSpeakJob({
      key,
      text,
      payload,
      streamId: payload.stream_id?.trim() || undefined,
      cfg: effectiveCfg,
      directive,
    })
  }

  function handleSpeakError(err: unknown): void {
    const msg = err instanceof Error ? err.message : String(err)
    if (/notallowed|play\(\)/i.test(msg)) {
      options.showToast(
        'info',
        '无法自动播放：请先点击聊天区域或麦克风后再发消息',
      )
    }
    else {
      options.showToast('warning', `语音朗读失败：${msg}`)
    }
  }

  async function speakReply(payload: MessageSentPayload, generation = speakGeneration): Promise<void> {
    const reply = payload.reply?.trim()
    if (!reply)
      return

    const cfg = await loadVoiceRuntimeConfig(id => pluginStore.isPluginDisabled(id))
    if (generation !== speakGeneration || !cfg?.tts_expansion_enabled || !cfg.auto_tts)
      return

    await scheduleVoiceExpansionWarm(id => pluginStore.isPluginDisabled(id))
    if (generation !== speakGeneration)
      return
    await queueSpeakText(reply, payload, cfg, {}, generation)
  }

  async function queueStreamSentence(p: StreamSentencePayload, generation: number): Promise<void> {
    const sentence = p.sentence!.trim()
    const cfg = await loadVoiceRuntimeConfig(id => pluginStore.isPluginDisabled(id))
    if (generation !== speakGeneration || !cfg?.tts_expansion_enabled || !cfg.auto_tts)
      return

    void scheduleVoiceExpansionWarm(id => pluginStore.isPluginDisabled(id))
    await queueSpeakText(sentence, p, cfg, { fastPath: true }, generation)
  }

  function onStreamSentence(payload: unknown): void {
    const p = payload as StreamSentencePayload
    const sentence = p.sentence?.trim()
    const streamId = p.stream_id?.trim()
    if (!sentence || !streamId)
      return

    const generation = speakGeneration
    const previous = streamPendingById.get(streamId) ?? Promise.resolve()
    const pending = previous.catch(() => {}).then(() => queueStreamSentence(p, generation))
    streamPendingById.set(streamId, pending)
    const cleanupPending = () => {
      if (streamPendingById.get(streamId) === pending)
        streamPendingById.delete(streamId)
    }
    void pending.then(cleanupPending, cleanupPending)
  }

  async function onMessageSent(payload: unknown): Promise<void> {
    const p = payload as MessageSentPayload
    const generation = speakGeneration
    if (!p.stream_id?.trim()) {
      void speakReply(p, generation)
      return
    }

    const streamId = p.stream_id.trim()
    await streamPendingById.get(streamId)?.catch(() => {})
    await waitForSpeakQueueIdle()
    if (generation !== speakGeneration)
      return
    // Only audio jobs that actually finished playback count as spoken. If any
    // streamed job failed, replay the final reply in full rather than dropping
    // text based on optimistic enqueue state.
    const spokenPrefix = streamPlaybackFailed.has(streamId)
      ? ''
      : streamSpokenPrefixById.get(streamId) || ''
    streamSpokenPrefixById.delete(streamId)
    streamPendingById.delete(streamId)
    streamPlaybackFailed.delete(streamId)

    const reply = p.reply?.trim()
    if (!reply)
      return

    const cfg = await loadVoiceRuntimeConfig(id => pluginStore.isPluginDisabled(id))
    if (generation !== speakGeneration || !cfg?.tts_expansion_enabled || !cfg.auto_tts)
      return

    const rawFull = p.stream_full_raw ?? ''
    const fullDialogue
      = voiceDialogueFromRaw(rawFull) || voiceDialogueFromRaw(reply) || reply
    const toSpeak = spokenPrefix
      ? remainderAfterSpokenPrefix(fullDialogue, spokenPrefix)
      : fullDialogue
    if (!toSpeak)
      return

    await scheduleVoiceExpansionWarm(id => pluginStore.isPluginDisabled(id))
    if (generation !== speakGeneration)
      return
    await queueSpeakText(toSpeak, p, cfg, {}, generation)
  }

  function onMessageSubmit(payload: unknown): void {
    void ensureVoiceAudioReady()
    resetSpeakPipeline()
    const generation = speakGeneration
    const p = payload as { role_id?: string }
    const roleId = p.role_id?.trim() || roleStore.currentRoleId
    const disabled = (id: string) => pluginStore.isPluginDisabled(id)
    void loadVoiceRuntimeConfig(disabled).then((cfg) => {
      if (generation !== speakGeneration || !cfg?.tts_expansion_enabled || !cfg.auto_tts)
        return
      void prefetchVoiceDirective(roleId, cfg.director_profile, 'neutral')
      void scheduleVoiceExpansionWarm(disabled)
    })
  }

  function onConfigUpdated(): void {
    invalidateVoiceRuntimeConfig()
    streamSpokenPrefixById.clear()
    directiveCache.clear()
    resetSpeakPipeline()
    resetVoiceExpansionWarmSchedule()
    void scheduleVoiceExpansionWarm(id => pluginStore.isPluginDisabled(id))
  }

  function onRoleSwitched(payload: unknown): void {
    resetSpeakPipeline()
    const generation = speakGeneration
    const p = payload as { roleId?: string }
    const roleId = p.roleId?.trim() || roleStore.currentRoleId
    const disabled = (id: string) => pluginStore.isPluginDisabled(id)
    void loadVoiceRuntimeConfig(disabled).then((cfg) => {
      if (generation !== speakGeneration || !cfg?.tts_expansion_enabled || !cfg.auto_tts)
        return
      void prefetchVoiceDirective(roleId, cfg.director_profile, 'neutral')
      void scheduleVoiceExpansionWarm(disabled)
    })
  }

  onMounted(() => {
    hostEventBus.on('message:submit', onMessageSubmit)
    hostEventBus.on('message:sent', onMessageSent)
    hostEventBus.on(VOICE_STREAM_SENTENCE_EVENT, onStreamSentence)
    hostEventBus.on(VOICE_ASR_CONFIG_UPDATED_EVENT, onConfigUpdated)
    hostEventBus.on('role:switched', onRoleSwitched)
    const disabled = (id: string) => pluginStore.isPluginDisabled(id)
    void loadVoiceRuntimeConfig(disabled).then((cfg) => {
      if (!cfg?.tts_expansion_enabled)
        return
      void prefetchVoiceDirective(
        roleStore.currentRoleId,
        cfg.director_profile,
        'neutral',
      )
    })
    void scheduleVoiceExpansionWarm(disabled)
  })

  onBeforeUnmount(() => {
    resetSpeakPipeline()
    hostEventBus.off('message:submit', onMessageSubmit)
    hostEventBus.off('message:sent', onMessageSent)
    hostEventBus.off(VOICE_STREAM_SENTENCE_EVENT, onStreamSentence)
    hostEventBus.off(VOICE_ASR_CONFIG_UPDATED_EVENT, onConfigUpdated)
    hostEventBus.off('role:switched', onRoleSwitched)
  })
}
