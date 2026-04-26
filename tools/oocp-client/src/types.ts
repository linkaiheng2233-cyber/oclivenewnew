// OOCP v0.1 协议类型（客户端侧最小子集），与 oclive_core 定义保持一致。

/** capabilities 首帧 */
export interface OocpCapabilities {
  type: "capabilities";
  version: string;
  methods: string[];
  events: string[];
  limits: {
    max_concurrent_requests: number;
    max_message_chars: number;
  };
  auth_required: boolean;
}

/** 请求帧 */
export interface OocpRequest {
  type: "request";
  id: number | string;
  method: string;
  params?: Record<string, unknown> | null;
}

/** 成功响应帧 */
export interface OocpResponse {
  type: "response";
  id: number | string;
  result: Record<string, unknown> | null;
}

/** 错误帧 */
export interface OocpError {
  type: "error";
  id: number | string | null;
  error: {
    code: string;
    message: string;
    data?: unknown;
  };
}

/** 事件帧（服务端主动推送） */
export interface OocpEvent {
  type: "event";
  event: string;
  payload?: Record<string, unknown>;
}

/** 服务端可发出的任意帧 */
export type OocpMessage =
  | OocpCapabilities
  | OocpResponse
  | OocpError
  | OocpEvent;

/** 连接配置 */
export interface OocpConnectOptions {
  /** WebSocket URL，默认 ws://127.0.0.1:48888/oocp */
  url?: string;
  /** Bearer token；也可通过 `?token=` query 携带（当 url 已包含 token 时此处可省略） */
  token?: string;
  /** 连接超时（ms），默认 5000 */
  timeoutMs?: number;
}

/** 事件回调 */
export type OocpEventCallback = (event: OocpEvent) => void;

/** capabilities 就绪回调 */
export type OocpCapabilitiesCallback = (caps: OocpCapabilities) => void;

/** 错误回调 */
export type OocpErrorCallback = (err: Error) => void;