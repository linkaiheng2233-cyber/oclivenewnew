# Agent Remote / Directory 协议（宿主 ↔ 侧车）

**实现状态**：宿主在 `crates/oclive_kernel_host/src/infrastructure/remote_plugin/agent_http.rs` 与 `domain/agent_mcp_bridge.rs` 中实现 **host-orchestrated** 多轮 JSON-RPC：侧车返回 `tool_calls[]`，**MCP 工具在本机执行**，结果经 `turn_context.tool_results` 回传下一轮 `agent.process`。

---

## 1. 环境变量

| 变量 | 说明 |
|------|------|
| `OCLIVE_REMOTE_AGENT_URL` | Agent 侧车 JSON-RPC 根 URL（优先） |
| `OCLIVE_REMOTE_AGENT_TIMEOUT_MS` | 超时（默认 120000 ms，钳制 1s–600s） |
| `OCLIVE_REMOTE_AGENT_TOKEN` | 可选 Bearer Token |
| （回退）`OCLIVE_REMOTE_PLUGIN_URL` | 未设 Agent URL 时使用共享端点，方法为 `agent.process` |
| （回退）`OCLIVE_REMOTE_PLUGIN_TIMEOUT_MS` / `TOKEN` | 与 [REMOTE_PLUGIN_PROTOCOL.md](./REMOTE_PLUGIN_PROTOCOL.md) 相同 |

**Directory**：`plugin_backends.agent = directory` 且 `directory_plugins.agent` 指向已安装目录插件；RPC 根 URL 由宿主 lazy spawn 解析（与其它 directory 槽一致）。

**权限**：出站 HTTP 需 `high_risk_grants` 中的 `remote:agent`（或目录 loopback 策略）；MCP 工具调用复用 `mcp:http` / `mcp:stdio` grant，**不**建第二套 Agent 权限表。

**降级**：remote/directory RPC 失败时（grant 拒绝除外）自动回退 `BuiltinReActAgent`，日志 `target: oclive_agent`。

---

## 2. 方法 `agent.process`

### 2.1 params

| 字段 | 类型 | 说明 |
|------|------|------|
| `protocol_version` | `u32` | 当前为 `1` |
| `role_id` | string | 角色 manifest id |
| `session_namespace` | string | 会话命名空间（srid） |
| `scene_id` | string | 场景 id |
| `message` | string | 用户本回合消息 |
| `model` | string | 本会话 LLM 模型名（侧车可选用） |
| `constraints` | object | 宿主注入的角色约束（B 档，见下） |
| `tools` | array | MCP 工具 schema（`name` 为 `server_id::tool_name`） |
| `turn_context` | object | 多轮上下文 |

**`constraints`（`AgentRoleConstraints`）**

| 字段 | 类型 |
|------|------|
| `personality_vector` | `number[7]` |
| `relation_state` | string |
| `favorability` | number |
| `scene_label` | string |
| `interaction_mode` | string? |
| `policy_text` | string? |

**`turn_context`**

| 字段 | 类型 |
|------|------|
| `recent_turns` | `[user, assistant][]` 最近若干轮 |
| `tool_results` | 上一轮宿主执行 MCP 后的结果数组 |

**`tool_results[]` 元素**

| 字段 | 类型 |
|------|------|
| `server_id` | string |
| `tool_name` | string |
| `params` | object |
| `result` | any |
| `error` | string? |

### 2.2 result

| 字段 | 类型 | 说明 |
|------|------|------|
| `handled` | bool | `true` 表示 Agent 已接管本回合（宿主短路共景路径） |
| `reply` | string? | `handled=true` 时必填 |
| `tool_calls` | array? | 需宿主执行 MCP 时返回；见下 |
| `plan` | string? | 可选调试说明 |

**`tool_calls[]` 元素**

| 字段 | 类型 |
|------|------|
| `tool_name` | string | qualified `server::tool` 或 bare name |
| `server_id` | string? | 可选提示 |
| `params` | object |

### 2.3 多轮循环（宿主）

1. 宿主组装 `AgentInput`（含 constraints / tools / 空 `tool_results`）并调用 `agent.process`。
2. 若 `handled=true`，返回 `reply`。
3. 若 `tool_calls` 非空，宿主经 `AgentMcpBridge` 执行 MCP，填充 `turn_context.tool_results`，再次 `agent.process`（同一 `message` / `constraints` / `tools`）。
4. 最多 **3** 轮；超出则 `RemoteServiceUnavailable` 并触发 builtin 降级（remote/directory 包装器）。

侧车 **不得** 反向 HTTP 调用宿主执行工具；工具循环由宿主 orchestrate。

---

## 3. 与 builtin 的关系

| 后端 | 行为 |
|------|------|
| `builtin` | `BuiltinReActAgent`：本机 LLM + function calling + MCP |
| `remote` | `AgentRpcProvider` + `FallbackAgentProvider` |
| `directory` | 同上，URL 来自 directory 插件 |
| `none` | Noop；见 [MODULE_NONE_SEMANTICS.md](../kernel/MODULE_NONE_SEMANTICS.md) |

---

## 4. 相关文档

- [REMOTE_PLUGIN_PROTOCOL.md](./REMOTE_PLUGIN_PROTOCOL.md) — 传输层与共享 env
- [DIRECTORY_PLUGINS.md](./DIRECTORY_PLUGINS.md) — directory 槽位
- [PLUGIN_V1.md](./PLUGIN_V1.md) — `plugin_backends.agent`
