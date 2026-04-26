// Oclive VSCode Extension — OOCP WebSocket 客户端 v0.1
//
// 功能：
//   1. Oclive: Connect — 连接 OOCP WS，在状态栏显示 capabilities.version
//   2. Oclive: Disconnect — 断开连接
//   3. Oclive: Show Chat — 打开 webview 聊天面板（发送消息 → 显示 reply）

import * as vscode from "vscode";
import WebSocket from "ws";

// ── 类型定义（对齐 OOCP spec v0.1） ──

interface OocpCapabilities {
  type: "capabilities";
  version: string;
  methods: string[];
  auth_required: boolean;
  limits: {
    max_message_chars: number;
  };
}

interface OocpRequest {
  type: "request";
  id: number;
  method: string;
  params: Record<string, unknown> | null;
}

interface OocpResponse {
  type: "response";
  id: number;
  result: Record<string, unknown> | null;
}

interface OocpError {
  type: "error";
  id: number | null;
  error: {
    code: string;
    message: string;
    data: unknown;
  };
}

interface OocpEvent {
  type: "event";
  event: string;
  payload: unknown;
}

type OocpMessage = OocpCapabilities | OocpResponse | OocpError | OocpEvent;

// ── Webview 消息契约 ──

interface WebviewMessage {
  command: string;
  text?: string;
}

// ── 状态变量 ──

let statusBarItem: vscode.StatusBarItem;
let ws: WebSocket | null = null;
let capabilities: OocpCapabilities | null = null;
let nextId = 1;
let chatPanel: vscode.WebviewPanel | undefined;

// pending 请求: id → resolve/reject
const pending = new Map<
  number,
  {
    resolve: (r: OocpResponse) => void;
    reject: (e: Error) => void;
    timer: ReturnType<typeof setTimeout>;
  }
>();

// ── 扩展激活 ──

export function activate(context: vscode.ExtensionContext) {
  statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    100,
  );
  statusBarItem.text = "$(debug-disconnect) Oclive OOCP";
  statusBarItem.tooltip = "Not connected — use Oclive: Connect";
  statusBarItem.command = "oclive.connect";
  statusBarItem.show();
  context.subscriptions.push(statusBarItem);

  context.subscriptions.push(
    vscode.commands.registerCommand("oclive.connect", () => connect()),
  );
  context.subscriptions.push(
    vscode.commands.registerCommand("oclive.disconnect", () => disconnect()),
  );
  context.subscriptions.push(
    vscode.commands.registerCommand("oclive.showChat", () =>
      showChatPanel(context),
    ),
  );
}

export function deactivate() {
  disconnect();
}

// ── 连接管理 ──

async function connect(): Promise<void> {
  if (ws && ws.readyState === WebSocket.OPEN) {
    vscode.window.showInformationMessage("Oclive: already connected");
    updateStatusBar();
    return;
  }

  const config = vscode.workspace.getConfiguration("oclive.oocp");
  let url: string = config.get("url", "ws://127.0.0.1:48888/oocp");
  const token: string = config.get("token", "");

  if (token && !url.includes("?token=")) {
    url += (url.includes("?") ? "&" : "?") + "token=" + encodeURIComponent(token);
  }

  statusBarItem.text = "$(sync~spin) Oclive OOCP connecting...";
  statusBarItem.tooltip = "Connecting to " + url;
  statusBarItem.show();

  try {
    await new Promise<void>((resolve, reject) => {
      const socket = new WebSocket(url);
      ws = socket;

      const timeout = setTimeout(() => {
        socket.close();
        reject(new Error("连接超时 (5s) → " + url));
      }, 5000);

      socket.on("open", () => {});

      socket.on("message", (raw: Buffer) => {
        let msg: OocpMessage;
        try {
          msg = JSON.parse(raw.toString()) as OocpMessage;
        } catch {
          return;
        }

        if (msg.type === "capabilities") {
          capabilities = msg as OocpCapabilities;
          clearTimeout(timeout);
          updateStatusBar();
          vscode.window.showInformationMessage(
            "Oclive: connected to OOCP " + capabilities.version + " (" + capabilities.methods.length + " methods)",
          );
          resolve();
          return;
        }

        handleMessage(msg);
      });

      socket.on("close", () => {
        clearTimeout(timeout);
        if (capabilities === null) {
          reject(new Error("connection closed before capabilities"));
        }
        handleDisconnect();
      });

      socket.on("error", (e: Error) => {
        clearTimeout(timeout);
        reject(e);
      });
    });
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    statusBarItem.text = "$(error) Oclive OOCP (failed)";
    statusBarItem.tooltip = msg;
    statusBarItem.show();
    vscode.window.showErrorMessage("Oclive 连接失败: " + msg);
  }
}

