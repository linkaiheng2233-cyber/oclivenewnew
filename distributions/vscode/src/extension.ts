// Oclive VSCode Extension — OOCP WebSocket 客户端 v0.1
//
// 功能：
//   1. Oclive: Connect — 连接 OOCP WS，在状态栏显示 capabilities.version
//   2. Oclive: Disconnect — 断开连接
//   3. Oclive: Show Chat — 打开 webview 聊天面板（发送消息 → 显示 reply）

import * as vscode from "vscode";
import { connectOocp, OocpClient, OocpCapabilities } from "@oclive/oocp-client";

// ── Webview 消息契约 ──

interface WebviewMessage {
  command: string;
  text?: string;
}

// ── 状态变量 ──

let statusBarItem: vscode.StatusBarItem;
let client: OocpClient | null = null;
let capabilities: OocpCapabilities | null = null;
let chatPanel: vscode.WebviewPanel | undefined;
let sessionNs: string | null = null;

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
  if (client && client.connected) {
    vscode.window.showInformationMessage("Oclive: already connected");
    updateStatusBar();
    return;
  }

  const config = vscode.workspace.getConfiguration("oclive.oocp");
  const url: string = config.get("url", "ws://127.0.0.1:48888/oocp");
  const token: string = config.get("token", "");

  statusBarItem.text = "$(sync~spin) Oclive OOCP connecting...";
  statusBarItem.tooltip = "Connecting to " + url;
  statusBarItem.show();

  try {
    // 先断开旧连接
    if (client) {
      client.close();
      client = null;
    }
    capabilities = null;
    sessionNs = null;

    client = connectOocp(
      { url, token, timeoutMs: 5000 },
      {
        onConnected: (caps) => {
          capabilities = caps;
          updateStatusBar();
          vscode.window.showInformationMessage(
            "Oclive: connected to OOCP " +
              caps.version +
              " (" +
              caps.methods.length +
              " methods)",
          );
        },
        onDisconnected: (_reason) => {
          capabilities = null;
          updateStatusBar();
        },
        onError: (err) => {
          vscode.window.showErrorMessage("Oclive OOCP error: " + err.message);
        },
      },
    );

    await client.connect();
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    statusBarItem.text = "$(error) Oclive OOCP (failed)";
    statusBarItem.tooltip = msg;
    statusBarItem.show();
    vscode.window.showErrorMessage("Oclive 连接失败: " + msg);
    client = null;
  }
}

function disconnect(): void {
  if (client) {
    client.close();
    client = null;
  }
  capabilities = null;
  sessionNs = null;
  updateStatusBar();
}

