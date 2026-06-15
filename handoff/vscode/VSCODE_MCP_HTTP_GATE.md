# VS Code · MCP HTTP Gate（波次 4）

**状态**：已实现（内核 HTTP 镜像）  
**消费者**：`oclive-vscode` Agent profile（`skip_agent=false`）  
**不进入** `process_message` 编排链。

---

## 范围

| 路由 | 方法 | 说明 |
|------|------|------|
| `/mcp/servers` | GET | 等同 Tauri `list_mcp_servers` |
| `/mcp/tools?server_id=` | GET | 等同 `list_mcp_tools` |
| `/mcp/call` | POST | body: `{ server_id, tool_name, params? }` |

与既有 `/high_risk/grants` · `/high_risk/grant` 配合：`mcp:http` / `mcp:stdio` grant 未授予时 call 失败（`HIGH_RISK_CAPABILITY_NOT_GRANTED`）。

---

## 实现落点

- 服务层：`crates/oclive_kernel_host/src/service/mcp.rs`
- 路由：`crates/oclive_kernel_host/src/http_api/mcp.rs` · `api_router`
- 扩展：`oclive-vscode/src/mcpBridge.ts`（QuickPick + Output `OCLive MCP`）

---

## 扩展 UX 边界

| Profile | MCP 入口 |
|---------|----------|
| 默认 `vscode.oclive.toml`（`skip_agent=true`） | **无** UI；命令执行时提示切换 agent profile |
| `vscode-agent.oclive.toml` | `OCLive: Call MCP Tool` · `List MCP Servers` |

渗透写盘 **0.4+ 走渗透插件**；不经 MCP。0.3.x 仍为核心内置（deprecated）。

---

## 验收

1. 默认 profile：命令 palette 无 MCP 成功路径（InformationMessage 引导）
2. Agent profile + `{app_data}/mcp-servers/*.json` + grant：QuickPick 可选 server/tool，Output 可见 JSON 结果
3. `cargo test -p oclive_kernel_host` HTTP 路由编译通过

---

## 相关

- [`VS4_AGENT.md`](../../oclive-vscode/docs/VS4_AGENT.md)
- 主仓 `src-tauri/src/api/agent.rs`（Tauri 同源实现）