function disconnect(): void {
  if (ws) {
    try { ws.close(); } catch { }
    ws = null;
  }
  handleDisconnect();
}

function handleDisconnect(): void {
  for (const [, entry] of pending) {
    clearTimeout(entry.timer);
    entry.reject(new Error("disconnected"));
  }
  pending.clear();
  ws = null;
  capabilities = null;
  updateStatusBar();
}

function updateStatusBar(): void {
  if (ws && ws.readyState === WebSocket.OPEN && capabilities) {
    statusBarItem.text = "$(pass) Oclive OOCP " + capabilities.version;
    statusBarItem.tooltip = "Connected (" + capabilities.methods.length + " methods) — click to disconnect";
    statusBarItem.command = "oclive.disconnect";
  } else {
    statusBarItem.text = "$(debug-disconnect) Oclive OOCP";
    statusBarItem.tooltip = "Not connected — click to connect";
    statusBarItem.command = "oclive.connect";
  }
  statusBarItem.show();
}

// ── 消息处理 ──

function handleMessage(msg: OocpMessage): void {
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
      const id = err.id;
      if (id !== null && id !== undefined) {
        const entry = pending.get(id as number);
        if (entry) {
          clearTimeout(entry.timer);
          pending.delete(id as number);
          entry.reject(
            new Error("OOCP error [" + err.error.code + "]: " + err.error.message),
          );
          return;
        }
      }
      vscode.window.showWarningMessage(
        "OOCP error: [" + err.error.code + "] " + err.error.message,
      );
      break;
    }
    case "event": { break; }
  }
}

// ── 发送请求 ──

async function call(method: string, params: Record<string, unknown> = {}): Promise<OocpResponse> {
  return callRaw({ type: "request", id: nextId++, method, params });
}

function callRaw(request: OocpRequest): Promise<OocpResponse> {
  return new Promise<OocpResponse>((resolve, reject) => {
    if (!ws || ws.readyState !== WebSocket.OPEN) {
      reject(new Error("未连接到 OOCP 服务端，请先执行 Oclive: Connect"));
      return;
    }
    const id = request.id;
    const timer = setTimeout(() => {
      pending.delete(id);
      reject(new Error("OOCP request timeout (id=" + id + ")"));
    }, 30000);
    pending.set(id, { resolve, reject, timer });
    try {
      ws.send(JSON.stringify(request));
    } catch (e: unknown) {
      clearTimeout(timer);
      pending.delete(id);
      reject(new Error("OOCP send failed: " + (e instanceof Error ? e.message : String(e))));
    }
  });
}

// ── 聊天 webview 面板 ──

function showChatPanel(context: vscode.ExtensionContext): void {
  if (chatPanel) {
    chatPanel.reveal(vscode.ViewColumn.Two);
    return;
  }

  chatPanel = vscode.window.createWebviewPanel(
    "ocliveChat",
    "Oclive Chat",
    vscode.ViewColumn.Two,
    { enableScripts: true, retainContextWhenHidden: true },
  );

  chatPanel.webview.html = getChatHtml();

  chatPanel.onDidDispose(() => { chatPanel = undefined; });

  chatPanel.webview.onDidReceiveMessage(async (message: WebviewMessage) => {
    switch (message.command) {
      case "send": {
        const text = message.text as string;
        if (!text.trim()) return;
        try {
          if (!ws || ws.readyState !== WebSocket.OPEN) {
            postToChat({ type: "error", content: "未连接到 OOCP 服务端。\n请先执行 **Oclive: Connect** 或检查 URL 配置。\n当前配置: " + vscode.workspace.getConfiguration("oclive.oocp").get("url", "N/A") });
            return;
          }
          const resp = await call("chat.send_message", { session_ns: "vscode", user_message: text });
          const result = resp.result;
          if (result && typeof result === "object" && "reply" in result) {
            postToChat({ type: "reply", content: String(result.reply) });
          } else {
            postToChat({ type: "reply", content: JSON.stringify(result, null, 2) });
          }
        } catch (e: unknown) {
          const msg = e instanceof Error ? e.message : String(e);
          postToChat({ type: "error", content: "请求失败: " + msg });
        }
        break;
      }
      case "connect": {
        try {
          await connect();
          postToChat({ type: "system", content: "已连接到 OOCP " + (capabilities?.version || "?") });
        } catch (e: unknown) {
          postToChat({ type: "error", content: "连接失败: " + (e instanceof Error ? e.message : String(e)) });
        }
        break;
      }
    }
  });
}

