# Agent Remote / Directory protocol (host ↔ sidecar)

[中文](../../creator-docs/plugin-and-architecture/AGENT_REMOTE_PROTOCOL.md)

**Implementation status**: The host implements **host-orchestrated** multi-turn JSON-RPC in `kernel/crates/oclive_kernel_host/src/infrastructure/remote_plugin/agent_http.rs` and `domain/agent_mcp_bridge.rs`: the sidecar returns `tool_calls[]`; **MCP tools execute on the host**, results flow back via `turn_context.tool_results` for the next `agent.process` round.

---

## 1. Environment variables

| Variable | Description |
|----------|-------------|
| `OCLIVE_REMOTE_AGENT_URL` | Agent sidecar JSON-RPC root URL (preferred) |
| `OCLIVE_REMOTE_AGENT_TIMEOUT_MS` | Timeout (default 120000 ms, clamped 1s–600s) |
| `OCLIVE_REMOTE_AGENT_TOKEN` | Optional Bearer token |
| (fallback) `OCLIVE_REMOTE_PLUGIN_URL` | Shared endpoint when Agent URL unset; method `agent.process` |
| (fallback) `OCLIVE_REMOTE_PLUGIN_TIMEOUT_MS` / `TOKEN` | Same as [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md) |

**Directory**: `plugin_backends.agent = directory` and `directory_plugins.agent` points at an installed directory plugin; RPC root URL is resolved by host lazy spawn (same as other directory slots).

**Permissions**: Outbound HTTP needs `remote:agent` in `high_risk_grants` (or directory loopback policy); MCP tool calls reuse `mcp:http` / `mcp:stdio` grants — **no** second Agent permission table.

**Degradation**: On remote/directory RPC failure (except grant denial), fall back to `BuiltinReActAgent`; log `target: oclive_agent`.

---

## 2. Method `agent.process`

### 2.1 params

| Field | Type | Description |
|-------|------|-------------|
| `protocol_version` | `u32` | Currently `1` |
| `role_id` | string | Role manifest id |
| `session_namespace` | string | Session namespace (srid) |
| `scene_id` | string | Scene id |
| `message` | string | User message this turn |
| `model` | string | Session LLM model name (sidecar may use) |
| `constraints` | object | Host-injected role constraints (tier B, below) |
| `tools` | array | MCP tool schemas (`name` = `server_id::tool_name`) |
| `turn_context` | object | Multi-turn context |

**`constraints` (`AgentRoleConstraints`)**

| Field | Type |
|-------|------|
| `personality_vector` | `number[7]` |
| `relation_state` | string |
| `favorability` | number |
| `scene_label` | string |
| `interaction_mode` | string? |
| `policy_text` | string? |

**`turn_context`**

| Field | Type |
|-------|------|
| `recent_turns` | `[user, assistant][]` recent turns |
| `tool_results` | MCP results from previous host execution |

**`tool_results[]` element**

| Field | Type |
|-------|------|
| `server_id` | string |
| `tool_name` | string |
| `params` | object |
| `result` | any |
| `error` | string? |

### 2.2 result

| Field | Type | Description |
|-------|------|-------------|
| `handled` | bool | `true` = Agent took this turn (host short-circuits co-present path) |
| `reply` | string? | Required when `handled=true` |
| `tool_calls` | array? | Host must run MCP; see below |
| `plan` | string? | Optional debug note |

**`tool_calls[]` element**

| Field | Type |
|-------|------|
| `tool_name` | string | qualified `server::tool` or bare name |
| `server_id` | string? | Optional hint |
| `params` | object |

### 2.3 Multi-turn loop (host)

1. Host builds `AgentInput` (constraints / tools / empty `tool_results`) and calls `agent.process`.
2. If `handled=true`, return `reply`.
3. If `tool_calls` non-empty, host runs MCP via `AgentMcpBridge`, fills `turn_context.tool_results`, calls `agent.process` again (same `message` / `constraints` / `tools`).
4. **Max 3** rounds; beyond that → `RemoteServiceUnavailable` and builtin degradation (remote/directory wrapper).

The sidecar **must not** HTTP-call the host to run tools; the tool loop is host-orchestrated.

---

## 3. Relation to builtin

| Backend | Behavior |
|---------|----------|
| `builtin` | `BuiltinReActAgent`: local LLM + function calling + MCP |
| `remote` | `AgentRpcProvider` + `FallbackAgentProvider` |
| `directory` | Same; URL from directory plugin |
| `none` | Noop; see [MODULE_NONE_SEMANTICS.md](../kernel/MODULE_NONE_SEMANTICS.md) |

---

## 4. Related

- [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md) — transport & shared env
- [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md) — directory slots
- [PLUGIN_V1.md](PLUGIN_V1.md) — `plugin_backends.agent`
