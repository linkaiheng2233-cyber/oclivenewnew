// @vitest-environment jsdom
//
// CP-INT R2：桌面**已鉴权 IPC 流式传输** ↔ Tauri IPC 恢复之间的**请求身份连接**边界证据。
//
// 与既有单元/集成测试的区别（本文件的存在理由）：
//   * `sendChatStoreMessage`（真实 store 流程）、`sendMessageStream`（真实 IPC 客户端与传输句柄）、
//     `recoverMessage`（真实 invoke 参数构造与错误映射）、`invokeWithFriendlyError`
//     （真实错误解析 / `ApiInvokeError`）全部是真实实现；
//   * 只有两个外部边界被内存替身替换：`@tauri-apps/api/core` 的 `invoke` 与 `Channel`；
//     另有 role/adult/ui/debug 四个非权威 store 替身（不参与本 seam 的判定）。
//   * 流式断流形状可在途控制：首个 token 之后既无 done，随后桥失败或桥直接结束。
//
// **边界变更（R1 → R2）**：渲染层不再 `fetch /chat/stream`。Host 把该路由放在令牌中间件下，而令牌
// 只存在于 Rust（`KernelConnection::http_client`），因此 SSE 的字节级解析已移到
// `distros/desktop-tauri/src/kernel_attach/chat.rs` 的 `ChatStreamDecoder`，其分块/UTF-8 边界/
// 终止语义由该文件的 `stream_decoder_tests` 直接覆盖。本文件覆盖的是它上面那一层：IPC 通道事件
// 顺序、终态 DTO、取消与意外断流的区分、以及同身份恢复。
//
// 边界：内存 IPC。**不代表**真实 Tauri 进程、真实桌面联机或真实内核 HTTP 往返。

import type { SendMessageResponse } from '@oclive/shared/api'
import type { ChatMessage } from '../stores/chatStore'
import type { ChatStoreSendContext } from '../stores/chatStoreSend'
import { i18n } from '@oclive/shared/i18n/index'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { hostEventBus } from '../lib/hostEventBus'
import { sendChatStoreMessage } from '../stores/chatStoreSend'

interface Deferred<T> {
  promise: Promise<T>
  resolve: (value: T) => void
  reject: (error: unknown) => void
}

function deferred<T>(): Deferred<T> {
  let resolve!: (value: T) => void
  let reject!: (error: unknown) => void
  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

interface RendererChannel {
  onmessage: ((event: unknown) => void) | null
}

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  channels: [] as RendererChannel[],
  cancelQueue: vi.fn(),
  recordKnowledge: vi.fn(),
  updateLocal: vi.fn(),
  updateRelation: vi.fn(),
  uiStore: { sceneId: 'home' },
  roleStore: {
    currentRoleId: 'mumu',
    updateLocalAfterMessage: vi.fn(),
    updateRelationState: vi.fn(),
    roleInfo: {
      adultExtensionAvailable: false,
      relationState: 'Friend',
      replyMode: null,
    },
  },
  adultStore: {
    backgroundQueueEnabled: false,
    requestFor: vi.fn(() => undefined),
    sessionFor: vi.fn(() => ({ active: true, voiceTextOnly: false, updatedAt: 1 })),
    updateSession: vi.fn(),
  },
}))

// 真实 Tauri IPC 边界：只替换 `invoke` 与 `Channel`，保留该模块其余真实导出。
vi.mock('@tauri-apps/api/core', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@tauri-apps/api/core')>()
  return {
    ...actual,
    invoke: mocks.invoke,
    Channel: class {
      onmessage: ((event: unknown) => void) | null = null
      constructor() {
        mocks.channels.push(this as unknown as RendererChannel)
      }
    },
  }
})

