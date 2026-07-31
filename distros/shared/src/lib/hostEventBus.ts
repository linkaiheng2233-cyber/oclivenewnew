import mitt from 'mitt'

const bus = mitt<Record<string, unknown>>()

/**
 * Shared event bus for host and plugin slots.
 * - `emitBuiltin`: host built-in events. Internal host consumers always receive
 *   these; isolated plugin frames are filtered per registration by the parent broker.
 *   Common keys: `role:switched`, `role:info:updated`, `appearance:changed`, `message:sent`, `theme:changed`.
 * - `emit`: plugin custom events, no subscription filter; plugins should use `useOclive` `events.emit` (namespace validation).
 */
export const hostEventBus = {
  all: bus.all,
  on: bus.on.bind(bus),
  off: bus.off.bind(bus),
  emitBuiltin(type: string, event?: unknown) {
    bus.emit(type, event)
  },
  emit(type: string, event?: unknown) {
    bus.emit(type, event)
  },
}
