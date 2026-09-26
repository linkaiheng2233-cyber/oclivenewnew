// @vitest-environment jsdom
//
// CP-INT B5：共享前端 producer → 真实 hostEventBus → consumer 联动证据。
//
// 与既有单元测试的区别（也是本文件的存在理由）：
//   * `hostEventBus` **不被 mock** —— 真实 `sendChatStoreMessage` 发出的 `message:submit` /
//     `message:sent` 必须经由真实的 mitt 单例送达**真实** `useVoiceAutoTts` 的注册处理器；
//   * 不手动调用 `message:sent` handler 冒充联动，也不把两份单元测试拼接当作集成证据；
//   * 只有传输（API）、role/adult/plugin store、语音 RPC 与 Audio 用内存替身。
//
// 边界：内存传输 + 内存音频，**不代表**真实音频播放、真实 TTS 或 Tauri/浏览器联机。

import type { SendMessageResponse } from '@oclive/shared/api'
import type { ChatMessage } from '../stores/chatStore'
import type { ChatStoreSendContext } from '../stores/chatStoreSend'
import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent } from 'vue'
import { invalidateVoiceRuntimeConfig, useVoiceAutoTts } from '../composables/useVoiceAutoTts'
import { hostEventBus } from '../lib/hostEventBus'
import { cancelActiveChatSend, sendChatStoreMessage } from '../stores/chatStoreSend'

const STREAM_SENTENCE_EVENT = 'com.oclive.voice:stream-sentence'

const mocks = vi.hoisted(() => ({
  sendMessage: vi.fn(),
  recoverMessage: vi.fn(),
  sendMessageStream: vi.fn(),
  startQueue: vi.fn(),
  cancelQueue: vi.fn(),
  recordKnowledge: vi.fn(),
  updateLocal: vi.fn(),
  updateRelation: vi.fn(),
  updateSession: vi.fn(),
  markSettled: vi.fn(),
  directoryInvoke: vi.fn(),
  getSettings: vi.fn(),
  invokeFriendly: vi.fn(),
  showToast: vi.fn(),
  pluginDisabled: false,
  uiStore: { sceneId: 'home' },
  roleStore: {
    currentRoleId: 'mumu',
    roleInfo: {
      adultExtensionAvailable: false,
      relationState: 'Friend',
      replyMode: null,
      updateLocalAfterMessage: vi.fn(),
      updateRelationState: vi.fn(),
    },
  },
  adultStore: {
    backgroundQueueEnabled: false,
    requestFor: vi.fn(() => undefined),
    sessionFor: vi.fn(() => ({ active: true, voiceTextOnly: false, updatedAt: 1 })),
    updateSession: vi.fn(),
  },
}))

vi.mock('@oclive/shared/api', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@oclive/shared/api')>()
  return {
    ...actual,
    sendMessage: mocks.sendMessage,
    recoverMessage: mocks.recoverMessage,
    sendMessageStream: mocks.sendMessageStream,
    directoryPluginInvoke: mocks.directoryInvoke,
    getPluginSettingsUi: mocks.getSettings,
    toastAsyncError: vi.fn(),
  }
})

vi.mock('@oclive/shared/api/helpers', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@oclive/shared/api/helpers')>()
  return { ...actual, invokeWithFriendlyError: mocks.invokeFriendly }
})

vi.mock('@oclive/shared/composables/useVoiceExpansionWarm', () => ({
  resetVoiceExpansionWarmSchedule: vi.fn(),
  resolveVoiceSidecarEndpoint: vi.fn(async () => null),
  scheduleVoiceExpansionWarm: vi.fn(async () => undefined),
}))

vi.mock('@oclive/shared/lib/adultBeatQueue', () => ({
  cancelAdultBeatQueue: mocks.cancelQueue,
  resumeAdultBeatQueue: vi.fn(),
  startAdultBeatQueue: mocks.startQueue,
}))

vi.mock('@oclive/shared/lib/voicePlaybackSettlement', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@oclive/shared/lib/voicePlaybackSettlement')>()
  return { ...actual, markVoicePlaybackSettled: mocks.markSettled }
})

vi.mock('@oclive/shared/utils/chatStreamSettings', () => ({
  isChatStreamEnabled: () => true,
}))

// Audio-layer double only: jsdom has no WebAudio, so the shared `AudioContext` bootstrap is
// replaced. Everything above it (event chain, directive RPC, `voice.speak` call and the
// `new Audio()` playback double) stays real.
vi.mock('@oclive/shared/utils/cosyvoiceStreamPlayback', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@oclive/shared/utils/cosyvoiceStreamPlayback')>()
  return { ...actual, ensureVoiceAudioReady: vi.fn(async () => undefined) }
})

