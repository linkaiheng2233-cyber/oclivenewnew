export const PLUGIN_FRAME_BRIDGE_CHANNEL = 'oclive-plugin-frame-bridge-v1'

const REQUEST_ID_PATTERN = /^[A-Za-z0-9._:-]{1,128}$/
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

interface FrameRegistration extends PluginFrameIdentity {
  seenRequestIds: Set<string>
}

interface FrameRequestMessage {
  channel: typeof PLUGIN_FRAME_BRIDGE_CHANNEL
  kind: 'invoke'
  requestId: string
  command: string
  params?: Record<string, unknown>
}

interface FrameResponseMessage {
  channel: typeof PLUGIN_FRAME_BRIDGE_CHANNEL
  kind: 'result'
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
    || value.kind !== 'invoke'
    || typeof value.requestId !== 'string'
    || !REQUEST_ID_PATTERN.test(value.requestId)
    || typeof value.command !== 'string'
    || !COMMAND_PATTERN.test(value.command)
    || ('pluginId' in value)
    || ('assetRel' in value)
    || (value.params !== undefined && !isRecord(value.params))) {
    return null
  }
  return value as unknown as FrameRequestMessage
}

function errorMessage(error: unknown): string {
  if (error instanceof Error)
    return error.message
  return String(error)
}

/**
 * Parent-side broker for opaque-origin plugin frames.
 *
 * Authority comes exclusively from the registered `contentWindow`; untrusted
 * messages cannot select a plugin id or asset path. The Rust bridge remains the
 * command allowlist authority.
 */
export function createPluginFrameBridge(invoke: PluginFrameInvoke) {
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
  ): () => void {
    const registration: FrameRegistration = {
      pluginId: identity.pluginId,
      assetRel: identity.assetRel,
      seenRequestIds: new Set(),
    }
    registrations.set(source, registration)
    return () => {
      if (registrations.get(source) === registration)
        registrations.delete(source)
    }
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
      const value = await invoke({
        pluginId: registration.pluginId,
        assetRel: registration.assetRel,
        command: request.command,
        params: request.params ?? {},
      })
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
    registrations.clear()
  }

  return { register, handleMessage, dispose }
}
