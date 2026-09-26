// @vitest-environment jsdom
//
// CP-INT R2：桌面流式传输已改由**持有令牌的 Rust 客户端**承载。
//
// 本文件覆盖渲染层这一侧的合同：
//   * `sendMessageStream` 通过 Tauri IPC（`send_message_stream` + `Channel`）交付，
//     不再 `fetch`（Host 的路由要求 `x-oclive-api-token`，渲染层没有也不该有它）；
//   * token 顺序与累计文本、终态 DTO、错误码、主动取消与意外断流的区分；
//   * 传输实例句柄（`transportId`）与业务 `client_request_id` 分离；
//   * 非 Tauri 宿主是**明确不支持**，而不是静默退回无令牌 fetch。
//
// SSE 的字节级解析（分块、UTF-8 边界、终止语义）已移到 Rust，由
// `distros/desktop-tauri/src/kernel_attach/chat.rs` 的 `stream_decoder_tests` 覆盖。

import type { SendMessageResponse } from './chat'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { recoverMessage, sendMessageStream } from './chat'
import { ApiInvokeError } from './helpers'

interface RendererChannel {
  onmessage: ((event: unknown) => void) | null
}

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  channels: [] as RendererChannel[],
}))

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

const request = { role_id: 'role', user_message: '你好', scene_id: 'home', session_id: 'session' }
const identified = { ...request, client_request_id: 'ae45c9f0-5a98-4ac9-a103-112c1caa1526' }
const final = {
  reply: '最终台词',
  user_message_id: 'u1',
  assistant_message_id: 'a1',
} as unknown as SendMessageResponse

interface StreamCall {
  req: Record<string, unknown>
  transportId: string
  channel: RendererChannel
  resolve: (value: SendMessageResponse) => void
  reject: (error: unknown) => void
}

function installInvoke(): StreamCall[] {
  const calls: StreamCall[] = []
  mocks.invoke.mockImplementation(async (command: string, payload?: Record<string, unknown>) => {
    if (command === 'send_message_stream') {
      return await new Promise<SendMessageResponse>((resolve, reject) => {
        calls.push({
          req: (payload?.req ?? {}) as Record<string, unknown>,
          transportId: String(payload?.transportId ?? ''),
          channel: payload?.onEvent as RendererChannel,
          resolve,
          reject,
        })
      })
    }
    if (command === 'cancel_message_stream')
      return true
    if (command === 'recover_message')
      return final
    throw new Error(`unexpected invoke ${command}`)
  })
  return calls
}

