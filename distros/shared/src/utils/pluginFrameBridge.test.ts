import type { PluginFrameIdentity } from './pluginFrameBridge'
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

function activateFrame(
  broker: ReturnType<typeof createPluginFrameBridge>,
  frame: Window,
  identity: PluginFrameIdentity = { pluginId: 'plugin.a', assetRel: 'a.html' },
) {
  const registration = broker.register(frame, identity)
  expect(registration.activate()).toBe(true)
  const calls = vi.mocked(frame.postMessage).mock.calls
  const bind = calls.at(-1)?.[0] as { value?: { token?: string } }
  const token = bind.value?.token
  expect(token).toMatch(/^[\w.:-]{32,128}$/)
  vi.mocked(frame.postMessage).mockClear()
  return { registration, token: token! }
}

function invokeRequest(token: string, requestId = 'request-1') {
  return {
    channel: PLUGIN_FRAME_BRIDGE_CHANNEL,
    kind: 'invoke',
    requestId,
    token,
    command: 'get_current_role',
    params: { detail: true },
  }
}

describe('pluginFrameBridge', () => {
  it('binds authority to the registered frame identity', async () => {
    const invoke = vi.fn().mockResolvedValue({ roleId: 'mumu' })
    const broker = createPluginFrameBridge(invoke)
    const frame = frameSource()
    const { token } = activateFrame(broker, frame, {
      pluginId: 'plugin.a',
      assetRel: 'slots/a.html',
    })

    await broker.handleMessage(message(frame, invokeRequest(token)))

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
    const { token } = activateFrame(broker, registered)

    await broker.handleMessage(message(attacker, invokeRequest(token)))
    await broker.handleMessage(message(registered, invokeRequest(token), 'https://evil.test'))

    expect(invoke).not.toHaveBeenCalled()
    expect(attacker.postMessage).not.toHaveBeenCalled()
    expect(registered.postMessage).not.toHaveBeenCalled()
  })

  it('rejects identity claims and malformed params', async () => {
    const invoke = vi.fn()
    const broker = createPluginFrameBridge(invoke)
    const frame = frameSource()
    const { token } = activateFrame(broker, frame)

    await broker.handleMessage(message(frame, {
      ...invokeRequest(token, 'forged-id'),
      pluginId: 'plugin.b',
    }))
    await broker.handleMessage(message(frame, {
      ...invokeRequest(token, 'bad-params'),
      params: ['not', 'an', 'object'],
    }))

    expect(invoke).not.toHaveBeenCalled()
  })

  it('rejects replayed request ids without invoking twice', async () => {
    const invoke = vi.fn().mockResolvedValue('ok')
    const broker = createPluginFrameBridge(invoke)
    const frame = frameSource()
    const { token } = activateFrame(broker, frame)
    const request = invokeRequest(token, 'same-request')

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

  it('revokes bridge authority when the registered frame navigates', async () => {
    const invoke = vi.fn()
    const broker = createPluginFrameBridge(invoke)
    const frame = frameSource()
    const { registration, token } = activateFrame(broker, frame)

    expect(registration.activate()).toBe(false)
    await broker.handleMessage(message(frame, invokeRequest(token, 'after-navigation')))

    expect(invoke).not.toHaveBeenCalled()
  })

  it('emits only events in the registered plugin namespace', async () => {
    const emit = vi.fn()
    const broker = createPluginFrameBridge(vi.fn(), { emit })
    const frame = frameSource()
    const { token } = activateFrame(broker, frame)

    await broker.handleMessage(message(frame, {
      channel: PLUGIN_FRAME_BRIDGE_CHANNEL,
      kind: 'emit',
      requestId: 'emit-ok',
      token,
      event: 'plugin.a:submit',
      data: { text: 'hello' },
    }))
    await broker.handleMessage(message(frame, {
      channel: PLUGIN_FRAME_BRIDGE_CHANNEL,
      kind: 'emit',
      requestId: 'emit-forged',
      token,
      event: 'plugin.b:submit',
    }))

    expect(emit).toHaveBeenCalledTimes(1)
    expect(emit).toHaveBeenCalledWith('plugin.a:submit', { text: 'hello' })
    expect(frame.postMessage).toHaveBeenLastCalledWith(
      expect.objectContaining({ ok: false, error: 'plugin event namespace denied' }),
      '*',
    )
  })

  it('cleans source-bound custom event subscriptions on unregister', async () => {
    let handler: ((data: unknown) => void) | undefined
    const unsubscribe = vi.fn()
    const subscribe = vi.fn((_event, next) => {
      handler = next
      return unsubscribe
    })
    const broker = createPluginFrameBridge(vi.fn(), { subscribe })
    const frame = frameSource()
    const { registration, token } = activateFrame(broker, frame, {
      pluginId: 'plugin.a',
      assetRel: 'a.html',
    })

    await broker.handleMessage(message(frame, {
      channel: PLUGIN_FRAME_BRIDGE_CHANNEL,
      kind: 'subscribe',
      requestId: 'subscription-1',
      token,
      event: 'plugin.a:hold',
    }))
    handler?.({ phase: 'start' })

    expect(subscribe).toHaveBeenCalledWith('plugin.a:hold', expect.any(Function))
    expect(frame.postMessage).toHaveBeenLastCalledWith(
      expect.objectContaining({
        kind: 'event',
        value: { event: 'plugin.a:hold', data: { phase: 'start' } },
      }),
      '*',
    )
    registration.unregister()
    expect(unsubscribe).toHaveBeenCalledOnce()
  })

  it('subscribes only to host events declared for the registered slot', async () => {
    const subscribe = vi.fn(() => vi.fn())
    const broker = createPluginFrameBridge(vi.fn(), { subscribe })
    const frame = frameSource()
    const { token } = activateFrame(broker, frame, {
      pluginId: 'plugin.a',
      assetRel: 'a.html',
      allowedEvents: ['role:switched'],
    })

    await broker.handleMessage(message(frame, {
      channel: PLUGIN_FRAME_BRIDGE_CHANNEL,
      kind: 'subscribe',
      requestId: 'allowed-host-event',
      token,
      event: 'role:switched',
    }))
    await broker.handleMessage(message(frame, {
      channel: PLUGIN_FRAME_BRIDGE_CHANNEL,
      kind: 'subscribe',
      requestId: 'denied-host-event',
      token,
      event: 'message:sent',
    }))

    expect(subscribe).toHaveBeenCalledOnce()
    expect(subscribe).toHaveBeenCalledWith('role:switched', expect.any(Function))
    expect(frame.postMessage).toHaveBeenLastCalledWith(
      expect.objectContaining({ ok: false, error: 'plugin event subscription denied' }),
      '*',
    )
  })

  it('allows host audio capture only for the registered trusted frame', async () => {
    const audioCapture = {
      start: vi.fn().mockResolvedValue({ mimeType: 'audio/webm' }),
      stop: vi.fn().mockResolvedValue({ audioBase64: 'AA==', mimeType: 'audio/webm' }),
      cancel: vi.fn().mockResolvedValue(null),
    }
    const broker = createPluginFrameBridge(vi.fn(), { audioCapture })
    const voiceFrame = frameSource()
    const otherFrame = frameSource()
    const voice = activateFrame(broker, voiceFrame, {
      pluginId: 'com.oclive.voice.asr',
      assetRel: 'slots/toolbar.html',
      allowAudioCapture: true,
    })
    const other = activateFrame(broker, otherFrame)

    await broker.handleMessage(message(voiceFrame, {
      channel: PLUGIN_FRAME_BRIDGE_CHANNEL,
      kind: 'audio-start',
      requestId: 'voice-start',
      token: voice.token,
    }))
    await broker.handleMessage(message(otherFrame, {
      channel: PLUGIN_FRAME_BRIDGE_CHANNEL,
      kind: 'audio-start',
      requestId: 'other-start',
      token: other.token,
    }))

    expect(audioCapture.start).toHaveBeenCalledOnce()
    expect(otherFrame.postMessage).toHaveBeenLastCalledWith(
      expect.objectContaining({ ok: false, error: 'plugin audio capture denied' }),
      '*',
    )
    voice.registration.unregister()
    expect(audioCapture.cancel).toHaveBeenCalledOnce()
  })

  it('serializes audio commands from the trusted frame', async () => {
    let resolveStart!: (value: unknown) => void
    const startResult = new Promise(resolve => (resolveStart = resolve))
    const audioCapture = {
      start: vi.fn(() => startResult),
      stop: vi.fn().mockResolvedValue({ audioBase64: 'AA==', mimeType: 'audio/webm' }),
      cancel: vi.fn().mockResolvedValue(null),
    }
    const broker = createPluginFrameBridge(vi.fn(), { audioCapture })
    const frame = frameSource()
    const { token } = activateFrame(broker, frame, {
      pluginId: 'com.oclive.voice.asr',
      assetRel: 'slots/toolbar.html',
      allowAudioCapture: true,
    })

    const starting = broker.handleMessage(message(frame, {
      channel: PLUGIN_FRAME_BRIDGE_CHANNEL,
      kind: 'audio-start',
      requestId: 'serial-start',
      token,
    }))
    const stopping = broker.handleMessage(message(frame, {
      channel: PLUGIN_FRAME_BRIDGE_CHANNEL,
      kind: 'audio-stop',
      requestId: 'serial-stop',
      token,
    }))
    await Promise.resolve()
    expect(audioCapture.stop).not.toHaveBeenCalled()

    resolveStart({ mimeType: 'audio/webm' })
    await Promise.all([starting, stopping])
    expect(audioCapture.stop).toHaveBeenCalledOnce()
  })
})
