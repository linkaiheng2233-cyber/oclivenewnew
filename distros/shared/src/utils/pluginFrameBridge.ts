export const PLUGIN_FRAME_BRIDGE_CHANNEL = 'oclive-plugin-frame-bridge-v1'

const REQUEST_ID_PATTERN = /^[\w.:-]{1,128}$/
const COMMAND_PATTERN = /^[a-z][a-z0-9_]{0,63}$/
const MAX_SEEN_REQUESTS = 4_096

export interface PluginFrameIdentity {
  pluginId: string
  assetRel: string
}

export interface PluginFrameInvokeRequest extends PluginFrameIdentity {
  command: string
  params?: unknown
}

export type PluginFrameInvoke = (
  request: PluginFrameInvokeRequest,
) => Promise<unknown>

export interface PluginFrameBridgeOptions {
  emit?: (event: string, data?: unknown) => void
  subscribe?: (event: string, handler: (data: unknown) => void) => () => void
}

interface FrameRegistration extends PluginFrameIdentity {
  token: string
  activated: boolean
  seenRequestIds: Set<string>
  subscriptions: Map<string, () => void>
}

export interface PluginFrameRegistration {
  activate: () => boolean
  unregister: () => void
}

interface FrameRequestMessage {
  channel: typeof PLUGIN_FRAME_BRIDGE_CHANNEL
  kind: 'invoke' | 'emit' | 'subscribe' | 'unsubscribe'
  requestId: string
  token: string
  command?: string
  params?: Record<string, unknown>
  event?: string
  data?: unknown
  subscriptionId?: string
}

interface FrameResponseMessage {
  channel: typeof PLUGIN_FRAME_BRIDGE_CHANNEL
  kind: 'bind' | 'result' | 'event'
  requestId: string
  ok: boolean
  value?: unknown
  error?: string
}

type PluginFrameWindow = Pick<Window, 'postMessage'>

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function parseRequest(value: unknown): FrameRequestMessage | null {
  if (!isRecord(value)
    || value.channel !== PLUGIN_FRAME_BRIDGE_CHANNEL
    || !['invoke', 'emit', 'subscribe', 'unsubscribe'].includes(String(value.kind))
    || typeof value.requestId !== 'string'
    || !REQUEST_ID_PATTERN.test(value.requestId)
    || typeof value.token !== 'string'
    || !REQUEST_ID_PATTERN.test(value.token)
    || ('pluginId' in value)
    || ('assetRel' in value)) {
    return null
  }
  if (value.kind === 'invoke'
    && (typeof value.command !== 'string'
      || !COMMAND_PATTERN.test(value.command)
      || (value.params !== undefined && !isRecord(value.params)))) {
    return null
  }
  if ((value.kind === 'emit' || value.kind === 'subscribe')
    && (typeof value.event !== 'string' || value.event.length > 192)) {
    return null
  }
  if (value.kind === 'unsubscribe'
    && (typeof value.subscriptionId !== 'string'
      || !REQUEST_ID_PATTERN.test(value.subscriptionId))) {
    return null
  }
  return value as unknown as FrameRequestMessage
}

function errorMessage(error: unknown): string {
  if (error instanceof Error)
    return error.message
  return String(error)
}

function createFrameToken(): string {
  const cryptoApi = globalThis.crypto
  if (!cryptoApi)
    throw new Error('secure plugin frame token unavailable')
  if (typeof cryptoApi.randomUUID === 'function')
    return cryptoApi.randomUUID()
  const bytes = new Uint8Array(32)
  cryptoApi.getRandomValues(bytes)
  return Array.from(bytes, value => value.toString(16).padStart(2, '0')).join('')
}

/**
 * Parent-side broker for opaque-origin plugin frames.
 *
 * Authority comes exclusively from the registered `contentWindow`; untrusted
 * messages cannot select a plugin id or asset path. The Rust bridge remains the
 * command allowlist authority.
 */
