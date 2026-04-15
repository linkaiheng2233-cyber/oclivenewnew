import mitt from "mitt";

const bus = mitt<Record<string, unknown>>();
/** `null`：尚未同步，不拦截（仅启动极短窗口）；同步后为空集表示无任何订阅。 */
let subscribed: Set<string> | null = null;

export function setHostEventSubscribedEvents(events: string[]): void {
  subscribed = new Set(events.map((e) => e.trim()).filter(Boolean));
}

/** 测试或热更新用：恢复为未同步状态。 */
export function clearHostEventSubscribedEvents(): void {
  subscribed = null;
}

/** 宿主与插件插槽共用的事件总线（内置事件名见文档）。未在 manifest `bridge.events` 中声明的内置事件不会广播。 */
export const hostEventBus = {
  all: bus.all,
  on: bus.on.bind(bus),
  off: bus.off.bind(bus),
  emit(type: string, event?: unknown) {
    if (subscribed !== null && !subscribed.has(type)) {
      return;
    }
    bus.emit(type, event);
  },
};
