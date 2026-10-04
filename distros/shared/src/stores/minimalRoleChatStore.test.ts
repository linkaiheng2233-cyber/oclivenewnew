// @vitest-environment jsdom
import type { MinimalRoleLocalSource, MinimalRoleMessageResponse } from '@oclive/shared/api/chat'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useMinimalRoleChatStore } from './minimalRoleChatStore'

const mocks = vi.hoisted(() => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/core', async importOriginal => ({
  ...await importOriginal<typeof import('@tauri-apps/api/core')>(),
  invoke: mocks.invoke,
}))

function source(id = 'minimal-fixture'): MinimalRoleLocalSource {
  return { role_id: id, asset_root: 'E:/synthetic-fixture', definition_reference: 'chosen.json' }
}

function result(id = 'minimal-fixture', reply = '  [EMO] untouched\r\n') {
  return { role_id: id, reply, product_extensions: 'unavailable' } as const
}

function deferred() {
  let resolve!: (value: MinimalRoleMessageResponse) => void
  let reject!: (reason: unknown) => void
  const promise = new Promise<MinimalRoleMessageResponse>((yes, no) => {
    resolve = yes
    reject = no
  })
  return { promise, resolve, reject }
}

const submitted = vi.fn()
const sent = vi.fn()

beforeEach(() => {
  setActivePinia(createPinia())
  mocks.invoke.mockReset()
  submitted.mockClear()
  sent.mockClear()
  hostEventBus.on('message:submit', submitted)
  hostEventBus.on('message:sent', sent)
})
afterEach(() => {
  hostEventBus.off('message:submit', submitted)
  hostEventBus.off('message:sent', sent)
})

