# oclive_agent_builtin

**官方默认 Agent 模块**（工程名：设施 crate）。进程内 **Builtin ReAct Agent**：对 `LlmClient::generate` 做 function-calling 循环，经 `McpInvoke` 调度 MCP 工具。

- **`feature = "providers"`**：编译 `BuiltinReActAgent`。
- 须由宿主注入 **`Arc<dyn LlmClient>`** 与 **`Arc<dyn McpInvoke>`**（桌面侧通常为 `Arc<McpClient>`，`McpClient` 在 runtime 中实现 `McpInvoke`）。
- 与 **`kernel-agent`** 分离：后者控制 MCP 客户端是否链接；本 crate 仅提供 ReAct 编排逻辑。

目录插件示例（JSON-RPC `agent.process` 契约）：`examples/oclive-agent-builtin-directory/`。
