// @vitest-environment jsdom

import type { SendMessageResponse } from '@oclive/shared/api'
import type { ChatMessage } from './chatStore'
import type { ChatStoreSendContext } from './chatStoreSend'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { sendChatStoreMessage } from './chatStoreSend'

const mocks = vi.hoisted(() => ({
  sendMessage: vi.fn(),
  sendMessageStream: vi.fn(),
  streamEnabled: false,
  startQueue: vi.fn(),
  cancelQueue: vi.fn(),
  emitBuiltin: vi.fn(),
  recordKnowledge: vi.fn(),
  updateLocal: vi.fn(),
  updateRelation: vi.fn(),
  updateSession: vi.fn(),
  uiStore: {
    sceneId: 'home',
  },
  roleStore: {
    currentRoleId: 'role',
    roleInfo: {
      adultExtensionAvailable: true,
      relationState: 'Friend',
      replyMode: null as null | {
        mode: 'burst'
        segments: number
        separator: string
        delays_ms: number[]
        streaming: 'live' | 'batch'
      },
    },
    updateLocalAfterMessage: vi.fn(),
    updateRelationState: vi.fn(),
  },
  adultStore: {
    backgroundQueueEnabled: true,
    requestFor: vi.fn(() => ({
      confirmed_adult: true,
      global_enabled: true,
      role_enabled: true,
      interaction_active: true,
      action: 'message',
    })),
    sessionFor: vi.fn(() => ({
      active: true,
      voiceTextOnly: false,
      updatedAt: 1,
    })),
    updateSession: vi.fn(),
  },
}))

vi.mock('@oclive/shared/api', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@oclive/shared/api')>()
  return {
    ...actual,
    sendMessage: mocks.sendMessage,
    sendMessageStream: mocks.sendMessageStream,
    toastAsyncError: vi.fn(),
  }
})

vi.mock('@oclive/shared/lib/adultBeatQueue', () => ({
  cancelAdultBeatQueue: mocks.cancelQueue,
  resumeAdultBeatQueue: vi.fn(),
  startAdultBeatQueue: mocks.startQueue,
}))

vi.mock('@oclive/shared/lib/hostEventBus', () => ({
  hostEventBus: { emitBuiltin: mocks.emitBuiltin },
}))

vi.mock('@oclive/shared/utils/chatStreamSettings', () => ({
  isChatStreamEnabled: () => mocks.streamEnabled,
}))

vi.mock('./adultInteractionStore', () => ({
  useAdultInteractionStore: () => mocks.adultStore,
}))

vi.mock('./debugStore', () => ({
  useDebugStore: () => ({ recordKnowledgeFromSend: mocks.recordKnowledge }),
}))

vi.mock('./roleStore', () => ({
  useRoleStore: () => mocks.roleStore,
}))

vi.mock('./uiStore', () => ({
  useUiStore: () => mocks.uiStore,
}))

function response(): SendMessageResponse {
  return {
    api_version: 1,
    schema: 1,
    presence_mode: 'co_present',
    relation_state: 'Friend',
    reply: 'dialogue',
    adult_beat: {
      dialogue: '只朗读这句对白',
      narration: '她把杯子轻轻放在桌上。',
      interaction_state: 'active',
      next_beat_interval_ms: 10,
    },
    emotion: {
      joy: 0,
      sadness: 0,
      anger: 0,
      fear: 0,
      surprise: 0,
      disgust: 0,
      neutral: 1,
    },
    bot_emotion: 'neutral',
    portrait_emotion: 'neutral',
    favorability_delta: 0,
    favorability_current: 1,
    events: [],
    scene_id: 'home',
    offer_destination_picker: false,
    offer_together_travel: false,
    reply_is_fallback: false,
    knowledge_chunks_in_prompt: 0,
    timestamp: 1,
    user_message_id: 'user-1',
    assistant_message_id: 'assistant-1',
    user_message_timestamp: 1,
    assistant_message_timestamp: 2,
  }
}

function segmentedResponse(): SendMessageResponse {
  return {
    ...response(),
    reply: 'first burst\nsecond burst\nthird burst',
    adult_beat: null,
    reply_presentation: {
      segments: ['first burst', 'second burst', 'third burst'],
      delays_ms: [0, 100, 100],
    },
  }
}

function ordinaryResponse(reply = 'ordinary stream.'): SendMessageResponse {
  return {
    ...response(),
    reply,
    adult_beat: null,
    reply_presentation: null,
  }
}

function context(messages: ChatMessage[]): ChatStoreSendContext {
  return {
    sceneHistorySplitIndex: {},
    setLoading: vi.fn(),
    getMessageCountForRoleScene: () => messages.length,
    addMessage: (_roleId, _sceneId, message) => messages.push(message),
    patchMessageById: vi.fn((_roleId, _sceneId, localId, patch) => {
      const message = messages.find(item => item.id === localId)
      if (message)
        Object.assign(message, patch)
    }),
    deleteMessage: vi.fn((_roleId, _sceneId, messageId) => {
      const index = messages.findIndex(item => item.id === messageId)
      if (index >= 0)
        messages.splice(index, 1)
    }),
    addSystemMessage: vi.fn(),
    clampSceneHistorySplitForBucket: vi.fn(),
  }
}