vi.mock('@oclive/shared/lib/adultBeatQueue', () => ({
  cancelAdultBeatQueue: mocks.cancelQueue,
  resumeAdultBeatQueue: vi.fn(),
  startAdultBeatQueue: vi.fn(),
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

/** 一次明确的用户意图文本；恢复必须复用同一 turn，而不是重发这段文本。 */
const USER_TEXT = '聊聊清晨的风景吧。'
/** 提供方在失败前已经吐出的暂态前缀。 */
const STALE_PREFIX = '清晨的风'
/** 恢复路径返回的权威最终台词。 */
const RECOVERED_REPLY = '这就是同一次请求的最终结果。'
const CANONICAL_UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/
const NIL_UUID = '00000000-0000-0000-0000-000000000000'

interface StreamCall {
  req: Record<string, unknown>
  transportId: string
  channel: RendererChannel
  settle: Deferred<SendMessageResponse>
}

function response(reply: string, assistantId: string): SendMessageResponse {
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
    scene_id: 'home',
    offer_destination_picker: false,
    offer_together_travel: false,
    reply_is_fallback: false,
    knowledge_chunks_in_prompt: 0,
    timestamp: 1,
    user_message_id: `user-${assistantId}`,
    assistant_message_id: assistantId,
    user_message_timestamp: '2026-09-25T00:00:00.000Z',
    assistant_message_timestamp: '2026-09-25T00:00:01.000Z',
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

interface InvokeCall {
  command: string
  payload: Record<string, unknown> | undefined
}

describe('cP-INT R2 authenticated IPC stream transport → Tauri recovery identity seam', () => {
  let invokeCalls: InvokeCall[] = []
  let streamCalls: StreamCall[] = []
  let streamAccumulated: string[] = []
  let recovery = deferred<SendMessageResponse>()
  let sentEvents: Record<string, unknown>[] = []
  let submitEvents: Record<string, unknown>[] = []

  const onSent = (payload: unknown) => {
    sentEvents.push(payload as Record<string, unknown>)
  }
  const onSubmit = (payload: unknown) => {
    submitEvents.push(payload as Record<string, unknown>)
  }

  const commands = (command: string) => invokeCalls.filter(call => call.command === command)
  const payloadOf = (command: string) => commands(command)[0]?.payload
  const assistants = (messages: ChatMessage[]) => messages.filter(item => item.role === 'assistant')
  const users = (messages: ChatMessage[]) => messages.filter(item => item.role === 'user')

  /** 第 `index` 条流式传输的可控句柄：token 经真实 IPC 通道投递，终态由 invoke 决定。 */
  function ipcTransport(index: number) {
    return {
      token(text: string) {
        const call = streamCalls[index]
        if (!call)
          throw new Error(`no stream call #${index + 1}`)
        streamAccumulated[index] = `${streamAccumulated[index] ?? ''}${text}`
        call.channel.onmessage?.({
          kind: 'token',
          token: text,
          accumulated: streamAccumulated[index],
        })
      },
      done(value: SendMessageResponse) {
        streamCalls[index]!.settle.resolve(value)
      },
      /** 桥在流中途失败：invoke reject（Rust 侧分块/EOF 语义见 chat.rs 的 stream_decoder_tests）。 */
      fail(error: Error) {
        streamCalls[index]!.settle.reject(error)
      },
      /** 取消后 Rust 侧的终态：命令以错误结束。 */
      rejectAfterCancel() {
        streamCalls[index]!.settle.reject(new Error('CHAT_STREAM_CANCELLED: aborted'))
      },
    }
  }

  beforeEach(() => {
    vi.clearAllMocks()
    invokeCalls = []
    streamCalls = []
    streamAccumulated = []
    sentEvents = []
    submitEvents = []
    recovery = deferred<SendMessageResponse>()
    mocks.channels.length = 0

    mocks.roleStore.currentRoleId = 'mumu'
    mocks.roleStore.roleInfo.adultExtensionAvailable = false
    mocks.roleStore.roleInfo.replyMode = null
    mocks.uiStore.sceneId = 'home'
    mocks.cancelQueue.mockResolvedValue(undefined)

    // 真实 Tauri IPC 边界替身：按命令分派并记录每一次调用。
    mocks.invoke.mockImplementation(
      async (command: string, payload?: Record<string, unknown>) => {
        invokeCalls.push({ command, payload })
        if (command === 'send_message_stream') {
          const settle = deferred<SendMessageResponse>()
          streamCalls.push({
            req: (payload?.req ?? {}) as Record<string, unknown>,
            transportId: String(payload?.transportId ?? ''),
            channel: payload?.onEvent as RendererChannel,
            settle,
          })
          return settle.promise
        }
        if (command === 'cancel_message_stream')
          return true
        if (command === 'recover_message')
          return recovery.promise
        throw new Error(`unexpected invoke ${command}`)
      },
    )

    // 已鉴权的桌面宿主：`sendMessageStream` 明确要求渲染层运行在 Tauri 内。
    vi.stubGlobal('__TAURI_INTERNALS__', { invoke: mocks.invoke })

    // 真实流式开关由 localStorage 驱动；jsdom 初始无该键，
    // 因此本文件覆盖的是生产默认（`/chat/stream` 开启）路径。
    expect(localStorage.getItem('oclive.chat.streamEnabled')).toBeNull()

    hostEventBus.on('message:sent', onSent)
    hostEventBus.on('message:submit', onSubmit)
  })

  afterEach(() => {
    hostEventBus.off('message:sent', onSent)
    hostEventBus.off('message:submit', onSubmit)
    vi.unstubAllGlobals()
  })

  it('r2 shape: a failed stream bridge recovers the same turn ID and never starts a new send', async () => {
    const transport = ipcTransport(0)
    const messages: ChatMessage[] = []

    const pending = sendChatStoreMessage(context(messages), USER_TEXT, 'home')
    await vi.waitFor(() => expect(commands('send_message_stream')).toHaveLength(1))
    const streamRequest = payloadOf('send_message_stream')?.req as Record<string, unknown>

    transport.token(STALE_PREFIX)
    // ---- 在途：暂态前缀只是视觉预览，不是最终助手行，也不产生 message:sent ----
    await vi.waitFor(() => expect(assistants(messages)).toHaveLength(1))
    expect(assistants(messages)[0]).toMatchObject({ content: STALE_PREFIX, streaming: true })
    expect(sentEvents).toEqual([])

    transport.fail(new TypeError('synthetic authenticated bridge failure'))
    await vi.waitFor(() => expect(commands('recover_message')).toHaveLength(1))

    // ---- 断流后、恢复结果在途：暂态气泡已撤，零最终助手行、零 message:sent ----
    expect(assistants(messages)).toEqual([])
    expect(sentEvents).toEqual([])
    expect(users(messages)).toHaveLength(1)
    expect(commands('send_message_stream')).toHaveLength(1)
    // 角色路径由持有令牌的 Rust 命令解析，渲染层不再需要它。
    expect(commands('get_role_pack_path')).toEqual([])

    // ---- 请求身份：同一 turn 的两个传输必须携带同一个规范 UUID ----
    const streamTurnId = streamRequest.client_request_id
    const recoverRequest = payloadOf('recover_message')?.req as Record<string, unknown>
    expect(typeof streamTurnId).toBe('string')
    expect(CANONICAL_UUID.test(String(streamTurnId))).toBe(true)
    expect(streamTurnId).not.toBe(NIL_UUID)
    expect(recoverRequest.client_request_id).toBe(streamTurnId)
    // 传输实例句柄必须与业务身份分离。
    const transportId = String(payloadOf('send_message_stream')?.transportId ?? '')
    expect(transportId).not.toBe(streamTurnId)
    expect(transportId.startsWith('chat-stream-')).toBe(true)
    expect(recoverRequest.role_id).toBe('mumu')
    expect(streamRequest.user_message).toBe(USER_TEXT)
    expect(recoverRequest.user_message).toBe(USER_TEXT)
    expect(streamRequest.scene_id).toBe('home')
    expect(recoverRequest.scene_id).toBe('home')
    // 线的归一化现在发生在 Rust 的 typed `SendMessageRequest` 里：缺省与显式 null 都反序列化为
    // `None`，因此渲染层不再需要把 undefined 改写成 null（该等价由 Rust 桥合同测试证明）。
    expect(streamRequest.adult).toBeUndefined()
    expect(recoverRequest.session_id).toBeUndefined()
    expect(recoverRequest.adult).toBeUndefined()
    // 恢复不是新意图：没有 send_message、没有第二个 UUID。
    expect(commands('send_message')).toEqual([])
    expect(streamTurnId).toBe(recoverRequest.client_request_id)

    recovery.resolve(response(RECOVERED_REPLY, 'assistant-recovered'))
    const settled = await pending

    // ---- 完成后：权威 DTO 只落一条最终气泡、一次 message:sent，且 ID 与恢复 DTO 绑定 ----
    expect(settled?.assistant_message_id).toBe('assistant-recovered')
    const final = assistants(messages)
    expect(final).toHaveLength(1)
    expect(final[0]).toMatchObject({
      id: 'assistant-recovered',
      content: RECOVERED_REPLY,
      streaming: false,
    })
    expect(final[0].content).not.toBe(STALE_PREFIX)
    expect(users(messages)).toHaveLength(1)
    expect(users(messages)[0].id).toBe('user-assistant-recovered')
    expect(sentEvents).toHaveLength(1)
    expect(sentEvents[0]).toMatchObject({
      role_id: 'mumu',
      scene_id: 'home',
      reply: RECOVERED_REPLY,
      skip_auto_tts: false,
    })
    expect(submitEvents).toHaveLength(1)
    expect(commands('recover_message')).toHaveLength(1)
    expect(commands('send_message')).toEqual([])
  })

  it('r2 shape: a bridge that ends after the first token is the same recovery, not a new turn', async () => {
    const transport = ipcTransport(0)
    const messages: ChatMessage[] = []

    const pending = sendChatStoreMessage(context(messages), USER_TEXT, 'home')
    await vi.waitFor(() => expect(commands('send_message_stream')).toHaveLength(1))
    const streamRequest = payloadOf('send_message_stream')?.req as Record<string, unknown>

    transport.token(STALE_PREFIX)
    await vi.waitFor(() => expect(assistants(messages)).toHaveLength(1))
    transport.fail(new Error('remote chat stream ended without done event'))

    await vi.waitFor(() => expect(commands('recover_message')).toHaveLength(1))
    const recoverRequest = payloadOf('recover_message')?.req as Record<string, unknown>
    expect(recoverRequest.client_request_id).toBe(streamRequest.client_request_id)
    expect(assistants(messages)).toEqual([])
    expect(commands('send_message_stream')).toHaveLength(1)

    recovery.resolve(response(RECOVERED_REPLY, 'assistant-eof'))
    await pending

    expect(assistants(messages)).toHaveLength(1)
    expect(assistants(messages)[0]).toMatchObject({ id: 'assistant-eof', content: RECOVERED_REPLY })
    expect(sentEvents).toHaveLength(1)
    expect(commands('send_message')).toEqual([])
  })

  it('keeps a recovery conflict identifiable and starts nothing', async () => {
    const transport = ipcTransport(0)
    const messages: ChatMessage[] = []

    const pending = sendChatStoreMessage(context(messages), USER_TEXT, 'home')
    await vi.waitFor(() => expect(commands('send_message_stream')).toHaveLength(1))
    transport.token(STALE_PREFIX)
    await vi.waitFor(() => expect(assistants(messages)).toHaveLength(1))
    transport.fail(new TypeError('synthetic authenticated bridge failure'))
    await vi.waitFor(() => expect(commands('recover_message')).toHaveLength(1))
    // 真实 Tauri 命令以序列化后的内核错误 JSON 字符串 reject。
    recovery.reject(new Error(JSON.stringify({
      code: 'CHAT_REQUEST_CONFLICT',
      message: 'client_request_id was reused with a different payload',
    })))

    await expect(pending).rejects.toMatchObject({ code: 'CHAT_REQUEST_CONFLICT' })
    expect(commands('send_message')).toEqual([])
    expect(commands('send_message_stream')).toHaveLength(1)
    expect(commands('recover_message')).toHaveLength(1)
    expect(assistants(messages)).toEqual([])
    expect(users(messages)).toEqual([])
    expect(sentEvents).toEqual([])
  })

  it('turns an old host without the command into an unconfirmed boundary, never a resend', async () => {
    const transport = ipcTransport(0)
    const messages: ChatMessage[] = []

    const pending = sendChatStoreMessage(context(messages), USER_TEXT, 'home')
    await vi.waitFor(() => expect(commands('send_message_stream')).toHaveLength(1))
    transport.token(STALE_PREFIX)
    await vi.waitFor(() => expect(assistants(messages)).toHaveLength(1))
    transport.fail(new Error('remote chat stream ended without done event'))
    await vi.waitFor(() => expect(commands('recover_message')).toHaveLength(1))
    recovery.reject(new Error('Command recover_message not found'))

    const failure = await pending.then(
      () => {
        throw new Error('recovery must not resolve')
      },
      (error: unknown) => error as Error & { cause?: unknown },
    )
    // 可确认性边界：界面看到的是映射后的 UNCONFIRMED 文案，原始 IPC 文本只留在 cause。
    expect(i18n.global.te('apiErrors.CHAT_REQUEST_UNCONFIRMED')).toBe(true)
    expect(failure.message).toBe(String(i18n.global.t('apiErrors.CHAT_REQUEST_UNCONFIRMED')))
    expect(failure.message).not.toContain('Command recover_message not found')
    expect(failure.cause).toBeInstanceOf(Error)
    expect(String((failure.cause as Error).message)).toContain('Command recover_message not found')
    expect(commands('send_message')).toEqual([])
    expect(commands('send_message_stream')).toHaveLength(1)
    expect(assistants(messages)).toEqual([])
    expect(users(messages)).toEqual([])
    expect(sentEvents).toEqual([])
  })

  it('gives an explicit second intent a new turn ID while the automatic recovery reused the first', async () => {
    const first = ipcTransport(0)
    const messages: ChatMessage[] = []

    const firstSend = sendChatStoreMessage(context(messages), USER_TEXT, 'home')
    await vi.waitFor(() => expect(commands('send_message_stream')).toHaveLength(1))
    first.token(STALE_PREFIX)
    await vi.waitFor(() => expect(assistants(messages)).toHaveLength(1))
    first.fail(new Error('remote chat stream ended without done event'))
    await vi.waitFor(() => expect(commands('recover_message')).toHaveLength(1))
    recovery.resolve(response(RECOVERED_REPLY, 'assistant-first'))
    await firstSend
    const firstTurnId = (payloadOf('recover_message')?.req as Record<string, unknown>)
      .client_request_id
    const recoverCommandCount = commands('recover_message').length

    // 同一文本、明确的**新意图**：走完整 done 的流式回合，不触发任何恢复。
    const second = ipcTransport(1)
    const secondSend = sendChatStoreMessage(context(messages), USER_TEXT, 'home')
    await vi.waitFor(() => expect(commands('send_message_stream')).toHaveLength(2))
    second.token(STALE_PREFIX)
    second.token('（同一文本的新意图）')
    second.done(response(`${STALE_PREFIX}（同一文本的新意图）`, 'assistant-second'))
    await secondSend

    const requests = commands('send_message_stream').map(call => call.payload?.req as Record<string, unknown>)
    const secondTurnId = requests[1]!.client_request_id
    expect(requests[0]!.client_request_id).toBe(firstTurnId)
    expect(secondTurnId).toMatch(CANONICAL_UUID)
    expect(secondTurnId).not.toBe(firstTurnId)
    expect(new Set([firstTurnId, secondTurnId]).size).toBe(2)
    // 两个传输实例句柄也必须互异。
    const transportIds = commands('send_message_stream').map(call => String(call.payload?.transportId))
    expect(new Set(transportIds).size).toBe(2)
    // 显式新意图自带 done，因此没有第二次恢复 invoke：自动恢复与新发送严格区分。
    expect(commands('recover_message')).toHaveLength(recoverCommandCount)
    expect(commands('send_message')).toEqual([])
    expect(assistants(messages).map(item => item.id))
      .toEqual(['assistant-first', 'assistant-second'])
  })

  it('cancels only this transport instance and drops tokens that arrive after the cancel', async () => {
    const transport = ipcTransport(0)
    const messages: ChatMessage[] = []

    const pending = sendChatStoreMessage(context(messages), USER_TEXT, 'home')
    await vi.waitFor(() => expect(commands('send_message_stream')).toHaveLength(1))
    const transportId = String(payloadOf('send_message_stream')?.transportId ?? '')
    transport.token(STALE_PREFIX)
    await vi.waitFor(() => expect(assistants(messages)).toHaveLength(1))

    // 真实产品动作：主动取消。
    const { cancelActiveChatSend } = await import('../stores/chatStoreSend')
    cancelActiveChatSend()
    await vi.waitFor(() => expect(commands('cancel_message_stream')).toHaveLength(1))
    expect(payloadOf('cancel_message_stream')?.transportId).toBe(transportId)
    // 取消后 Rust 侧以错误结束该传输实例。
    transport.rejectAfterCancel()
    await pending

    // 主动取消不触发恢复；迟到 token 不得污染界面。
    transport.token('迟到片段')
    await Promise.resolve()
    expect(commands('recover_message')).toEqual([])
    expect(commands('send_message')).toEqual([])
    expect(assistants(messages)).toEqual([])
    expect(sentEvents).toEqual([])
  })
})
