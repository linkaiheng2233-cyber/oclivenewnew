import { getCurrentInstance, onUnmounted } from "vue";
import { readHostAppearance } from "../lib/hostAppearance";
import { pluginBridgeInvoke } from "../utils/tauri-api";
import { hostEventBus } from "../lib/hostEventBus";

/** 事件名须为 `命名空间:后缀`；命名空间为 `[a-zA-Z0-9.-]+`（与 manifest id 字符集一致）。 */
const PLUGIN_EVENT_NS = /^([a-zA-Z0-9.-]+):(.+)$/;

/** `pluginId:event` → 请求-响应处理器（与 mitt 分离，支持返回值）。 */
const requestHandlers = new Map<
  string,
  Set<(data: unknown) => unknown | Promise<unknown>>
>();

export interface OcliveEvents {
  emit(event: string, data?: unknown): void;
  on(event: string, handler: (data: unknown) => void): void;
  off(event: string, handler: (data: unknown) => void): void;
  /**
   * 向已用 `onRequest` 注册的监听方发起请求，返回首个 fulfilled 的结果（多监听方时为 `Promise.race`）。
   * 事件名须为 `某插件ID:名称`（可跨插件调用，不要求与调用方 ID 一致）。
   */
  request(event: string, data?: unknown, timeoutMs?: number): Promise<unknown>;
  onRequest(
    event: string,
    handler: (data: unknown) => unknown | Promise<unknown>,
  ): void;
  offRequest(
    event: string,
    handler: (data: unknown) => unknown | Promise<unknown>,
  ): void;
}

export interface OcliveApi {
  pluginId: string;
  bridgeAssetRel: string;
  invoke(command: string, params?: unknown): Promise<unknown>;
  /** 与顶栏外观一致：有效深浅色、`html` 根字号缩放系数（`--oclive-ui-scale`） */
  getAppearance(): ReturnType<typeof readHostAppearance>;
  events: OcliveEvents;
}

function validateEmitEvent(pluginId: string, raw: string): boolean {
  const t = raw.trim();
  if (!t) {
    console.warn("[oclive.events] emit rejected: empty event name");
    return false;
  }
  const m = t.match(PLUGIN_EVENT_NS);
  if (!m) {
    console.warn(
      `[oclive.events] emit rejected: event must match /^[a-zA-Z0-9.-]+:/ (${raw})`,
    );
    return false;
  }
  if (m[1] !== pluginId) {
    console.warn(
      `[oclive.events] emit rejected: namespace must match plugin id (${pluginId}): ${raw}`,
    );
    return false;
  }
  return true;
}

/** `oclive:role:switched` → `role:switched`；`oclive:appearance:changed` → `appearance:changed`；否则须为完整 `pluginId:…` 名。 */
function resolveListenEventName(raw: string): string | null {
  const t = raw.trim();
  if (!t) {
    console.warn("[oclive.events] on/off: empty event name");
    return null;
  }
  if (t.startsWith("oclive:")) {
    const rest = t.slice("oclive:".length);
    if (!rest) {
      console.warn(`[oclive.events] on/off: invalid builtin event: ${raw}`);
      return null;
    }
    return rest;
  }
  if (!PLUGIN_EVENT_NS.test(t)) {
    console.warn(
      `[oclive.events] on/off rejected: use \`pluginId:event\` or \`oclive:builtin:event\` (${raw})`,
    );
    return null;
  }
  return t;
}

function resolveRequestEventName(raw: string): string | null {
  const t = raw.trim();
  if (!t) {
    console.warn("[oclive.events.request] empty event name");
    return null;
  }
  if (!PLUGIN_EVENT_NS.test(t)) {
    console.warn(
      `[oclive.events.request] rejected: event must match /^[a-zA-Z0-9.-]+:/ (${raw})`,
    );
    return null;
  }
  return t;
}

function makeEvents(pluginId: string): OcliveEvents {
  const inst = getCurrentInstance();
  return {
    emit(event: string, data?: unknown) {
      if (!validateEmitEvent(pluginId, event)) return;
      hostEventBus.emit(event.trim(), data);
    },
    on(event: string, handler: (data: unknown) => void) {
      const resolved = resolveListenEventName(event);
      if (resolved === null) return;
      const fn = handler as (d: unknown) => void;
      hostEventBus.on(resolved, fn);
      if (inst) {
        onUnmounted(() => hostEventBus.off(resolved, fn), inst);
      }
    },
    off(event: string, handler: (data: unknown) => void) {
      const resolved = resolveListenEventName(event);
      if (resolved === null) return;
      hostEventBus.off(resolved, handler as (d: unknown) => void);
    },
    async request(event: string, data?: unknown, timeoutMs = 15000) {
      const resolved = resolveRequestEventName(event);
      if (resolved === null) {
        return Promise.reject(
          new Error("[oclive.events.request] invalid event name"),
        );
      }
      const set = requestHandlers.get(resolved);
      if (!set || set.size === 0) {
        return Promise.reject(
          new Error(`[oclive.events.request] no handler for ${resolved}`),
        );
      }
      const runners = [...set].map((h) =>
        Promise.resolve().then(() => h(data)),
      );
      return Promise.race([
        Promise.race(runners),
        new Promise((_, reject) => {
          setTimeout(
            () =>
              reject(
                new Error(
                  `[oclive.events.request] timeout after ${timeoutMs}ms (${resolved})`,
                ),
              ),
            timeoutMs,
          );
        }),
      ]);
    },
    onRequest(
      event: string,
      handler: (data: unknown) => unknown | Promise<unknown>,
    ) {
      const resolved = resolveRequestEventName(event);
      if (resolved === null) return;
      if (!requestHandlers.has(resolved)) {
        requestHandlers.set(resolved, new Set());
      }
      requestHandlers.get(resolved)!.add(handler);
      if (inst) {
        onUnmounted(() => {
          const s = requestHandlers.get(resolved);
          if (!s) return;
          s.delete(handler);
          if (s.size === 0) {
            requestHandlers.delete(resolved);
          }
        }, inst);
      }
    },
    offRequest(
      event: string,
      handler: (data: unknown) => unknown | Promise<unknown>,
    ) {
      const resolved = resolveRequestEventName(event);
      if (resolved === null) return;
      const s = requestHandlers.get(resolved);
      if (!s) return;
      s.delete(handler);
      if (s.size === 0) {
        requestHandlers.delete(resolved);
      }
    },
  };
}

/** 供 `provide('oclive', …)`；`bridgeAssetRel` 为 manifest 资源路径（插槽 `entry`、或整壳 `shell.vueEntry` 等），与 `plugin_bridge_invoke` 的 `assetRel` 一致。 */
export function createOcliveApi(
  pluginId: string,
  bridgeAssetRel: string,
): OcliveApi {
  return {
    pluginId,
    bridgeAssetRel,
    getAppearance() {
      return readHostAppearance();
    },
    async invoke(command: string, params?: unknown) {
      return pluginBridgeInvoke({
        pluginId,
        assetRel: bridgeAssetRel,
        command,
        params,
      });
    },
    events: makeEvents(pluginId),
  };
}
