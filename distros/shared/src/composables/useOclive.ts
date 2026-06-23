import { getCurrentInstance, onUnmounted } from 'vue'
import { readHostAppearance } from '@oclive/shared/lib/hostAppearance'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { pluginBridgeInvoke } from '@oclive/shared/api'

/** Event names must be `namespace:suffix`; namespace is `[a-zA-Z0-9.-]+` (same charset as manifest id). */
const PLUGIN_EVENT_NS = /^([a-z0-9.-]+):(.+)$/i

/** `pluginId:event` → request-response handlers (separate from mitt; supports return values). */
const requestHandlers = new Map<
  string,
  Set<(data: unknown) => unknown | Promise<unknown>>
>()

export interface OcliveEvents {
  emit: (event: string, data?: unknown) => void
  on: (event: string, handler: (data: unknown) => void) => void
  off: (event: string, handler: (data: unknown) => void) => void
  /**
   * Request listeners registered via `onRequest`; returns the first fulfilled result (`Promise.race` when multiple).
   * Event name must be `somePluginId:name` (cross-plugin OK; need not match caller id).
   */
  request: (event: string, data?: unknown, timeoutMs?: number) => Promise<unknown>
  onRequest: (
    event: string,
    handler: (data: unknown) => unknown | Promise<unknown>,
  ) => void
  offRequest: (
    event: string,
    handler: (data: unknown) => unknown | Promise<unknown>,
  ) => void
}

export interface OcliveApi {
  pluginId: string
  bridgeAssetRel: string
  invoke: (command: string, params?: unknown) => Promise<unknown>
  /** Matches top-bar appearance: effective light/dark and root font scale (`--oclive-ui-scale`). */
  getAppearance: () => ReturnType<typeof readHostAppearance>
  events: OcliveEvents
}

function validateEmitEvent(pluginId: string, raw: string): boolean {
  const t = raw.trim()
  if (!t) {
    console.warn('[oclive.events] emit rejected: empty event name')
    return false
  }
  const m = t.match(PLUGIN_EVENT_NS)
  if (!m) {
    console.warn(
      `[oclive.events] emit rejected: event must match /^[a-zA-Z0-9.-]+:/ (${raw})`,
    )
    return false
  }
  if (m[1] !== pluginId) {
    console.warn(
      `[oclive.events] emit rejected: namespace must match plugin id (${pluginId}): ${raw}`,
    )
    return false
  }
  return true
}

/** `oclive:role:switched` → `role:switched`; `oclive:appearance:changed` → `appearance:changed`; otherwise full `pluginId:…` name required. */
function resolveListenEventName(raw: string): string | null {
  const t = raw.trim()
  if (!t) {
    console.warn('[oclive.events] on/off: empty event name')
    return null
  }
  if (t.startsWith('oclive:')) {
    const rest = t.slice('oclive:'.length)
    if (!rest) {
      console.warn(`[oclive.events] on/off: invalid builtin event: ${raw}`)
      return null
    }
    return rest
  }
  if (!PLUGIN_EVENT_NS.test(t)) {
    console.warn(
      `[oclive.events] on/off rejected: use \`pluginId:event\` or \`oclive:builtin:event\` (${raw})`,
    )
    return null
  }
  return t
}

function resolveRequestEventName(raw: string): string | null {
  const t = raw.trim()
  if (!t) {
    console.warn('[oclive.events.request] empty event name')
    return null
  }
  if (!PLUGIN_EVENT_NS.test(t)) {
    console.warn(
      `[oclive.events.request] rejected: event must match /^[a-zA-Z0-9.-]+:/ (${raw})`,
    )
    return null
  }
  return t
}

function makeEvents(pluginId: string): OcliveEvents {
  const inst = getCurrentInstance()
  return {
    emit(event: string, data?: unknown) {
      if (!validateEmitEvent(pluginId, event))
        return
      hostEventBus.emit(event.trim(), data)
    },
    on(event: string, handler: (data: unknown) => void) {
      const resolved = resolveListenEventName(event)
      if (resolved === null)
        return
      const fn = handler as (d: unknown) => void
      hostEventBus.on(resolved, fn)
      if (inst) {
        onUnmounted(() => hostEventBus.off(resolved, fn), inst)
      }
    },
    off(event: string, handler: (data: unknown) => void) {
      const resolved = resolveListenEventName(event)
      if (resolved === null)
        return
      hostEventBus.off(resolved, handler as (d: unknown) => void)
    },
    async request(event: string, data?: unknown, timeoutMs = 15000) {
      const resolved = resolveRequestEventName(event)
      if (resolved === null) {
        return Promise.reject(
          new Error('[oclive.events.request] invalid event name'),
        )
      }
      const set = requestHandlers.get(resolved)
      if (!set || set.size === 0) {
        return Promise.reject(
          new Error(`[oclive.events.request] no handler for ${resolved}`),
        )
      }
      const runners = [...set].map(h =>
        Promise.resolve().then(() => h(data)),
      )
      let timeoutId: ReturnType<typeof setTimeout> | undefined
      try {
        return await Promise.race([
          Promise.race(runners),
          new Promise((_, reject) => {
            timeoutId = setTimeout(
              () =>
                reject(
                  new Error(
                    `[oclive.events.request] timeout after ${timeoutMs}ms (${resolved})`,
                  ),
                ),
              timeoutMs,
            )
          }),
        ])
      }
      finally {
        if (timeoutId !== undefined)
          clearTimeout(timeoutId)
      }
    },
    onRequest(
      event: string,
      handler: (data: unknown) => unknown | Promise<unknown>,
    ) {
      const resolved = resolveRequestEventName(event)
      if (resolved === null)
        return
      if (!requestHandlers.has(resolved)) {
        requestHandlers.set(resolved, new Set())
      }
      requestHandlers.get(resolved)!.add(handler)
      const unregister = () => {
        const s = requestHandlers.get(resolved)
        if (!s)
          return
        s.delete(handler)
        if (s.size === 0) {
          requestHandlers.delete(resolved)
        }
      }
      if (inst) {
        onUnmounted(unregister, inst)
      }
      else {
        // Non-component callers (e.g. plugin bootstrap) must unregister explicitly.
        const tagged = handler as typeof handler & { __ocliveUnregister?: () => void }
        tagged.__ocliveUnregister = unregister
      }
    },
    offRequest(
      event: string,
      handler: (data: unknown) => unknown | Promise<unknown>,
    ) {
      const resolved = resolveRequestEventName(event)
      if (resolved === null)
        return
      const s = requestHandlers.get(resolved)
      if (!s)
        return
      s.delete(handler)
      if (s.size === 0) {
        requestHandlers.delete(resolved)
      }
    },
  }
}

/** For `provide('oclive', …)`; `bridgeAssetRel` is manifest asset path (slot `entry`, full shell `shell.vueEntry`, etc.), same as `plugin_bridge_invoke` `assetRel`. */
export function createOcliveApi(
  pluginId: string,
  bridgeAssetRel: string,
): OcliveApi {
  return {
    pluginId,
    bridgeAssetRel,
    getAppearance() {
      return readHostAppearance()
    },
    async invoke(command: string, params?: unknown) {
      return pluginBridgeInvoke({
        pluginId,
        assetRel: bridgeAssetRel,
        command,
        params,
      })
    },
    events: makeEvents(pluginId),
  }
}
