import mitt from "mitt";

const bus = mitt<Record<string, unknown>>();
/** `null`：尚未同步，不拦截（仅启动极短窗口）；同步后为空集表示无任何订阅。 */
let subscribed: Set<string> | null = null;
let subscribedSignature = "";

export function setHostEventSubscribedEvents(events: string[]): void {
  const normalized = events.map((e) => e.trim()).filter(Boolean);
  const nextSignature = normalized.join("\u001f");
  if (subscribed !== null && nextSignature === subscribedSignature) {
    return;
  }
  subscribed = new Set(normalized);
  subscribedSignature = nextSignature;
}

/** 测试或热更新用：恢复为未同步状态。 */
export function clearHostEventSubscribedEvents(): void {
  subscribed = null;
  subscribedSignature = "";
}

function shouldEmitBuiltin(type: string): boolean {
  if (subscribed === null) {
    return true;
  }
  return subscribed.has(type);
}

/**
 * 宿主与插件插槽共用的事件总线。
 * - `emitBuiltin`：仅用于宿主内置事件，受 manifest `bridge.events` 订阅过滤。
 *   常见键：`role:switched`、`role:info:updated`、`appearance:changed`、`message:sent`、`theme:changed`。
 * - `emit`：用于插件自定义事件，不做订阅过滤；插件侧应通过 `useOclive` 的 `events.emit`（带命名空间校验）。
 */
export const hostEventBus = {
  all: bus.all,
  on: bus.on.bind(bus),
  off: bus.off.bind(bus),
  emitBuiltin(type: string, event?: unknown) {
    if (!shouldEmitBuiltin(type)) {
      return;
    }
    bus.emit(type, event);
  },
  emit(type: string, event?: unknown) {
    bus.emit(type, event);
  },
};