describe('cP-INT R2 authenticated IPC stream client', () => {
  let calls: StreamCall[] = []

  beforeEach(() => {
    vi.clearAllMocks()
    mocks.channels.length = 0
    calls = installInvoke()
    vi.stubGlobal('__TAURI_INTERNALS__', { invoke: mocks.invoke })
  })
  afterEach(() => vi.unstubAllGlobals())

  it('delivers ordered token events on the IPC channel and resolves with the terminal DTO', async () => {
    const onToken = vi.fn()
    const abort = new AbortController()
    const pending = sendMessageStream(request, { onToken, signal: abort.signal })

    await vi.waitFor(() => expect(calls).toHaveLength(1))
    const call = calls[0]!
    expect(call.req).toEqual(request)
    expect(call.transportId.startsWith('chat-stream-')).toBe(true)
    expect(call.transportId).not.toBe(call.req.client_request_id)

    call.channel.onmessage?.({ kind: 'token', token: '你', accumulated: '你' })
    call.channel.onmessage?.({ kind: 'token', token: '好', accumulated: '你好' })
    // Tokens are the only thing the channel carries; the DTO is the typed return value, so it is
    // necessarily delivered after every token.
    expect(onToken.mock.calls).toEqual([['你', '你'], ['好', '你好']])

    call.resolve(final)
    await expect(pending).resolves.toBe(final)
    expect(mocks.invoke.mock.calls.map(entry => entry[0])).toEqual(['send_message_stream'])
  })

  it('preserves the shared turn identity in the stream request and keeps recovery on the same one', async () => {
    const pending = sendMessageStream(identified)
    await vi.waitFor(() => expect(calls).toHaveLength(1))
    expect(calls[0]!.req.client_request_id).toBe(identified.client_request_id)
    calls[0]!.resolve(final)
    await pending

    expect(await recoverMessage(identified)).toEqual(final)
    expect(mocks.invoke).toHaveBeenLastCalledWith('recover_message', { req: identified })
    expect(mocks.invoke.mock.calls.filter(entry => entry[0] === 'send_message_stream')).toHaveLength(1)
  })

  it('passes a legacy request without a turn id through untouched instead of inventing one', async () => {
    const legacy = { ...request }
    const pending = sendMessageStream(legacy)
    await vi.waitFor(() => expect(calls).toHaveLength(1))
    expect(calls[0]!.req.client_request_id).toBeUndefined()
    calls[0]!.resolve(final)
    await pending
  })

  it('surfaces the kernel error code from the command rejection', async () => {
    const pending = sendMessageStream(request)
    await vi.waitFor(() => expect(calls).toHaveLength(1))
    calls[0]!.reject(new Error(JSON.stringify({
      code: 'LLM_ERROR',
      message: 'synthetic provider failure',
    })))

    const failure = await pending.then(
      () => {
        throw new Error('stream must not resolve')
      },
      (error: unknown) => error,
    )
    expect(failure).toBeInstanceOf(ApiInvokeError)
    expect((failure as ApiInvokeError).code).toBe('LLM_ERROR')
  })

  it('cancels the transport instance through its own handle and reports an AbortError', async () => {
    const abort = new AbortController()
    const pending = sendMessageStream(request, { signal: abort.signal })
    await vi.waitFor(() => expect(calls).toHaveLength(1))
    const transportId = calls[0]!.transportId

    abort.abort()
    await vi.waitFor(() => expect(
      mocks.invoke.mock.calls.some(entry => entry[0] === 'cancel_message_stream'),
    ).toBe(true))
    const cancelCall = mocks.invoke.mock.calls.find(entry => entry[0] === 'cancel_message_stream')
    expect(cancelCall?.[1]).toEqual({ transportId })
    expect(cancelCall?.[1]).not.toEqual({ req: expect.anything() })

    calls[0]!.reject(new Error('CHAT_STREAM_CANCELLED: aborted'))
    const failure = await pending.then(
      () => {
        throw new Error('cancelled stream must not resolve')
      },
      (error: unknown) => error,
    )
    expect(failure).toBeInstanceOf(DOMException)
    expect((failure as DOMException).name).toBe('AbortError')
  })

  it('drops tokens that arrive after the cancel instead of leaking them into the UI', async () => {
    const abort = new AbortController()
    const onToken = vi.fn()
    const pending = sendMessageStream(request, { onToken, signal: abort.signal })
    await vi.waitFor(() => expect(calls).toHaveLength(1))

    calls[0]!.channel.onmessage?.({ kind: 'token', token: '先到', accumulated: '先到' })
    abort.abort()
    calls[0]!.channel.onmessage?.({ kind: 'token', token: '迟到', accumulated: '先到迟到' })
    calls[0]!.reject(new Error('CHAT_STREAM_CANCELLED: aborted'))
    await expect(pending).rejects.toBeInstanceOf(DOMException)

    expect(onToken.mock.calls).toEqual([['先到', '先到']])
  })

  it('rejects an already-aborted signal without opening a transport', async () => {
    const abort = new AbortController()
    abort.abort()
    await expect(sendMessageStream(request, { signal: abort.signal })).rejects.toBeInstanceOf(DOMException)
    expect(mocks.invoke).not.toHaveBeenCalled()
    expect(mocks.channels).toHaveLength(0)
  })

  it('refuses to fall back to an unauthenticated transport outside the desktop host', async () => {
    vi.unstubAllGlobals()
    const fetchSpy = vi.fn()
    vi.stubGlobal('fetch', fetchSpy)

    await expect(sendMessageStream(request)).rejects.toThrow(
      'sendMessageStream requires the desktop host',
    )
    expect(fetchSpy).not.toHaveBeenCalled()
    expect(mocks.invoke).not.toHaveBeenCalled()
    expect(mocks.channels).toHaveLength(0)
  })
})