export function createPluginFrameBridge(
  invoke: PluginFrameInvoke,
  options: PluginFrameBridgeOptions = {},
) {
  const registrations = new Map<MessageEventSource, FrameRegistration>()

  function postResult(
    target: MessageEventSource,
    response: FrameResponseMessage,
  ): void {
    if ('postMessage' in target) {
      ;(target as PluginFrameWindow).postMessage(response, '*')
    }
  }

  function register(
    source: Window,
    identity: PluginFrameIdentity,
  ): PluginFrameRegistration {
    const registration: FrameRegistration = {
      pluginId: identity.pluginId,
      assetRel: identity.assetRel,
      token: createFrameToken(),
      activated: false,
      seenRequestIds: new Set(),
      subscriptions: new Map(),
    }
    registrations.set(source, registration)
    const unregister = () => {
      if (registrations.get(source) === registration) {
        for (const unsubscribe of registration.subscriptions.values())
          unsubscribe()
        registration.subscriptions.clear()
        registrations.delete(source)
      }
    }
    const activate = () => {
      if (registrations.get(source) !== registration)
        return false
      if (registration.activated) {
        unregister()
        return false
      }
      registration.activated = true
      postResult(source, {
        channel: PLUGIN_FRAME_BRIDGE_CHANNEL,
        kind: 'bind',
        requestId: registration.token,
        ok: true,
        value: { token: registration.token },
      })
      return true
    }
    return { activate, unregister }
  }

  async function handleMessage(event: MessageEvent): Promise<void> {
    const source = event.source
    if (!source || event.origin !== 'null')
      return
    const registration = registrations.get(source)
    if (!registration)
      return

    const request = parseRequest(event.data)
    if (!request)
      return
    if (!registration.activated || request.token !== registration.token)
      return

    if (registration.seenRequestIds.has(request.requestId)) {
      postResult(source, {
        channel: PLUGIN_FRAME_BRIDGE_CHANNEL,
        kind: 'result',
        requestId: request.requestId,
        ok: false,
        error: 'replayed plugin frame request',
      })
      return
    }
    if (registration.seenRequestIds.size >= MAX_SEEN_REQUESTS) {
      postResult(source, {
        channel: PLUGIN_FRAME_BRIDGE_CHANNEL,
        kind: 'result',
        requestId: request.requestId,
        ok: false,
        error: 'plugin frame request limit reached; reload the frame',
      })
      return
    }
    registration.seenRequestIds.add(request.requestId)

    try {
      let value: unknown
      if (request.kind === 'invoke') {
        value = await invoke({
          pluginId: registration.pluginId,
          assetRel: registration.assetRel,
          command: request.command!,
          params: request.params ?? {},
        })
      }
      else if (request.kind === 'emit') {
        const expectedPrefix = `${registration.pluginId}:`
        if (!request.event!.startsWith(expectedPrefix) || request.event!.length === expectedPrefix.length)
          throw new Error('plugin event namespace denied')
        if (!options.emit)
          throw new Error('plugin event bridge unavailable')
        options.emit(request.event!, request.data)
        value = null
      }
      else if (request.kind === 'subscribe') {
        const expectedPrefix = `${registration.pluginId}:`
        if (!request.event!.startsWith(expectedPrefix) || request.event!.length === expectedPrefix.length)
          throw new Error('plugin event subscription denied')
        if (!options.subscribe)
          throw new Error('plugin event bridge unavailable')
        const subscriptionId = request.requestId
        const unsubscribe = options.subscribe(request.event!, (data) => {
          postResult(source, {
            channel: PLUGIN_FRAME_BRIDGE_CHANNEL,
            kind: 'event',
            requestId: subscriptionId,
            ok: true,
            value: { event: request.event, data },
          })
        })
        registration.subscriptions.set(subscriptionId, unsubscribe)
        value = { subscriptionId }
      }
      else {
        const unsubscribe = registration.subscriptions.get(request.subscriptionId!)
        unsubscribe?.()
        registration.subscriptions.delete(request.subscriptionId!)
        value = null
      }
      postResult(source, {
        channel: PLUGIN_FRAME_BRIDGE_CHANNEL,
        kind: 'result',
        requestId: request.requestId,
        ok: true,
        value,
      })
    }
    catch (error) {
      postResult(source, {
        channel: PLUGIN_FRAME_BRIDGE_CHANNEL,
        kind: 'result',
        requestId: request.requestId,
        ok: false,
        error: errorMessage(error),
      })
    }
  }

  function dispose(): void {
    for (const registration of registrations.values()) {
      for (const unsubscribe of registration.subscriptions.values())
        unsubscribe()
    }
    registrations.clear()
  }

  return { register, handleMessage, dispose }
}
