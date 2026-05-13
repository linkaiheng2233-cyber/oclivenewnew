// OOCP WebSocket Client SDK v0.1
// 统一处理连接、token、request/response id、capabilities、事件回调。

import WebSocket from "ws";
import type {
  OocpCapabilities,
  OocpConnectOptions,
  OocpError,
  OocpErrorCallback,
  OocpEvent,
  OocpEventCallback,
  OocpMessage,
  OocpRequest,
  OocpResponse,
  OocpCapabilitiesCallback,
} from "./types.js";

export type {
  OocpCapabilities,
  OocpConnectOptions,
  OocpError,
  OocpErrorCallback,
  OocpEvent,
  OocpEventCallback,
  OocpMessage,
  OocpRequest,
  OocpResponse,
  OocpCapabilitiesCallback,
};

/** 默认连接配置 */
const DEFAULT_OPTIONS: Required<OocpConnectOptions> = {
  url: "ws://127.0.0.1:48888/oocp",
  token: "",
  timeoutMs: 5000,
};

/** 可挂载 lifecycle 回调。可用于 VSCode 状态栏等场景。 */
export interface OocpClientLifecycle {
  onConnected?: (caps: OocpCapabilities) => void;
  onDisconnected?: (reason?: string) => void;
  onEvent?: OocpEventCallback;
  onError?: OocpErrorCallback;
}

export interface OocpClient {
  /** 当前 capabilities（连接后由服务端推送的首帧），未连接时为 null */
  readonly capabilities: OocpCapabilities | null;

  /** 连接状态 */
  readonly connected: boolean;

  /** 建立 WebSocket 连接并等待 capabilities 首帧 */
  connect(): Promise<OocpCapabilities>;

  /** 发送 OOCP request，返回 Promise<OocpResponse>。若连接未就绪会 reject。 */
  call(method: string, params?: Record<string, unknown>): Promise<OocpResponse>;

  /** 发送原始 request 对象（用于自定义 id 等场景） */
  callRaw(request: OocpRequest): Promise<OocpResponse>;

  /** 注册事件监听 */
  onEvent(cb: OocpEventCallback): void;

  /** 关闭连接 */
  close(): void;
}