vi.mock('@oclive/shared/stores/pluginStore', () => ({
  usePluginStore: () => ({ isPluginDisabled: () => mocks.pluginDisabled }),
}))

vi.mock('@oclive/shared/stores/roleStore', () => ({
  useRoleStore: () => mocks.roleStore,
}))

vi.mock('../stores/adultInteractionStore', () => ({
  useAdultInteractionStore: () => mocks.adultStore,
}))

vi.mock('../stores/debugStore', () => ({
  useDebugStore: () => ({ recordKnowledgeFromSend: mocks.recordKnowledge }),
}))

vi.mock('../stores/uiStore', () => ({
  useUiStore: () => mocks.uiStore,
}))

// Real `useVoiceAutoTts` must be mounted so its handlers register on the real bus.
const VoiceHarness = defineComponent({
  setup() {
    useVoiceAutoTts({ showToast: mocks.showToast })
    return () => null
  },
})

interface AudioDouble extends EventTarget {
  src: string
  play: () => Promise<void>
  pause: () => void
  load: () => void
  removeAttribute: (name: string) => void
}

const audioDoubles: AudioDouble[] = []

class InMemoryAudio extends EventTarget implements AudioDouble {
  src = ''
  paused = true

  constructor() {
    super()
    audioDoubles.push(this)
  }

  play(): Promise<void> {
    this.paused = false
    queueMicrotask(() => this.dispatchEvent(new Event('ended')))
    return Promise.resolve()
  }

  pause(): void {
    this.paused = true
  }

  load(): void {}

  removeAttribute(name: string): void {
    if (name === 'src')
      this.src = ''
  }
}

/** The transient prefix the double streams before the provider fails. */
const STALE_PREFIX = '清晨的风'
/** The authoritative fallback the host sends in the single terminal result. */
const AUTHORITATIVE_REPLY = '（有点卡）A MuMu：你刚说的「聊聊清晨的风景吧。」，嗯，我听到了。你接着说。'

function ordinaryResponse(reply: string): SendMessageResponse {
  return {
    api_version: 1,
    schema: 16,
    presence_mode: 'co_present',
    relation_state: 'Friend',
    reply,
    adult_beat: null,
    reply_presentation: null,
    emotion: { joy: 0, sadness: 0, anger: 0, fear: 0, surprise: 0, disgust: 0, neutral: 1 },
    bot_emotion: 'neutral',
    portrait_emotion: 'neutral',
    favorability_delta: 0,
    favorability_current: 50,
    events: [],
    scene_id: 'default',
    offer_destination_picker: false,
    offer_together_travel: false,
    reply_is_fallback: true,
    knowledge_chunks_in_prompt: 0,
    timestamp: 1,
    user_message_id: 'user-1',
    assistant_message_id: 'assistant-1',
    user_message_timestamp: 1,
    assistant_message_timestamp: 2,
  }
}

function context(messages: ChatMessage[]): ChatStoreSendContext {
  return {
    sceneHistorySplitIndex: {},
    setLoading: vi.fn(),
    getMessageCountForRoleScene: () => messages.length,
    addMessage: (_roleId, _sceneId, message) => {
      messages.push(message)
    },
    patchMessageById: (roleId, sceneId, localId, patch) => {
      void roleId
      void sceneId
      const message = messages.find(item => item.id === localId)
      if (message)
        Object.assign(message, patch)
    },
    deleteMessage: (_roleId, _sceneId, messageId) => {
      const index = messages.findIndex(item => item.id === messageId)
      if (index >= 0)
        messages.splice(index, 1)
    },
    addSystemMessage: vi.fn(),
    clampSceneHistorySplitForBucket: vi.fn(),
  }
}