function postToChat(message: { type: "reply" | "error" | "system"; content: string }): void {
  chatPanel?.webview.postMessage(message);
}

function getChatHtml(): string {
  return "<!DOCTYPE html>\n<html lang=\"zh-CN\">\n<head>\n  <meta charset=\"UTF-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n  <title>Oclive Chat</title>\n  <style>\n    * { box-sizing: border-box; margin: 0; padding: 0; }\n    body {\n      font-family: var(--vscode-font-family, -apple-system, sans-serif);\n      font-size: var(--vscode-font-size, 13px);\n      color: var(--vscode-foreground);\n      background: var(--vscode-editor-background);\n      display: flex; flex-direction: column; height: 100vh; padding: 0;\n    }\n    #output { flex: 1; overflow-y: auto; padding: 16px; }\n    .msg { margin-bottom: 12px; padding: 8px 12px; border-radius: 6px; line-height: 1.5; white-space: pre-wrap; word-break: break-word; }\n    .msg.user { background: var(--vscode-textBlockQuote-background); border-left: 3px solid var(--vscode-textLink-foreground); }\n    .msg.reply { background: var(--vscode-editor-inactiveSelectionBackground); border-left: 3px solid var(--vscode-charts-green); }\n    .msg.error { background: var(--vscode-inputValidation-errorBackground); border-left: 3px solid var(--vscode-inputValidation-errorBorder); color: var(--vscode-inputValidation-errorForeground); }\n    .msg.system { background: var(--vscode-editorWidget-background); border-left: 3px solid var(--vscode-textSeparator-foreground); font-style: italic; opacity: 0.8; }\n    #input-area { display: flex; padding: 8px 16px 16px; border-top: 1px solid var(--vscode-panel-border); gap: 8px; }\n    #input { flex: 1; padding: 6px 10px; border: 1px solid var(--vscode-input-border); background: var(--vscode-input-background); color: var(--vscode-input-foreground); border-radius: 4px; font-family: inherit; font-size: inherit; resize: none; min-height: 32px; }\n    #input:focus { outline: 1px solid var(--vscode-focusBorder); }\n    button { padding: 6px 16px; border: none; background: var(--vscode-button-background); color: var(--vscode-button-foreground); border-radius: 4px; cursor: pointer; font-family: inherit; font-size: inherit; }\n    button:hover { background: var(--vscode-button-hoverBackground); }\n    button:disabled { opacity: 0.5; cursor: default; }\n  </style>\n</head>\n<body>\n  <div id=\"output\"></div>\n  <div id=\"input-area\">\n    <textarea id=\"input\" rows=\"2\" placeholder=\"输入消息...\" autofocus></textarea>\n    <button id=\"send-btn\">Send</button>\n  </div>\n  <script>\n    const vscode = acquireVsCodeApi();\n    const output = document.getElementById('output');\n    const input = document.getElementById('input');\n    const sendBtn = document.getElementById('send-btn');\n\n    function addMessage(type, content) {\n      const div = document.createElement('div');\n      div.className = 'msg ' + type;\n      div.textContent = content;\n      output.appendChild(div);\n      output.scrollTop = output.scrollHeight;\n    }\n\n    function send() {\n      const text = input.value.trim();\n      if (!text) return;\n      addMessage('user', text);\n      vscode.postMessage({ command: 'send', text });\n      input.value = '';\n    }\n\n    sendBtn.addEventListener('click', send);\n    input.addEventListener('keydown', (e) => {\n      if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); send(); }\n    });\n\n    window.addEventListener('message', (event) => {\n      const msg = event.data;\n      if (msg && msg.type && msg.content !== undefined) {\n        addMessage(msg.type, msg.content);\n      }\n    });\n\n    addMessage('system', 'Oclive Chat v0.1\\\\n发送消息前请确保已连接：Ctrl+Shift+P → Oclive: Connect');\n  </script>\n</body>\n</html>";
}