export function connectOocp(
  options?: OocpConnectOptions,
  lifecycle?: OocpClientLifecycle,
): OocpClient {
  const opts = { ...DEFAULT_OPTIONS, ...options };
  let ws: WebSocket | null = null;
  let caps: OocpCapabilities | null = null;
  let nextId = 1;
  let eventCb: OocpEventCallback | null = null;

  // 待处理请求映射：id → { resolve, reject, timeout }
  const pending = new Map<
    number | string,
    {
      resolve: (r: OocpResponse) => void;
      reject: (e: Error) => void;
      timer: ReturnType<typeof setTimeout>;
    }
  >();

  let connectResolve: ((caps: OocpCapabilities) => void) | null = null;
  let connectReject: ((e: Error) => void) | null = null;
  let connectTimer: ReturnType<typeof setTimeout> | null = null;

  const client: OocpClient = {
    get capabilities() {
      return caps;
    },
    get connected() {
      return ws !== null && ws.readyState === WebSocket.OPEN;
    },

    connect(): Promise<OocpCapabilities> {
      return new Promise<OocpCapabilities>((resolve, reject) => {
        connectResolve = resolve;
        connectReject = reject;

        // 构建 URL（可在 url 中直接携带 ?token= 或按 Bearer header 携带）
        let url = opts.url;
        if (opts.token && !url.includes("?token=")) {
          url +=
            (url.includes("?") ? "&" : "?") +
            "token=" +
            encodeURIComponent(opts.token);
        }

        const socket = new WebSocket(url);
        ws = socket;

        connectTimer = setTimeout(() => {
          const msg = `OOCP 连接超时 (${opts.timeoutMs}ms) → ${url}`;
          connectReject?.(new Error(msg));
          cleanup();
        }, opts.timeoutMs);

        socket.on("open", () => {
          // 等待 capabilities 首帧
        });

        socket.on("message", (raw: WebSocket.Data) => {
          let msg: OocpMessage;
          try {
            msg = JSON.parse(raw.toString()) as OocpMessage;
          } catch {
            return; // 忽略无法解析的帧
          }

          // 首帧必须是 capabilities
          if (msg.type === "capabilities") {
            caps = msg as OocpCapabilities;
            if (connectTimer) {
              clearTimeout(connectTimer);
              connectTimer = null;
            }
            lifecycle?.onConnected?.(caps);
            connectResolve?.(caps);
            connectResolve = null;
            connectReject = null;
            return;
          }

          // 非 capabilities 帧按各自类型处理
          switch (msg.type) {
            case "response": {
              const resp = msg as OocpResponse;
              const entry = pending.get(resp.id);
              if (entry) {
                clearTimeout(entry.timer);
                pending.delete(resp.id);
                entry.resolve(resp);
              }
              break;
            }
            case "error": {
              const err = msg as OocpError;
              const id =
                err.id !== null && err.id !== undefined ? err.id : null;
              if (id !== null) {
                const entry = pending.get(id);
                if (entry) {
                  clearTimeout(entry.timer);
                  pending.delete(id);
                  entry.reject(
                    new Error(
                      `OOCP error [${err.error.code}]: ${err.error.message}`,
                    ),
                  );
                  return;
                }
              }
              lifecycle?.onError?.(
                new Error(
                  `OOCP error [${err.error.code}]: ${err.error.message}`,
                ),
              );
              break;
            }
            case "event": {
              const ev = msg as OocpEvent;
              eventCb?.(ev);
              lifecycle?.onEvent?.(ev);
              break;
            }
          }
        });

        socket.on("close", () => {
          const reason = "connection closed";
          lifecycle?.onDisconnected?.(reason);
          // reject 所有未完成的 pending 请求
          for (const [, entry] of pending) {
            clearTimeout(entry.timer);
            entry.reject(new Error(reason));
          }
          pending.clear();
          // 若还在等待连接，reject
          connectReject?.(new Error(reason));
          connectReject = null;
          cleanup();
        });

        socket.on("error", (e: Error) => {
          const reason = e.message || "connection error";
          lifecycle?.onError?.(new Error(reason));
          connectReject?.(new Error(reason));
          connectReject = null;
          cleanup();
        });
      });
    },

    call(
      method: string,
      params?: Record<string, unknown>,
    ): Promise<OocpResponse> {
      return this.callRaw({
        type: "request",
        id: nextId++,
        method,
        params: params ?? null,
      });
    },

    callRaw(request: OocpRequest): Promise<OocpResponse> {
      return new Promise<OocpResponse>((resolve, reject) => {
        if (!ws || ws.readyState !== WebSocket.OPEN) {
          reject(new Error("未连接到 OOCP 服务端，请先调用 connect()"));
          return;
        }

        const id = request.id;
        const timer = setTimeout(() => {
          pending.delete(id);
          reject(new Error(`OOCP request timeout (id=${String(id)})`));
        }, 30000);

        pending.set(id, { resolve, reject, timer });

        try {
          ws.send(JSON.stringify(request));
        } catch (e) {
          clearTimeout(timer);
          pending.delete(id);
          reject(
            new Error(
              `OOCP send failed: ${e instanceof Error ? e.message : String(e)}`,
            ),
          );
        }
      });
    },

    onEvent(cb: OocpEventCallback): void {
      eventCb = cb;
    },

    close(): void {
      cleanup();
      lifecycle?.onDisconnected?.("client closed");
    },
  };

  function cleanup() {
    if (connectTimer) {
      clearTimeout(connectTimer);
      connectTimer = null;
    }
    if (ws) {
      try {
        ws.close();
      } catch {
        // ignore
      }
      ws = null;
    }
  }

  return client;
}