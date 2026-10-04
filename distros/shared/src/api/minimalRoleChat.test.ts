// @vitest-environment jsdom
import type { MinimalRoleLocalMessageRequest } from './chat'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { sendMinimalMessage } from './chat'
import { ApiInvokeError } from './helpers'

const mocks = vi.hoisted(() => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/core', async importOriginal => ({
  ...await importOriginal<typeof import('@tauri-apps/api/core')>(),
  invoke: mocks.invoke,
}))

const request: MinimalRoleLocalMessageRequest = {
  source: {
    role_id: 'converter-owned-id',
    asset_root: 'E:/fixture-assets',
    definition_reference: 'chosen-content.json',
  },
  message: { user_message: '  current material\r\n', requirements: '' },
}

afterEach(() => vi.clearAllMocks())

describe('minimal text transport', () => {
  it('invokes the authenticated Rust path with the original source and returns only basic fields', async () => {
    const response = { role_id: request.source.role_id, reply: '  authoritative text\r\n', product_extensions: 'unavailable' }
    mocks.invoke.mockResolvedValueOnce(response)
    const fetch = vi.spyOn(globalThis, 'fetch')
    try {
      expect(await sendMinimalMessage(request)).toBe(response)
      expect(mocks.invoke).toHaveBeenCalledExactlyOnceWith('send_minimal_message', { req: request })
      expect(mocks.invoke.mock.calls[0]?.[1].req).toBe(request)
      expect(fetch).not.toHaveBeenCalled()
    }
    finally {
      fetch.mockRestore()
    }
  })

  it('preserves the kernel error and never switches to rich sending or recovery', async () => {
    const error = JSON.stringify({ code: 'KERNEL_AUTH_REQUIRED', message: 'fixture auth required' })
    mocks.invoke.mockRejectedValueOnce(error)
    const failure = await sendMinimalMessage(request).catch(value => value)
    expect(failure).toBeInstanceOf(ApiInvokeError)
    expect(failure.code).toBe('KERNEL_AUTH_REQUIRED')
    expect(failure.kernel.message).toBe('fixture auth required')
    expect(mocks.invoke).toHaveBeenCalledExactlyOnceWith('send_minimal_message', { req: request })
  })

  it('carries explicit temporary conversation through the same authenticated IPC request', async () => {
    const withConversation: MinimalRoleLocalMessageRequest = {
      ...request,
      conversation: [{ user_message: 'She does not like coffee.\r\n', reply: '' }],
    }
    mocks.invoke.mockResolvedValueOnce({ role_id: request.source.role_id, reply: 'text', product_extensions: 'unavailable' })
    await sendMinimalMessage(withConversation)
    expect(mocks.invoke).toHaveBeenCalledExactlyOnceWith('send_minimal_message', { req: withConversation })
    expect(mocks.invoke.mock.calls[0]?.[1].req).toBe(withConversation)
    expect(mocks.invoke.mock.calls[0]?.[1].req.conversation).toBe(withConversation.conversation)
  })

  it('does not discard commitments to obtain success from the default Prompt', async () => {
    const unsupported = { ...request, message: { ...request.message, requirements: 'retain topic' } }
    mocks.invoke.mockRejectedValueOnce(JSON.stringify({ code: 'INVALID_PARAMETER', message: 'requirements unsupported by this bound Prompt' }))
    await expect(sendMinimalMessage(unsupported)).rejects.toMatchObject({ code: 'INVALID_PARAMETER' })
    expect(mocks.invoke).toHaveBeenCalledExactlyOnceWith('send_minimal_message', { req: unsupported })
  })
})
