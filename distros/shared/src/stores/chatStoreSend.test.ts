// @vitest-environment jsdom

import type { SendMessageResponse } from '@oclive/shared/api'
import type { ChatMessage } from './chatStore'
import type { ChatStoreSendContext } from './chatStoreSend'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cancelActiveChatSend, sendChatStoreMessage } from './chatStoreSend'

const mocks = vi.hoisted(() => ({
  sendMessage: vi.fn(),
  recoverMessage: vi.fn(),
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
    recoverMessage: mocks.recoverMessage,
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
    mocks.recoverMessage.mockReset()
    mocks.recoverMessage.mockResolvedValue(response())
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
    })
    expect(sentEvent?.[1]?.stream_id).toBeUndefined()
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
    mocks.recoverMessage.mockResolvedValue(segmentedResponse())
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
    expect(mocks.recoverMessage).toHaveBeenCalledTimes(1)
    expect(mocks.sendMessage).not.toHaveBeenCalled()
  })

  it('withholds ordinary stream voice until the authoritative final reply, then speaks it once', async () => {
    mocks.streamEnabled = true
    mocks.roleStore.roleInfo.adultExtensionAvailable = false
    mocks.roleStore.roleInfo.replyMode = null
    mocks.sendMessageStream.mockImplementation(async (
      _request: unknown,
      handlers: { onToken: (token: string, accumulated: string) => void },
    ) => {
      handlers.onToken('清晨的风', '清晨的风')
      handlers.onToken('轻轻吹过树梢。', '清晨的风轻轻吹过树梢。')
      return ordinaryResponse('清晨的风轻轻吹过树梢。')
    })
    const messages: ChatMessage[] = []

    await sendChatStoreMessage(context(messages), 'hello', 'home')

    // No voice fragment may leave before — or after — the final result confirms the turn.
    expect(mocks.emitBuiltin).not.toHaveBeenCalledWith(
      'com.oclive.voice:stream-sentence',
      expect.anything(),
    )
    const sent = mocks.emitBuiltin.mock.calls.filter(call => call[0] === 'message:sent')
    expect(sent).toHaveLength(1)
    expect(sent[0][1]).toMatchObject({
      reply: '清晨的风轻轻吹过树梢。',
      skip_auto_tts: false,
    })
    // Without a stream id the consumer speaks the full authoritative reply exactly once,
    // and it holds no streamed prefix/index it could use to drop part of the text.
    expect(sent[0][1].stream_id).toBeUndefined()
    expect(sent[0][1].stream_spoken_prefix).toBeUndefined()
    expect(sent[0][1].stream_full_raw).toBeUndefined()
    expect(sent[0][1].stream_spoken_end_index).toBeUndefined()
    const assistant = messages.find(message => message.role === 'assistant')
    expect(assistant).toMatchObject({
      id: 'assistant-1',
      content: '清晨的风轻轻吹过树梢。',
      streaming: false,
    })
  })

  it('replaces the token preview with the processed authoritative body and speaks that text', async () => {
    mocks.streamEnabled = true
    mocks.roleStore.roleInfo.adultExtensionAvailable = false
    mocks.roleStore.roleInfo.replyMode = null
    // The Host may clean or trim the reply, so the authoritative text is deliberately NOT
    // equal to the raw token prefix the bubble previewed.
    mocks.sendMessageStream.mockImplementation(async (
      _request: unknown,
      handlers: { onToken: (token: string, accumulated: string) => void },
    ) => {
      handlers.onToken('清晨的风', '清晨的风')
      handlers.onToken('轻轻吹过树梢。', '清晨的风轻轻吹过树梢。')
      return ordinaryResponse('清晨的风轻轻吹过树梢')
    })
    const messages: ChatMessage[] = []

    await sendChatStoreMessage(context(messages), 'hello', 'home')

    const assistants = messages.filter(message => message.role === 'assistant')
    expect(assistants).toHaveLength(1)
    expect(assistants[0]).toMatchObject({ id: 'assistant-1', content: '清晨的风轻轻吹过树梢' })
    expect(assistants[0].content).not.toBe('清晨的风轻轻吹过树梢。')
    const sent = mocks.emitBuiltin.mock.calls.filter(call => call[0] === 'message:sent')
    expect(sent).toHaveLength(1)
    // Voice input is the final processed text, not the raw token stream.
    expect(sent[0][1].reply).toBe('清晨的风轻轻吹过树梢')
  })

  it('keeps the wire free of voice when a failed stream recovers the same turn', async () => {
    mocks.streamEnabled = true
    mocks.roleStore.roleInfo.adultExtensionAvailable = false
    mocks.roleStore.roleInfo.replyMode = null
    mocks.sendMessageStream.mockImplementation(async (
      _request: unknown,
      handlers: { onToken: (token: string, accumulated: string) => void },
    ) => {
      handlers.onToken('partial', 'partial')
      throw new Error('stream ended without done event')
    })
    mocks.recoverMessage.mockResolvedValue(ordinaryResponse('recovered final'))
    const messages: ChatMessage[] = []

    await sendChatStoreMessage(context(messages), 'hello', 'home')

    expect(mocks.recoverMessage).toHaveBeenCalledTimes(1)
    expect(mocks.emitBuiltin).not.toHaveBeenCalledWith(
      'com.oclive.voice:stream-sentence',
      expect.anything(),
    )
    const sent = mocks.emitBuiltin.mock.calls.filter(call => call[0] === 'message:sent')
    expect(sent).toHaveLength(1)
    expect(sent[0][1]).toMatchObject({ reply: 'recovered final' })
    expect(sent[0][1].stream_id).toBeUndefined()
    const assistants = messages.filter(message => message.role === 'assistant')
    expect(assistants).toHaveLength(1)
    expect(assistants[0]).toMatchObject({ id: 'assistant-1', content: 'recovered final' })
  })

  it('publishes no voice and leaves no bubble when recovery fails', async () => {
    mocks.streamEnabled = true
    mocks.roleStore.roleInfo.adultExtensionAvailable = false
    mocks.roleStore.roleInfo.replyMode = null
    mocks.sendMessageStream.mockImplementation(async (
      _request: unknown,
      handlers: { onToken: (token: string, accumulated: string) => void },
    ) => {
      handlers.onToken('partial', 'partial')
      throw new Error('stream ended without done event')
    })
    const recoveryError = new Error('outcome unconfirmed')
    mocks.recoverMessage.mockRejectedValue(recoveryError)
    const messages: ChatMessage[] = []

    await expect(sendChatStoreMessage(context(messages), 'hello', 'home')).rejects.toBe(recoveryError)

    expect(mocks.emitBuiltin).not.toHaveBeenCalledWith(
      'com.oclive.voice:stream-sentence',
      expect.anything(),
    )
    expect(mocks.emitBuiltin.mock.calls.filter(call => call[0] === 'message:sent')).toHaveLength(0)
    expect(messages.filter(message => message.role === 'assistant')).toHaveLength(0)
  })

  it('keeps a voice-text-only turn silent for ordinary replies', async () => {
    mocks.streamEnabled = true
    mocks.roleStore.roleInfo.adultExtensionAvailable = false
    mocks.roleStore.roleInfo.replyMode = null
    mocks.adultStore.sessionFor.mockReturnValue({ active: true, voiceTextOnly: true, updatedAt: 1 })
    mocks.sendMessageStream.mockImplementation(async (
      _request: unknown,
      handlers: { onToken: (token: string, accumulated: string) => void },
    ) => {
      handlers.onToken('ordinary stream.', 'ordinary stream.')
      return ordinaryResponse()
    })
    const messages: ChatMessage[] = []

    try {
      await sendChatStoreMessage(context(messages), 'hello', 'home')

      expect(mocks.emitBuiltin).not.toHaveBeenCalledWith(
        'com.oclive.voice:stream-sentence',
        expect.anything(),
      )
      const sent = mocks.emitBuiltin.mock.calls.filter(call => call[0] === 'message:sent')
      expect(sent).toHaveLength(1)
      expect(sent[0][1]).toMatchObject({ skip_auto_tts: true })
      expect(sent[0][1].stream_id).toBeUndefined()
    }
    finally {
      mocks.adultStore.sessionFor.mockReturnValue({ active: true, voiceTextOnly: false, updatedAt: 1 })
    }
  })

  it('cP-INT B2 replaces partial stream bubbles with one authoritative retry response', async () => {
    mocks.streamEnabled = true
    mocks.roleStore.roleInfo.adultExtensionAvailable = false
    mocks.sendMessageStream.mockImplementation(async (
      _request: unknown,
      handlers: { onToken: (token: string, accumulated: string) => void },
    ) => {
      handlers.onToken('partial', 'partial')
      throw new Error('stream ended without done event')
    })
    mocks.recoverMessage.mockResolvedValue(ordinaryResponse('final retry'))
    const messages: ChatMessage[] = []
    const ctx = context(messages)
    await sendChatStoreMessage(ctx, 'hello', 'home')
    expect(mocks.sendMessageStream).toHaveBeenCalledTimes(1)
    expect(mocks.sendMessage).not.toHaveBeenCalled()
    expect(mocks.recoverMessage).toHaveBeenCalledTimes(1)
    expect(mocks.recoverMessage).toHaveBeenCalledWith({
      client_request_id: expect.any(String),
      role_id: 'role',
      user_message: 'hello',
      scene_id: 'home',
      adult: undefined,
    })
    expect(mocks.recoverMessage.mock.calls[0][0]).toBe(mocks.sendMessageStream.mock.calls[0][0])
    expect(mocks.recoverMessage.mock.calls[0][0].client_request_id).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/)
    expect(messages).toHaveLength(2)
    expect(messages[0]).toMatchObject({ id: 'user-1', role: 'user', content: 'hello' })
    expect(messages[1]).toMatchObject({ id: 'assistant-1', content: 'final retry', streaming: false })
    expect(mocks.emitBuiltin.mock.calls.filter(call => call[0] === 'message:sent')).toHaveLength(1)
    expect(ctx.setLoading).toHaveBeenLastCalledWith(false)
  })

  it('cp-int b2 cancellation aborts the owned transport without retry or success publication', async () => {
    mocks.streamEnabled = true
    mocks.roleStore.roleInfo.adultExtensionAvailable = false
    let capturedSignal: AbortSignal | undefined
    mocks.sendMessageStream.mockImplementation((
      _request: unknown,
      handlers: { signal: AbortSignal, onToken: (token: string, accumulated: string) => void },
    ) => {
      capturedSignal = handlers.signal
      handlers.onToken('partial', 'partial')
      return new Promise((_resolve, reject) => {
        handlers.signal.addEventListener('abort', () => reject(new DOMException('test abort', 'AbortError')))
      })
    })
    const messages: ChatMessage[] = []
    const pending = sendChatStoreMessage(context(messages), 'hello', 'home')
    await vi.waitFor(() => expect(capturedSignal).toBeDefined())
    cancelActiveChatSend()
    await pending
    expect(capturedSignal?.aborted).toBe(true)
    expect(mocks.sendMessage).not.toHaveBeenCalled()
    expect(mocks.recoverMessage).not.toHaveBeenCalled()
    expect(messages.filter(message => message.role === 'assistant')).toHaveLength(0)
    expect(mocks.updateLocal).not.toHaveBeenCalled()
    expect(mocks.emitBuiltin.mock.calls.filter(call => call[0] === 'message:sent')).toHaveLength(0)
  })

  it.each(['role', 'scene'])('cp-int b2 stale stream tokens and done are ignored after changing %s', async (changed) => {
    mocks.streamEnabled = true
    mocks.roleStore.roleInfo.adultExtensionAvailable = false
    let resolveStream: ((value: SendMessageResponse) => void) | undefined
    let onToken: ((token: string, accumulated: string) => void) | undefined
    mocks.sendMessageStream.mockImplementation((
      _request: unknown,
      handlers: { onToken: (token: string, accumulated: string) => void },
    ) => {
      onToken = handlers.onToken
      return new Promise<SendMessageResponse>((resolve) => {
        resolveStream = resolve
      })
    })
    const messages: ChatMessage[] = []
    const ctx = context(messages)
    const pending = sendChatStoreMessage(ctx, 'hello', 'home')
    await vi.waitFor(() => expect(resolveStream).toBeDefined())
    if (changed === 'role')
      mocks.roleStore.currentRoleId = 'other-role'
    else
      mocks.uiStore.sceneId = 'garden'
    onToken!('late', 'late')
    resolveStream!(ordinaryResponse('late final'))
    await pending
    expect(ctx.patchMessageById).not.toHaveBeenCalled()
    expect(ctx.setLoading).not.toHaveBeenCalledWith(false)
    expect(mocks.sendMessage).not.toHaveBeenCalled()
    expect(mocks.recoverMessage).not.toHaveBeenCalled()
    expect(messages.filter(message => message.role === 'assistant')).toHaveLength(0)
    expect(mocks.updateLocal).not.toHaveBeenCalled()
    expect(mocks.updateRelation).not.toHaveBeenCalled()
    expect(mocks.emitBuiltin.mock.calls.filter(call => call[0] === 'message:sent')).toHaveLength(0)
  })

  it('never starts a new send when recovery fails and gives new intentional sends new IDs', async () => {
    mocks.streamEnabled = true
    mocks.roleStore.roleInfo.adultExtensionAvailable = false
    mocks.sendMessageStream.mockRejectedValue(new Error('connection lost'))
    const recoveryError = new Error('outcome unconfirmed')
    mocks.recoverMessage.mockRejectedValue(recoveryError)
    const messages: ChatMessage[] = []
    const firstContext = context(messages)
    await expect(sendChatStoreMessage(firstContext, 'same', 'home')).rejects.toBe(recoveryError)
    expect(firstContext.setLoading).toHaveBeenLastCalledWith(false)
    expect(messages).toHaveLength(0)
    const secondContext = context(messages)
    await expect(sendChatStoreMessage(secondContext, 'same', 'home')).rejects.toBe(recoveryError)
    expect(secondContext.setLoading).toHaveBeenLastCalledWith(false)
    expect(messages).toHaveLength(0)
    expect(mocks.sendMessageStream).toHaveBeenCalledTimes(2)
    expect(mocks.sendMessage).not.toHaveBeenCalled()
    expect(mocks.recoverMessage).toHaveBeenCalledTimes(2)
    expect(mocks.recoverMessage.mock.calls[0][0].client_request_id).not.toBe(mocks.recoverMessage.mock.calls[1][0].client_request_id)
    expect(mocks.recoverMessage.mock.calls[0][0]).toBe(mocks.sendMessageStream.mock.calls[0][0])
    expect(mocks.recoverMessage.mock.calls[1][0]).toBe(mocks.sendMessageStream.mock.calls[1][0])
    expect(mocks.updateLocal).not.toHaveBeenCalled()
    expect(mocks.updateRelation).not.toHaveBeenCalled()
    expect(mocks.emitBuiltin.mock.calls.filter(call => call[0] === 'message:sent')).toHaveLength(0)
  })
})