function updateStatusBar(): void {
  if (client && client.connected && capabilities) {
    statusBarItem.text = "$(pass) Oclive OOCP " + capabilities.version;
    statusBarItem.tooltip =
      "Connected (" +
      capabilities.methods.length +
      " methods) — click to disconnect";
    statusBarItem.command = "oclive.disconnect";
  } else {
    statusBarItem.text = "$(debug-disconnect) Oclive OOCP";
    statusBarItem.tooltip = "Not connected — click to connect";
    statusBarItem.command = "oclive.connect";
  }
  statusBarItem.show();
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

  chatPanel.onDidDispose(() => {
    chatPanel = undefined;
  });

  chatPanel.webview.onDidReceiveMessage(
    async (message: WebviewMessage) => {
      switch (message.command) {
        case "send": {
          const text = message.text as string;
          if (!text.trim()) return;
          try {
            if (!client || !client.connected) {
              postToChat({
                type: "error",
                content:
                  "未连接到 OOCP 服务端。\n请先执行 **Oclive: Connect** 或检查 URL 配置。\n当前配置: " +
                  vscode.workspace
                    .getConfiguration("oclive.oocp")
                    .get("url", "N/A"),
              });
              return;
            }

            // 如果没有 session，先查询角色列表再创建会话
            if (!sessionNs) {
              const roleListResp = await client.call("role.list", {});
              const roles = roleListResp.result;
              let extractedRoles: unknown[] = [];

              if (Array.isArray(roles)) {
                extractedRoles = roles;
              } else if (
                roles &&
                typeof roles === "object" &&
                "roles" in roles &&
                Array.isArray((roles as Record<string, unknown>).roles)
              ) {
                extractedRoles = (roles as Record<string, unknown>)
                  .roles as unknown[];
              }

              if (extractedRoles.length === 0) {
                postToChat({
                  type: "error",
                  content:
                    "没有可用角色 (role.list 返回空)。\n请先在 Oclive 中创建角色。",
                });
                return;
              }

              const firstRole = extractedRoles[0] as Record<string, unknown>;
              // 兼容常见字段名
              const roleId: string =
                (firstRole.role_id as string) ||
                (firstRole.manifestId as string) ||
                (firstRole.id as string) ||
                "";
              if (!roleId) {
                postToChat({
                  type: "error",
                  content:
                    "无法从 role.list 结果中提取 role_id。\n返回数据: " +
                    JSON.stringify(firstRole, null, 2),
                });
                return;
              }

              const sessionResp = await client.call("session.create", {
                role_id: roleId,
              });
              const sessResult = sessionResp.result;
              if (
                sessResult &&
                typeof sessResult === "object" &&
                "session_ns" in sessResult
              ) {
                sessionNs = String(
                  (sessResult as Record<string, unknown>).session_ns,
                );
              } else {
                // fallback: 尝试其他字段名
                const fallbackNs =
                  sessResult &&
                  typeof sessResult === "object"
                    ? ((sessResult as Record<string, unknown>).id as string) ||
                      ((sessResult as Record<string, unknown>)
                        .session_id as string) ||
                      JSON.stringify(sessResult)
                    : JSON.stringify(sessResult);
                sessionNs = String(fallbackNs);
              }

              postToChat({
                type: "system",
                content: "会话已创建 (role=" + roleId + ")",
              });
            }

            const resp = await client.call("chat.send_message", {
              session_ns: sessionNs,
              user_message: text,
            });
            const result = resp.result;
            if (
              result &&
              typeof result === "object" &&
              "reply" in result
            ) {
              postToChat({
                type: "reply",
                content: String(result.reply),
              });
            } else {
              postToChat({
                type: "reply",
                content: JSON.stringify(result, null, 2),
              });
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
            postToChat({
              type: "system",
              content:
                "已连接到 OOCP " +
                (capabilities?.version || "?"),
            });
          } catch (e: unknown) {
            postToChat({
              type: "error",
              content:
                "连接失败: " +
                (e instanceof Error ? e.message : String(e)),
            });
          }
          break;
        }
      }
    },
  );
}

function postToChat(message: {
  type: "reply" | "error" | "system";
  content: string;
}): void {
  chatPanel?.webview.postMessage(message);
}

function getChatHtml(): string {
  return `<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Oclive Chat</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: var(--vscode-font-family, -apple-system, sans-serif);
      font-size: var(--vscode-font-size, 13px);
      color: var(--vscode-foreground);
      background: var(--vscode-editor-background);
      display: flex; flex-direction: column; height: 100vh; padding: 0;
    }
    #output { flex: 1; overflow-y: auto; padding: 16px; }
    .msg { margin-bottom: 12px; padding: 8px 12px; border-radius: 6px; line-height: 1.5; white-space: pre-wrap; word-break: break-word; }
    .msg.user { background: var(--vscode-textBlockQuote-background); border-left: 3px solid var(--vscode-textLink-foreground); }
    .msg.reply { background: var(--vscode-editor-inactiveSelectionBackground); border-left: 3px solid var(--vscode-charts-green); }
    .msg.error { background: var(--vscode-inputValidation-errorBackground); border-left: 3px solid var(--vscode-inputValidation-errorBorder); color: var(--vscode-inputValidation-errorForeground); }
    .msg.system { background: var(--vscode-editorWidget-background); border-left: 3px solid var(--vscode-textSeparator-foreground); font-style: italic; opacity: 0.8; }
    #input-area { display: flex; padding: 8px 16px 16px; border-top: 1px solid var(--vscode-panel-border); gap: 8px; }
    #input { flex: 1; padding: 6px 10px; border: 1px solid var(--vscode-input-border); background: var(--vscode-input-background); color: var(--vscode-input-foreground); border-radius: 4px; font-family: inherit; font-size: inherit; resize: none; min-height: 32px; }
    #input:focus { outline: 1px solid var(--vscode-focusBorder); }
    button { padding: 6px 16px; border: none; background: var(--vscode-button-background); color: var(--vscode-button-foreground); border-radius: 4px; cursor: pointer; font-family: inherit; font-size: inherit; }
    button:hover { background: var(--vscode-button-hoverBackground); }
    button:disabled { opacity: 0.5; cursor: default; }
  </style>
</head>
<body>
  <div id="output"></div>
  <div id="input-area">
    <textarea id="input" rows="2" placeholder="输入消息..." autofocus></textarea>
    <button id="send-btn">Send</button>
  </div>
  <script>
    const vscode = acquireVsCodeApi();
    const output = document.getElementById('output');
    const input = document.getElementById('input');
    const sendBtn = document.getElementById('send-btn');

    function addMessage(type, content) {
      const div = document.createElement('div');
      div.className = 'msg ' + type;
      div.textContent = content;
      output.appendChild(div);
      output.scrollTop = output.scrollHeight;
    }

    function send() {
      const text = input.value.trim();
      if (!text) return;
      addMessage('user', text);
      vscode.postMessage({ command: 'send', text });
      input.value = '';
    }

    sendBtn.addEventListener('click', send);
    input.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); send(); }
    });

    window.addEventListener('message', (event) => {
      const msg = event.data;
      if (msg && msg.type && msg.content !== undefined) {
        addMessage(msg.type, msg.content);
      }
    });

    addMessage('system', 'Oclive Chat v0.1\\n发送消息前请确保已连接：Ctrl+Shift+P → Oclive: Connect');
  </script>
</body>
</html>`;
}