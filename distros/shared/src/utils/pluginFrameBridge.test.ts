import { describe, expect, it, vi } from 'vitest'
import {
  createPluginFrameBridge,
  PLUGIN_FRAME_BRIDGE_CHANNEL,
} from './pluginFrameBridge'

function frameSource() {
  return { postMessage: vi.fn() } as unknown as Window
}

function message(source: Window, data: unknown, origin = 'null'): MessageEvent {
  return { source, data, origin } as unknown as MessageEvent
}

function invokeRequest(requestId = 'request-1') {
  return {
    channel: PLUGIN_FRAME_BRIDGE_CHANNEL,
    kind: 'invoke',
    requestId,
    command: 'get_current_role',
    params: { detail: true },
  }
}

describe('pluginFrameBridge', () => {
  it('binds authority to the registered frame identity', async () => {
    const invoke = vi.fn().mockResolvedValue({ roleId: 'mumu' })
    const broker = createPluginFrameBridge(invoke)
    const frame = frameSource()
    broker.register(frame, { pluginId: 'plugin.a', assetRel: 'slots/a.html' })

    await broker.handleMessage(message(frame, invokeRequest()))

    expect(invoke).toHaveBeenCalledWith({
      pluginId: 'plugin.a',
      assetRel: 'slots/a.html',
      command: 'get_current_role',
      params: { detail: true },
    })
    expect(frame.postMessage).toHaveBeenCalledWith(
      expect.objectContaining({ ok: true, value: { roleId: 'mumu' } }),
      '*',
    )
  })

  it('rejects cross-frame and non-opaque-origin messages', async () => {
    const invoke = vi.fn()
    const broker = createPluginFrameBridge(invoke)
    const registered = frameSource()
    const attacker = frameSource()
    broker.register(registered, { pluginId: 'plugin.a', assetRel: 'a.html' })

    await broker.handleMessage(message(attacker, invokeRequest()))
    await broker.handleMessage(message(registered, invokeRequest(), 'https://evil.test'))

    expect(invoke).not.toHaveBeenCalled()
    expect(attacker.postMessage).not.toHaveBeenCalled()
    expect(registered.postMessage).not.toHaveBeenCalled()
  })

  it('rejects identity claims and malformed params', async () => {
    const invoke = vi.fn()
    const broker = createPluginFrameBridge(invoke)
    const frame = frameSource()
    broker.register(frame, { pluginId: 'plugin.a', assetRel: 'a.html' })

    await broker.handleMessage(message(frame, {
      ...invokeRequest('forged-id'),
      pluginId: 'plugin.b',
    }))
    await broker.handleMessage(message(frame, {
      ...invokeRequest('bad-params'),
      params: ['not', 'an', 'object'],
    }))

    expect(invoke).not.toHaveBeenCalled()
  })

  it('rejects replayed request ids without invoking twice', async () => {
    const invoke = vi.fn().mockResolvedValue('ok')
    const broker = createPluginFrameBridge(invoke)
    const frame = frameSource()
    broker.register(frame, { pluginId: 'plugin.a', assetRel: 'a.html' })
    const request = invokeRequest('same-request')

    await broker.handleMessage(message(frame, request))
    await broker.handleMessage(message(frame, request))

    expect(invoke).toHaveBeenCalledTimes(1)
    expect(frame.postMessage).toHaveBeenLastCalledWith(
      expect.objectContaining({
        ok: false,
        error: 'replayed plugin frame request',
      }),
      '*',
    )
  })
})