describe('chat store send presentation', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mocks.sendMessageStream.mockReset()
    mocks.streamEnabled = false
    mocks.roleStore.currentRoleId = 'role'
    mocks.roleStore.roleInfo.adultExtensionAvailable = true
    mocks.roleStore.roleInfo.replyMode = null
    mocks.uiStore.sceneId = 'home'
    mocks.roleStore.updateLocalAfterMessage = mocks.updateLocal
    mocks.roleStore.updateRelationState = mocks.updateRelation
    mocks.adultStore.updateSession = mocks.updateSession
    mocks.sendMessage.mockResolvedValue(response())
    mocks.cancelQueue.mockResolvedValue(undefined)
    mocks.startQueue.mockResolvedValue(undefined)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('renders dialogue and narration separately and emits only dialogue for TTS', async () => {
    const messages: ChatMessage[] = []

    await sendChatStoreMessage(
      context(messages),
      '继续',
      'home',
    )

    const assistant = messages.find(message => message.role === 'assistant')
    expect(assistant).toMatchObject({
      content: '只朗读这句对白',
      aside: '她把杯子轻轻放在桌上。',
    })
    const sentEvent = mocks.emitBuiltin.mock.calls.find(
      call => call[0] === 'message:sent',
    )
    expect(sentEvent?.[1]).toMatchObject({
      reply: '只朗读这句对白',
      reply_aside: '她把杯子轻轻放在桌上。',
      role_id: 'role',
      scene_id: 'home',
      skip_auto_tts: false,
    })
    expect(String(sentEvent?.[1]?.reply)).not.toContain('她把杯子')
  })

  it('uses portrait emotion for role state while keeping bot emotion on the bubble', async () => {
    mocks.sendMessage.mockResolvedValue({
      ...response(),
      bot_emotion: 'happy',
      portrait_emotion: 'angry',
    })
    const messages: ChatMessage[] = []

    await sendChatStoreMessage(context(messages), '继续', 'home')

    expect(messages.find(message => message.role === 'assistant')).toMatchObject({
      emotion: 'happy',
    })
    expect(mocks.updateLocal).toHaveBeenCalledWith(
      'angry',
      1,
      expect.any(Object),
    )
  })

  it('drops a late reply after the foreground scene changes', async () => {
    let resolveSend: ((value: SendMessageResponse) => void) | undefined
    mocks.sendMessage.mockReturnValueOnce(new Promise((resolve) => {
      resolveSend = resolve
    }))
    const messages: ChatMessage[] = []

    const pending = sendChatStoreMessage(context(messages), '继续', 'home')
    await vi.waitFor(() => expect(mocks.sendMessage).toHaveBeenCalledTimes(1))
    mocks.uiStore.sceneId = 'garden'
    resolveSend?.(response())
    await pending

    expect(messages).toHaveLength(1)
    expect(messages[0]).toMatchObject({ role: 'user', content: '继续' })
    expect(mocks.emitBuiltin).not.toHaveBeenCalledWith(
      'message:sent',
      expect.anything(),
    )
    expect(mocks.updateLocal).not.toHaveBeenCalled()
    expect(mocks.updateRelation).not.toHaveBeenCalled()
  })

  it('reveals every live reply-mode segment in order and defers voice to the clean final reply', async () => {
    vi.useFakeTimers()
    mocks.streamEnabled = true
    mocks.roleStore.roleInfo.adultExtensionAvailable = false
    mocks.roleStore.roleInfo.replyMode = {
      mode: 'burst',
      segments: 3,
      separator: '+++',
      delays_ms: [0, 100, 100],
      streaming: 'live',
    }
    let resolveStream: ((value: SendMessageResponse) => void) | undefined
    mocks.sendMessageStream.mockImplementation((
      _request: unknown,
      handlers: { onToken: (token: string, accumulated: string) => void },
    ) => {
      handlers.onToken('', 'first burst\n+++\nsecond burst\n+++\nthird burst')
      return new Promise<SendMessageResponse>((resolve) => {
        resolveStream = resolve
      })
    })
    const messages: ChatMessage[] = []

    const pending = sendChatStoreMessage(context(messages), 'hello', 'home')
    await vi.advanceTimersByTimeAsync(0)
    expect(messages.filter(message => message.role === 'assistant')).toHaveLength(1)

    await vi.advanceTimersByTimeAsync(100)
    expect(messages.filter(message => message.role === 'assistant')).toHaveLength(2)
    await vi.advanceTimersByTimeAsync(100)
    expect(messages.filter(message => message.role === 'assistant')).toHaveLength(3)

    expect(resolveStream).toBeTypeOf('function')
    resolveStream!(segmentedResponse())
    await vi.advanceTimersByTimeAsync(0)
    await pending

    const assistants = messages.filter(message => message.role === 'assistant')
    expect(assistants.map(message => message.id)).toEqual([
      'assistant-1#s0',
      'assistant-1#s1',
      'assistant-1#s2',
    ])
    expect(assistants.map(message => message.content)).toEqual([
      'first burst',
      'second burst',
      'third burst',
    ])
    expect(mocks.emitBuiltin).not.toHaveBeenCalledWith(
      'com.oclive.voice:stream-sentence',
      expect.anything(),
    )
    const sentEvent = mocks.emitBuiltin.mock.calls.find(call => call[0] === 'message:sent')
    expect(sentEvent?.[1]).toMatchObject({
      reply: 'first burst\nsecond burst\nthird burst',
      stream_id: undefined,
    })
  })

  it('withholds batch reply-mode bubbles until the final presentation arrives', async () => {
    mocks.streamEnabled = true
    mocks.roleStore.roleInfo.adultExtensionAvailable = false
    mocks.roleStore.roleInfo.replyMode = {
      mode: 'burst',
      segments: 3,
      separator: '+++',
      delays_ms: [0, 0, 0],
      streaming: 'batch',
    }
    let resolveStream: ((value: SendMessageResponse) => void) | undefined
    mocks.sendMessageStream.mockImplementation((
      _request: unknown,
      handlers: { onToken: (token: string, accumulated: string) => void },
    ) => {
      handlers.onToken('', 'first burst\n+++\nsecond burst\n+++\nthird burst')
      return new Promise<SendMessageResponse>((resolve) => {
        resolveStream = resolve
      })
    })
    const messages: ChatMessage[] = []

    const pending = sendChatStoreMessage(context(messages), 'hello', 'home')
    await vi.waitFor(() => expect(mocks.sendMessageStream).toHaveBeenCalledTimes(1))
    expect(messages.filter(message => message.role === 'assistant')).toHaveLength(0)

    expect(resolveStream).toBeTypeOf('function')
    resolveStream!({
      ...segmentedResponse(),
      reply_presentation: {
        segments: ['first burst', 'second burst', 'third burst'],
        delays_ms: [0, 0, 0],
      },
    })
    await pending

    expect(messages.filter(message => message.role === 'assistant')).toHaveLength(3)
    expect(mocks.emitBuiltin).not.toHaveBeenCalledWith(
      'com.oclive.voice:stream-sentence',
      expect.anything(),
    )
  })

  it('restarts final reply-mode delays after a failed live stream falls back', async () => {
    vi.useFakeTimers()
    mocks.streamEnabled = true
    mocks.roleStore.roleInfo.adultExtensionAvailable = false
    mocks.roleStore.roleInfo.replyMode = {
      mode: 'burst',
      segments: 3,
      separator: '+++',
      delays_ms: [0, 100, 100],
      streaming: 'live',
    }
    let rejectStream: ((reason: Error) => void) | undefined
    mocks.sendMessageStream.mockImplementation((
      _request: unknown,
      handlers: { onToken: (token: string, accumulated: string) => void },
    ) => {
      handlers.onToken('', 'first attempt\n+++\nsecond attempt\n+++\nthird attempt')
      return new Promise<SendMessageResponse>((_resolve, reject) => {
        rejectStream = reject
      })
    })
    mocks.sendMessage.mockResolvedValue(segmentedResponse())
    const messages: ChatMessage[] = []

    const pending = sendChatStoreMessage(context(messages), 'hello', 'home')
    await vi.advanceTimersByTimeAsync(100)
    expect(messages.filter(message => message.role === 'assistant')).toHaveLength(2)

    expect(rejectStream).toBeTypeOf('function')
    rejectStream!(new Error('stream disconnected'))
    await vi.advanceTimersByTimeAsync(0)
    expect(messages.filter(message => message.role === 'assistant')).toHaveLength(1)

    await vi.advanceTimersByTimeAsync(100)
    expect(messages.filter(message => message.role === 'assistant')).toHaveLength(2)
    await vi.advanceTimersByTimeAsync(100)
    await pending

    expect(messages.filter(message => message.role === 'assistant')).toHaveLength(3)
    expect(mocks.sendMessage).toHaveBeenCalledTimes(1)
  })

  it('keeps low-latency stream voice for ordinary single replies', async () => {
    mocks.streamEnabled = true
    mocks.roleStore.roleInfo.adultExtensionAvailable = false
    mocks.roleStore.roleInfo.replyMode = null
    mocks.sendMessageStream.mockImplementation(async (
      _request: unknown,
      handlers: { onToken: (token: string, accumulated: string) => void },
    ) => {
      handlers.onToken('', 'ordinary stream.')
      return ordinaryResponse()
    })
    const messages: ChatMessage[] = []

    await sendChatStoreMessage(context(messages), 'hello', 'home')

    expect(mocks.emitBuiltin).toHaveBeenCalledWith(
      'com.oclive.voice:stream-sentence',
      expect.objectContaining({ sentence: 'ordinary stream.' }),
    )
    const sentEvent = mocks.emitBuiltin.mock.calls.find(call => call[0] === 'message:sent')
    expect(sentEvent?.[1]?.stream_id).toEqual(expect.any(String))
  })
})