describe('transient minimal role chat consumption', () => {
  it('binds a source snapshot and consumes one raw reply without rich state or stored IDs', async () => {
    const pending = deferred()
    mocks.invoke.mockReturnValueOnce(pending.promise)
    const store = useMinimalRoleChatStore()
    const input = source()
    store.bindSource(input)
    input.asset_root = 'E:/mutated-caller'
    const sending = store.sendMessage('  original user\r\n')
    expect(store.isLoading).toBe(true)
    expect(store.messages).toHaveLength(1)
    expect(store.messages[0]?.content).toBe('  original user\r\n')
    expect(sent).not.toHaveBeenCalled()
    expect(submitted).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({ skip_auto_tts: true }))
    const response = result()
    pending.resolve(response)
    expect(await sending).toBe(response)
    expect(store.isLoading).toBe(false)
    expect(store.messages.map(message => message.content)).toEqual(['  original user\r\n', response.reply])
    expect(store.messages[1]).toEqual({
      id: expect.stringMatching(/^minimal-local-assistant-/),
      role: 'assistant',
      content: response.reply,
      timestamp: expect.any(Number),
    })
    expect(mocks.invoke).toHaveBeenCalledExactlyOnceWith('send_minimal_message', {
      req: { source: source(), message: { user_message: '  original user\r\n', requirements: '' } },
    })
    expect(sent).toHaveBeenCalledExactlyOnceWith({
      message: '  original user\r\n',
      reply: response.reply,
      role_id: 'minimal-fixture',
      product_extensions: 'unavailable',
      turn_id: expect.stringMatching(/^minimal-local-turn-/),
      skip_auto_tts: true,
    })
    expect(store.$state).not.toHaveProperty('roleInfo')
  })

  it('preserves normally empty model output and previous completed messages', async () => {
    const store = useMinimalRoleChatStore()
    store.bindSource(source())
    mocks.invoke.mockResolvedValueOnce(result()).mockResolvedValueOnce(result('minimal-fixture', ''))
    await store.sendMessage('first')
    await store.sendMessage('second')
    expect(store.messages).toHaveLength(4)
    expect(store.messages[3]?.content).toBe('')
    expect(sent).toHaveBeenCalledTimes(2)
    const requests = mocks.invoke.mock.calls.map(call => call[1].req)
    expect(requests.map(request => request.message)).toEqual([
      { user_message: 'first', requirements: '' },
      { user_message: 'second', requirements: '' },
    ])
    expect(requests[1]).not.toHaveProperty('history')
    expect(requests[1].conversation).toEqual([{ user_message: 'first', reply: result().reply }])
  })

  it('rejects missing bindings and blank messages before creating events or invoking IPC', async () => {
    const store = useMinimalRoleChatStore()
    await expect(store.sendMessage('hello')).rejects.toThrow('source')
    expect(() => store.bindSource({ ...source(), role_id: ' ' })).toThrow('source')
    store.bindSource(source())
    await expect(store.sendMessage(' \r\n')).rejects.toThrow('message')
    expect(mocks.invoke).not.toHaveBeenCalled()
    expect(store.messages).toEqual([])
    expect(submitted).not.toHaveBeenCalled()
    expect(sent).not.toHaveBeenCalled()
  })

  it('leaves the existing source and conversation intact when a new binding is invalid', async () => {
    const store = useMinimalRoleChatStore()
    store.bindSource(source())
    mocks.invoke.mockResolvedValueOnce(result())
    await store.sendMessage('completed')
    const oldSource = store.source
    expect(Object.isFrozen(oldSource)).toBe(true)
    expect(() => store.bindSource({ ...source('other'), asset_root: ' ' })).toThrow('source')
    expect(store.source).toBe(oldSource)
    expect(store.messages).toHaveLength(2)
    expect(mocks.invoke).toHaveBeenCalledTimes(1)
  })

  it('does not start transport when a real synchronous submit listener cancels the turn', async () => {
    const store = useMinimalRoleChatStore()
    store.bindSource(source())
    const cancel = () => store.cancelPendingSend()
    hostEventBus.on('message:submit', cancel)
    try {
      expect(await store.sendMessage('hello')).toBeUndefined()
      expect(store.messages).toEqual([])
      expect(store.isLoading).toBe(false)
      expect(mocks.invoke).not.toHaveBeenCalled()
      expect(sent).not.toHaveBeenCalled()
    }
    finally {
      hostEventBus.off('message:submit', cancel)
    }
  })

  it.each(['success', 'failure'])('discards a late %s after immediate client cancellation', async (outcome) => {
    const store = useMinimalRoleChatStore()
    store.bindSource(source())
    const pending = deferred()
    mocks.invoke.mockReturnValueOnce(pending.promise)
    const sending = store.sendMessage('hello')
    store.cancelPendingSend()
    expect(store.messages).toEqual([])
    expect(store.isLoading).toBe(false)
    if (outcome === 'success')
      pending.resolve(result())
    else
      pending.reject(JSON.stringify({ code: 'LLM_ERROR', message: 'late fixture error' }))
    expect(await sending).toBeUndefined()
    expect(store.messages).toEqual([])
    expect(sent).not.toHaveBeenCalled()
    expect(mocks.invoke).toHaveBeenCalledTimes(1)
  })

  it('does not let a superseded send clear the newer loading state or emit its result', async () => {
    const store = useMinimalRoleChatStore()
    store.bindSource(source())
    const old = deferred()
    const current = deferred()
    mocks.invoke.mockReturnValueOnce(old.promise).mockReturnValueOnce(current.promise)
    const first = store.sendMessage('old')
    const second = store.sendMessage('current')
    old.resolve(result('minimal-fixture', 'obsolete'))
    expect(await first).toBeUndefined()
    expect(store.isLoading).toBe(true)
    expect(store.messages.map(message => message.content)).toEqual(['current'])
    current.resolve(result('minimal-fixture', 'current reply'))
    await second
    expect(store.messages.map(message => message.content)).toEqual(['current', 'current reply'])
    expect(sent).toHaveBeenCalledTimes(1)
  })

  it('invalidates an old binding even when the technical ID is reused, and unbinds without I/O', async () => {
    const store = useMinimalRoleChatStore()
    store.bindSource(source())
    const pending = deferred()
    mocks.invoke.mockReturnValueOnce(pending.promise)
    const sending = store.sendMessage('old')
    store.bindSource({ ...source(), definition_reference: 'other.json' })
    pending.resolve(result())
    expect(await sending).toBeUndefined()
    expect(store.messages).toEqual([])
    expect(sent).not.toHaveBeenCalled()
    store.unbindSource()
    expect(store.source).toBeNull()
    await expect(store.sendMessage('no binding')).rejects.toThrow('source')
    expect(mocks.invoke).toHaveBeenCalledTimes(1)
  })

  it('preserves the Host error, removes only the failed user bubble and never recovers or retries', async () => {
    const store = useMinimalRoleChatStore()
    store.bindSource(source())
    mocks.invoke.mockResolvedValueOnce(result()).mockRejectedValueOnce(JSON.stringify({ code: 'LLM_ERROR', message: 'fixture failure' }))
    await store.sendMessage('completed')
    await expect(store.sendMessage('failed')).rejects.toMatchObject({ code: 'LLM_ERROR' })
    expect(store.messages).toHaveLength(2)
    expect(store.isLoading).toBe(false)
    expect(sent).toHaveBeenCalledTimes(1)
    expect(mocks.invoke.mock.calls.map(call => call[0])).toEqual(['send_minimal_message', 'send_minimal_message'])
  })

  it.each([
    { ...result(), role_id: 'different-id' },
    { ...result(), product_extensions: 'available' },
    { ...result(), reply: null },
  ])('refuses a malformed basic response without inventing a final result: %j', async (response) => {
    const store = useMinimalRoleChatStore()
    store.bindSource(source())
    mocks.invoke.mockResolvedValueOnce(response)
    await expect(store.sendMessage('hello')).rejects.toThrow('minimal response')
    expect(store.messages).toEqual([])
    expect(sent).not.toHaveBeenCalled()
    expect(store.isLoading).toBe(false)
  })

  it('snapshots only completed current turns before submit listeners can change state', async () => {
    const store = useMinimalRoleChatStore()
    store.bindSource(source())
    mocks.invoke.mockResolvedValue(result())
    await store.sendMessage('  coffee\r\n')
    const mutate = () => {
      store.messages[0]!.content = 'changed by synchronous listener'
    }
    hostEventBus.on('message:submit', mutate)
    try {
      await store.sendMessage('coffee again')
      expect(mocks.invoke.mock.calls[1]?.[1].req.conversation).toEqual([
        { user_message: '  coffee\r\n', reply: result().reply },
      ])
    }
    finally {
      hostEventBus.off('message:submit', mutate)
    }
    store.bindSource(source('other'))
    mocks.invoke.mockResolvedValueOnce(result('other'))
    await store.sendMessage('new role')
    expect(mocks.invoke.mock.calls[2]?.[1].req).not.toHaveProperty('conversation')
  })

  it('never contributes failed or cancelled turns to later memory candidates', async () => {
    const store = useMinimalRoleChatStore()
    store.bindSource(source())
    mocks.invoke.mockResolvedValueOnce(result('minimal-fixture', ''))
    await store.sendMessage('completed empty reply')
    mocks.invoke.mockRejectedValueOnce(JSON.stringify({ code: 'LLM_ERROR', message: 'fixture failure' }))
    await expect(store.sendMessage('failed')).rejects.toMatchObject({ code: 'LLM_ERROR' })
    const pending = deferred()
    mocks.invoke.mockReturnValueOnce(pending.promise)
    const cancelled = store.sendMessage('cancelled')
    store.cancelPendingSend()
    mocks.invoke.mockResolvedValueOnce(result())
    await store.sendMessage('current')
    expect(mocks.invoke.mock.calls[3]?.[1].req.conversation).toEqual([
      { user_message: 'completed empty reply', reply: '' },
    ])
    pending.resolve(result('minimal-fixture', 'obsolete'))
    expect(await cancelled).toBeUndefined()
  })

  it('sends at most the latest eight complete turns within the UTF-8 byte budget', async () => {
    const store = useMinimalRoleChatStore()
    store.bindSource(source())
    mocks.invoke.mockResolvedValue(result('minimal-fixture', ''))
    for (let index = 0; index < 10; index++)
      await store.sendMessage(`turn-${index}`)
    await store.sendMessage('next')
    expect(mocks.invoke.mock.calls.at(-1)?.[1].req.conversation.map((turn: { user_message: string }) => turn.user_message))
      .toEqual(Array.from({ length: 8 }, (_, index) => `turn-${index + 2}`))
    // One latest oversized pair means no contiguous suffix fits: do not cut its
    // words or skip it to smuggle in older candidates.
    await store.sendMessage('😀'.repeat(16 * 1024 + 1))
    await store.sendMessage('after large turn')
    expect(mocks.invoke.mock.calls.at(-1)?.[1].req).not.toHaveProperty('conversation')
  })
})
