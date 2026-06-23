import mitt from 'mitt'

const bus = mitt<Record<string, unknown>>()
/** `null`: not yet synced, do not filter (brief startup window); after sync, empty set means no subscriptions. */
let subscribed: Set<string> | null = null
let subscribedSignature = ''

export function setHostEventSubscribedEvents(events: string[]): void {
  const normalized = events.map(e => e.trim()).filter(Boolean)
  const nextSignature = normalized.join('\u001F')
  if (subscribed !== null && nextSignature === subscribedSignature) {
    return
  }
  subscribed = new Set(normalized)
  subscribedSignature = nextSignature
}

/** For tests or HMR: reset to unsynced state. */
export function clearHostEventSubscribedEvents(): void {
  subscribed = null
  subscribedSignature = ''
}

function shouldEmitBuiltin(type: string): boolean {
  if (subscribed === null) {
    return true
  }
  return subscribed.has(type)
}

/**
 * Shared event bus for host and plugin slots.
 * - `emitBuiltin`: host built-in events only; filtered by manifest `bridge.events` subscriptions.
 *   Common keys: `role:switched`, `role:info:updated`, `appearance:changed`, `message:sent`, `theme:changed`.
 * - `emit`: plugin custom events, no subscription filter; plugins should use `useOclive` `events.emit` (namespace validation).
 */
export const hostEventBus = {
  all: bus.all,
  on: bus.on.bind(bus),
  off: bus.off.bind(bus),
  emitBuiltin(type: string, event?: unknown) {
    if (!shouldEmitBuiltin(type)) {
      return
    }
    bus.emit(type, event)
  },
  emit(type: string, event?: unknown) {
    bus.emit(type, event)
  },
}