describe('shared frontend voice event chain (real hostEventBus)', () => {
  let wrapper: ReturnType<typeof mount> | undefined
  let sentEvents: Record<string, unknown>[] = []
  let streamSentenceEvents: Record<string, unknown>[] = []
  let speakTexts: string[] = []
  const onSent = (payload: unknown) => {
    sentEvents.push(payload as Record<string, unknown>)
  }
  const onSentence = (payload: unknown) => {
    streamSentenceEvents.push(payload as Record<string, unknown>)
  }

  beforeEach(async () => {
    vi.clearAllMocks()
    audioDoubles.length = 0
    vi.stubGlobal('Audio', InMemoryAudio)
    invalidateVoiceRuntimeConfig()
    sentEvents = []
    streamSentenceEvents = []
    speakTexts = []

    mocks.pluginDisabled = false
    mocks.roleStore.currentRoleId = 'mumu'
    mocks.roleStore.roleInfo.adultExtensionAvailable = false
    mocks.roleStore.roleInfo.replyMode = null
    mocks.uiStore.sceneId = 'home'
    mocks.roleStore.updateLocalAfterMessage = mocks.updateLocal
    mocks.roleStore.updateRelationState = mocks.updateRelation
    mocks.adultStore.updateSession = mocks.updateSession
    mocks.adultStore.sessionFor = vi.fn(() => ({ active: true, voiceTextOnly: false, updatedAt: 1 }))
    mocks.cancelQueue.mockResolvedValue(undefined)
    mocks.startQueue.mockResolvedValue(undefined)
    mocks.invokeFriendly.mockResolvedValue('D:/roles/mumu')
    mocks.sendMessage.mockReset()
    mocks.recoverMessage.mockReset()
    mocks.sendMessageStream.mockReset()
    mocks.getSettings.mockResolvedValue({
      config: {
        tts_expansion_enabled: true,
        auto_tts: true,
        role_tts_enabled: { mumu: true },
        tts_profile: 'bundled-cosyvoice2-zh',
        synth_provider: 'bundled',
      },
    })
    mocks.directoryInvoke.mockImplementation(async (
      _pluginId: string,
      method: string,
      payload?: { text?: string },
    ) => {
      if (method === 'voice.list_profiles')
        return { profiles: [] }
      if (method === 'voice.read_role_profile') {
        return { ok: true, profile: { synth_profile: 'bundled-cosyvoice2-zh' } }
      }
      if (method === 'voice.build_directive') {
        return { ok: true, directive: { synth_profile: 'bundled-cosyvoice2-zh' } }
      }
      if (method === 'voice.speak') {
        speakTexts.push(String(payload?.text ?? ''))
        return { ok: true, audio_base64: 'QUJD', audio_mime: 'audio/wav' }
      }
      return { ok: true }
    })

    // Real listeners on the real bus: these observe what the chain actually published.
    hostEventBus.on('message:sent', onSent)
    hostEventBus.on(STREAM_SENTENCE_EVENT, onSentence)

    wrapper = mount(VoiceHarness)
    // Let the mounted consumer finish its first config load before the turn starts.
    await vi.waitFor(() => expect(mocks.getSettings).toHaveBeenCalled())
  })

  afterEach(() => {
    hostEventBus.off('message:sent', onSent)
    hostEventBus.off(STREAM_SENTENCE_EVENT, onSentence)
    wrapper?.unmount()
    vi.unstubAllGlobals()
  })

  it('keeps the transient prefix silent and speaks the authoritative reply exactly once', async () => {
    expect(AUTHORITATIVE_REPLY.startsWith(STALE_PREFIX)).toBe(false)
    let releaseStream: ((value: SendMessageResponse) => void) | undefined
    mocks.sendMessageStream.mockImplementation((
      _request: unknown,
      handlers: { onToken: (token: string, accumulated: string) => void },
    ) => {
      // The provider emits a partial prefix and then keeps the turn open.
      handlers.onToken(STALE_PREFIX, STALE_PREFIX)
      return new Promise<SendMessageResponse>((resolve) => {
        releaseStream = resolve
      })
    })
    const messages: ChatMessage[] = []
    const ctx = context(messages)

    const pending = sendChatStoreMessage(ctx, 'hello', 'home')
    await vi.waitFor(() => expect(mocks.sendMessageStream).toHaveBeenCalledTimes(1))

    // ---- 最终结果释放前：暂态视觉可见，但零播报、零流式句子事件 ----
    const transient = messages.filter(message => message.role === 'assistant')
    expect(transient).toHaveLength(1)
    expect(transient[0]).toMatchObject({ id: expect.any(String), content: STALE_PREFIX })
    expect(speakTexts).toEqual([])
    expect(mocks.directoryInvoke.mock.calls.filter(call => call[1] === 'voice.speak')).toHaveLength(0)
    expect(streamSentenceEvents).toEqual([])
    expect(sentEvents).toEqual([])

    releaseStream?.(ordinaryResponse(AUTHORITATIVE_REPLY))
    await pending

    // ---- 完成后：唯一权威气泡 + 唯一 message:sent + 唯一一次全文播报 ----
    const assistants = messages.filter(message => message.role === 'assistant')
    expect(assistants).toHaveLength(1)
    expect(assistants[0]).toMatchObject({
      id: 'assistant-1',
      content: AUTHORITATIVE_REPLY,
      streaming: false,
    })
    expect(assistants[0].content).not.toBe(STALE_PREFIX)

    expect(sentEvents).toHaveLength(1)
    const sent = sentEvents[0]
    expect(sent.reply).toBe(AUTHORITATIVE_REPLY)
    expect(sent).not.toHaveProperty('stream_id')
    expect(sent).not.toHaveProperty('stream_spoken_prefix')
    expect(sent).not.toHaveProperty('stream_full_raw')
    expect(sent).not.toHaveProperty('stream_spoken_end_index')
    expect(sent.skip_auto_tts).toBe(false)

    // The real consumer is fire-and-forget from the producer's point of view, so wait for the
    // chain to settle instead of calling the handler by hand.
    await vi.waitFor(() => expect(speakTexts).toHaveLength(1))
    await vi.waitFor(() => expect(mocks.markSettled).toHaveBeenCalledTimes(1))
    // The real consumer spoke the whole authoritative text once — not a remainder "top-up".
    expect(speakTexts).toEqual([AUTHORITATIVE_REPLY])
    expect(mocks.markSettled).toHaveBeenCalledWith(expect.any(String), 'complete')
    expect(streamSentenceEvents).toEqual([])
    expect(mocks.sendMessage).not.toHaveBeenCalled()
  })

  it('recovers the same identified turn, stays silent until it lands, then speaks the full reply once', async () => {
    let rejectStream: ((reason: Error) => void) | undefined
    let releaseRecovery: ((value: SendMessageResponse) => void) | undefined
    mocks.sendMessageStream.mockImplementation((
      _request: unknown,
      handlers: { onToken: (token: string, accumulated: string) => void },
    ) => {
      handlers.onToken(STALE_PREFIX, STALE_PREFIX)
      return new Promise<SendMessageResponse>((_resolve, reject) => {
        rejectStream = reject
      })
    })
    mocks.recoverMessage.mockImplementation(() => new Promise<SendMessageResponse>((resolve) => {
      releaseRecovery = resolve
    }))
    const messages: ChatMessage[] = []
    const ctx = context(messages)

    const pending = sendChatStoreMessage(ctx, 'hello', 'home')
    await vi.waitFor(() => expect(mocks.sendMessageStream).toHaveBeenCalledTimes(1))
    rejectStream?.(new Error('stream ended without done event'))
    await vi.waitFor(() => expect(mocks.recoverMessage).toHaveBeenCalledTimes(1))

    // Same turn identity, no new ordinary /chat turn.
    const streamRequest = mocks.sendMessageStream.mock.calls[0][0] as { client_request_id?: string }
    const recoveryRequest = mocks.recoverMessage.mock.calls[0][0] as { client_request_id?: string }
    expect(recoveryRequest).toBe(streamRequest)
    expect(recoveryRequest.client_request_id).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/,
    )
    expect(mocks.sendMessage).not.toHaveBeenCalled()

    // Recovery is still in flight: the transient bubble is gone and nothing is spoken.
    expect(messages.filter(message => message.role === 'assistant')).toHaveLength(0)
    expect(speakTexts).toEqual([])
    expect(sentEvents).toEqual([])
    expect(streamSentenceEvents).toEqual([])

    releaseRecovery?.(ordinaryResponse(AUTHORITATIVE_REPLY))
    await pending

    const assistants = messages.filter(message => message.role === 'assistant')
    expect(assistants).toHaveLength(1)
    expect(assistants[0]).toMatchObject({ id: 'assistant-1', content: AUTHORITATIVE_REPLY })
    expect(sentEvents).toHaveLength(1)
    expect(sentEvents[0].reply).toBe(AUTHORITATIVE_REPLY)
    await vi.waitFor(() => expect(speakTexts).toHaveLength(1))
    await vi.waitFor(() => expect(mocks.markSettled).toHaveBeenCalledTimes(1))
    expect(speakTexts).toEqual([AUTHORITATIVE_REPLY])
    expect(mocks.markSettled).toHaveBeenCalledWith(expect.any(String), 'complete')
  })

  it('publishes nothing and leaves no bubble when recovery fails', async () => {
    mocks.sendMessageStream.mockImplementation((
      _request: unknown,
      handlers: { onToken: (token: string, accumulated: string) => void },
    ) => {
      handlers.onToken(STALE_PREFIX, STALE_PREFIX)
      return Promise.reject(new Error('stream ended without done event'))
    })
    const recoveryError = new Error('outcome unconfirmed')
    mocks.recoverMessage.mockRejectedValue(recoveryError)
    const messages: ChatMessage[] = []
    const ctx = context(messages)

    await expect(sendChatStoreMessage(ctx, 'hello', 'home')).rejects.toBe(recoveryError)
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(messages.filter(message => message.role === 'assistant')).toHaveLength(0)
    expect(sentEvents).toEqual([])
    expect(speakTexts).toEqual([])
    expect(streamSentenceEvents).toEqual([])
    expect(mocks.markSettled).not.toHaveBeenCalled()
    expect(ctx.setLoading).toHaveBeenLastCalledWith(false)
  })

  it('cancels the owned transport without publishing or speaking', async () => {
    let capturedSignal: AbortSignal | undefined
    mocks.sendMessageStream.mockImplementation((
      _request: unknown,
      handlers: { signal: AbortSignal, onToken: (token: string, accumulated: string) => void },
    ) => {
      capturedSignal = handlers.signal
      handlers.onToken(STALE_PREFIX, STALE_PREFIX)
      return new Promise<SendMessageResponse>((_resolve, reject) => {
        handlers.signal.addEventListener('abort', () => reject(new DOMException('test abort', 'AbortError')))
      })
    })
    const messages: ChatMessage[] = []
    const pending = sendChatStoreMessage(context(messages), 'hello', 'home')
    await vi.waitFor(() => expect(capturedSignal).toBeDefined())
    expect(messages.filter(message => message.role === 'assistant')).toHaveLength(1)

    cancelActiveChatSend()
    await pending
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(capturedSignal?.aborted).toBe(true)
    expect(messages.filter(message => message.role === 'assistant')).toHaveLength(0)
    expect(sentEvents).toEqual([])
    expect(speakTexts).toEqual([])
    expect(streamSentenceEvents).toEqual([])
    expect(mocks.recoverMessage).not.toHaveBeenCalled()
    expect(mocks.sendMessage).not.toHaveBeenCalled()
  })

  it('h02 shape: the host fallback token is previewed but stays silent until done speaks it once', async () => {
    // Precise shape (B5-R1): the *provider* fails before its first model token, so no model
    // text ever streams. The HOST then emits exactly one fallback token frame followed by the
    // single done, and that token text is byte-identical to done.reply. This is why the case
    // must deliver a token: "no model token" is not the same as "no SSE token at all".
    const fallback = AUTHORITATIVE_REPLY
    let releaseStream: ((value: SendMessageResponse) => void) | undefined
    mocks.sendMessageStream.mockImplementation((
      _request: unknown,
      handlers: { onToken: (token: string, accumulated: string) => void },
    ) => {
      handlers.onToken(fallback, fallback)
      return new Promise<SendMessageResponse>((resolve) => {
        releaseStream = resolve
      })
    })
    const messages: ChatMessage[] = []
    const pending = sendChatStoreMessage(context(messages), 'hello', 'home')
    await vi.waitFor(() => expect(mocks.sendMessageStream).toHaveBeenCalledTimes(1))

    // The fallback token is already visible as a transient preview while done is still pending:
    // nothing may be spoken and nothing may be published yet.
    const transient = messages.filter(message => message.role === 'assistant')
    expect(transient).toHaveLength(1)
    expect(transient[0].content).toBe(fallback)
    expect(speakTexts).toEqual([])
    expect(streamSentenceEvents).toEqual([])
    expect(sentEvents).toEqual([])
    expect(mocks.markSettled).not.toHaveBeenCalled()

    // Done carries the same fallback text, byte-for-byte.
    releaseStream?.(ordinaryResponse(fallback))
    await pending

    const assistants = messages.filter(message => message.role === 'assistant')
    expect(assistants).toHaveLength(1)
    expect(assistants[0]).toMatchObject({
      id: 'assistant-1',
      content: fallback,
      streaming: false,
    })
    expect(sentEvents).toHaveLength(1)
    expect(sentEvents[0].reply).toBe(fallback)
    expect(sentEvents[0]).not.toHaveProperty('stream_id')
    expect(sentEvents[0]).not.toHaveProperty('stream_spoken_prefix')
    await vi.waitFor(() => expect(speakTexts).toHaveLength(1))
    await vi.waitFor(() => expect(mocks.markSettled).toHaveBeenCalledTimes(1))
    expect(speakTexts).toEqual([fallback])
    expect(mocks.markSettled).toHaveBeenCalledWith(expect.any(String), 'complete')
    expect(streamSentenceEvents).toEqual([])
  })
})
