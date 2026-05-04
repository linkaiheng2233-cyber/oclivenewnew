# Agent 目录插件示例（Kernel V2 阶段 5-4）

将 **`AgentProvider::process`** 以 **Node JSON-RPC 侧车** 暴露为 **`agent.process`**，供 `plugin_backends.agent = directory` 使用。

## 与进程内 `default-agent-providers` 的关系

- 官方 **`full`** profile 通过 **`default-agent-providers`** 链接设施 crate **`oclive_agent_builtin`**（进程内 **Builtin ReAct**）。
- **`kernel-agent`**：MCP 客户端、`McpShellAgent`、调试面板等基础能力；关闭则无 MCP。
- 关闭 **`default-agent-providers`** 且开启 **`kernel-agent`** 时，宿主使用 **`McpShellAgent`**（`process` 恒为未接管）。
- 关闭两者中的内置 ReAct 后，可将 `agent` 设为 **`directory`** 并指向本插件 id（需 **`process:spawn`**）。侧车须自行实现 ReAct 或等价逻辑；本目录仅为协议演示。

## 协议

- 方法 **`agent.process`**，`params`：`role_id`、`session_namespace`、`message`、`model`。
- `result`：`handled`、`reply`（与 `AgentOutput` 一致）。
