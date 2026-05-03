# OOCP 完整参考规范（与实现对齐）

> **定位**：本文是 **OClive Open Control Protocol（OOCP）** 的**实现级**单一编排页：消息形状、方法参数键、`result` 轮廓、WebSocket 行为、错误码与 **`capabilities`** 白名单均以当前代码为准。  
> **叙述性 v0.1 草案**（设计原则与示例）：[`OOCP_SPEC_v0_1.md`](./OOCP_SPEC_v0_1.md)。  
> **传输与端口**：[`OOCP_TRANSPORTS.md`](./OOCP_TRANSPORTS.md)。  
> **冻结与 MAJOR 策略**：[`OOCP_FREEZE_POLICY.md`](./OOCP_FREEZE_POLICY.md)。  
> **发版与 crates 顺序**：[`../distributions/KERNEL_AND_OOCP_RELEASE_PATH.md`](../distributions/KERNEL_AND_OOCP_RELEASE_PATH.md)。

---

## 0. 真相源（源码路径）

| 主题 | 路径 |
|------|------|
| 协议版本、方法/事件白名单 | `crates/oclive_core/src/capabilities/mod.rs`（`OOCP_VERSION`、`OOCP_METHODS`、`OOCP_EVENTS`） |
| 请求/响应/错误/capabilities 类型 | `crates/oclive_core/src/oocp/mod.rs` |
| 方法路由与 `params` 键解析 | `crates/oclive_core/src/oocp_handler.rs`（`handle_method`、`dispatch_oocp_request`） |
| 运行时 `result` 与副作用 | `crates/oclive_kernel_runtime/src/domain/adapters/runtime_oocp_handler.rs` |
| WebSocket 传输 | `crates/oclive_kernel_runtime/src/domain/adapters/oocp_ws.rs` |
| `chat.send_message` 的 DTO | `crates/oclive_kernel_runtime/src/models/dto.rs`（`SendMessageResponse`，字段 **`reply`**） |

---

## 1. 协议版本与能力协商

- **`capabilities.version`**：等于 Rust 常量 **`OOCP_VERSION`**（当前 **`0.1.0`**）。客户端应据此做兼容分支。  
- **`capabilities.methods` / `events`**：分别等于 **`OOCP_METHODS`** / **`OOCP_EVENTS`** 的向量拷贝。未列入的方法在 `dispatch_oocp_request` 层返回 **`UNSUPPORTED_METHOD`**。  
- **`capabilities.auth_required`**：当且仅当进程环境变量 **`OOCP_API_TOKEN`** 非空（trim 后）时为 **`true`**。  
- **`capabilities.limits`**：由适配层传入 `get_capabilities(auth_required, max_concurrent_requests, max_message_chars)`。  
  - **官方内核 WebSocket**（`runtime` 的 `oocp_handler::get_capabilities`）：**`max_concurrent_requests = 8`**，**`max_message_chars = 4096`**。  
  - **`oclive_core::capabilities::DefaultLimits`** 中的默认常量（**16** / **65536**）为**库级默认值**；**WS 握手以 runtime 传入值为准**，与 v0.1 叙述文档中的数字可能不一致时，**以首帧 `capabilities` 为准**。

---

## 2. 顶层消息（JSON 字段名）

Serde 使用 **`type`** 作为 JSON 键（Rust 字段名为 `msg_type`）。

### 2.1 Request（客户端 → 服务端）

| 键 | 类型 | 说明 |
|----|------|------|
| `type` | string | 应为 **`"request"`**；若省略或与方法组合不合法，`oocp_ws` 会尝试归一为 `request`（见 §6） |
| `id` | number 或 string | 任意 JSON；原样抄回 `response` / `error` |
| `method` | string | 点分方法名；**空字符串**表示「仅索要 capabilities」（见 §6） |
| `params` | object | 方法参数；缺键时多为 **`INVALID_PARAMS`** |

### 2.2 Response

| 键 | 类型 |
|----|------|
| `type` | **`"response"`** |
| `id` | 与 request 相同 |
| `result` | object / array / 任意 JSON，由方法决定 |

### 2.3 Error

| 键 | 类型 |
|----|------|
| `type` | **`"error"`** |
| `id` | 与 request 相同 |
| `error` | object：`code`（字符串）、`message`（字符串）、`data`（可选，默认不序列化 null） |

**`error.code`** 取值来自 **`OocpErrorCode::as_str()`**：

| 常量 | 字符串 |
|------|--------|
| `UnsupportedMethod` | `UNSUPPORTED_METHOD` |
| `InvalidParams` | `INVALID_PARAMS` |
| `SessionNotFound` | `SESSION_NOT_FOUND` |
| `RoleNotFound` | `ROLE_NOT_FOUND` |
| `LlmFailure` | `LLM_FAILURE` |
| `Internal` | `INTERNAL` |
| `AuthRequired` | `AUTH_REQUIRED` |
| `AuthFailed` | `AUTH_FAILED` |
| `RateLimited` | `RATE_LIMITED` |

> **与 Tauri/DTO 的措辞区别**：桌面 IPC 与 `SendMessageResponse` 使用字段名 **`reply`** 表示模型文本。OOCP 外层仍使用 **`type: "response"`** 与 **`result`** 包裹；**`chat.send_message` 的 `result` 对象内** 的对话正文键名为 **`reply`**（见 §5.7），**不要**在 `result` 顶层使用键名 `response` 指代正文。

### 2.4 Event（服务端 → 客户端）

| 键 | 类型 |
|----|------|
| `type` | **`"event"`** |
| `event` | string，须为 **`OOCP_EVENTS`** 之一或实现扩展（扩展前应更新白名单与本文） |
| `payload` | object |

当前白名单中的事件名：

- `chat.monologue`  
- `session.time_tick`  
- `agent.debug_trace`  
- `trace.append`（新客户端优先；与旧名并存说明见 `capabilities` 注释）

### 2.5 Capabilities（首帧或显式请求）

| 键 | 类型 |
|----|------|
| `type` | **`"capabilities"`** |
| `version` | string |
| `methods` | string 数组 |
| `events` | string 数组 |
| `limits` | `{ "max_concurrent_requests", "max_message_chars" }` |
| `auth_required` | boolean |

---

## 3. `session_ns` 约定

运行时规则（`runtime_oocp_handler`）：

- 有显式会话 id：`{role_id}__sess__{session_id}`  
- 无会话 id：`{role_id}__sess__default`  

凡参数名为 **`session_ns`** 的，须为上述格式之一；否则多数字段返回 **`INVALID_PARAMS`**（文案含「无效的 session_ns」）。

---

## 4. 方法白名单与分发表

下列 **`method`** 与 **`OOCP_METHODS`** 一致；参数键与 **`oocp_handler::handle_method`** 一致。

---

## 5. 各方法：`params` 与 `result`

### 5.1 `session.create`

**`params`**

| 键 | 必填 | 类型 | 说明 |
|----|------|------|------|
| `role_id` | 是 | string | manifest id |
| `session_id` | 否 | string | 多路会话；缺省走 `default` 段 |
| `scene_id` | 否 | string | **当前实现未使用**（保留与 Tauri 对齐）；可省略 |

**`result`**

- `session_ns`：string  
- `role`：`{ "role_id", "name", "interaction_mode" }`  

副作用：确保 `role_runtime`、默认情绪等（见实现）。

---

### 5.2 `session.destroy`

**`params`**：`session_ns`（必填）

**`result`**：空对象 `{}`（占位；不保证释放所有内存，以实现为准）。

---

### 5.3 `session.get_state`

**`params`**：`session_ns`（必填）

**`result`**（示例键，均为实现序列化结果）：

- `role_id`, `current_scene`, `current_favorability`, `relation_state`, `current_emotion`  
- `interaction_mode`, `remote_life_enabled`, `user_presence_scene`, `virtual_time_ms`

---

### 5.4 `session.switch_scene`

**`params`**：`session_ns`、`scene_id`（均必填）

**`result`**：`scene_id`, `scene_name`

**事件**：可能推送 **`chat.monologue`**，payload 含 `session_ns`、`monologue`、`scene_id`、`trigger: "scene_change"`。

---

### 5.5 `session.switch_interaction_mode`

**`params`**：`session_ns`、`mode`（均必填）

**`result`**：`{ "mode": "<trimmed>" }`

---

### 5.6 `session.export_chat_logs`

**`params`**

| 键 | 必填 | 说明 |
|----|------|------|
| `session_ns` | 是 | |
| `format` | 是 | **`json`** 或 **`txt`**（大小写不敏感，实现中 trim + ascii lowercase） |
| `path` | 否 | **当前 OOCP 实现忽略**（不写入客户端指定路径）；导出内容为内联返回 |

**`result`**

- `format`  
- `suggested_filename`  
- `content`：JSON 时为 pretty 字符串；TXT 时为纯文本  

JSON 内容结构见 `export_session_chat_logs_oocp_value`（含 `exported_at`、`role_id`、`turns[]` 等）。

---

### 5.7 `chat.send_message`

**`params`**

| 键 | 必填 |
|----|------|
| `session_ns` | 是 |
| `user_message` | 是 |
| `scene_id` | 否 |

**`result`**：`SendMessageResponse` 的 JSON（`crates/oclive_kernel_runtime/src/models/dto.rs`）。要点：

- **`reply`**：主对话文本（契约字段名）。  
- **`emotion`**：七维浮点（用户侧分析），**不是**枚举标签集合。  
- **`bot_emotion`** / **`portrait_emotion`**：字符串标签。  
- 另含 `api_version`, `schema`, `presence_mode`, `relation_state`, `favorability_*`, `events`, `scene_id`, `offer_destination_picker`, `offer_together_travel`, `reply_is_fallback`, `knowledge_chunks_in_prompt`, `timestamp` 等。

失败时常见 **`LLM_FAILURE`** 或 **`INTERNAL`**。

---

### 5.8 `chat.generate_monologue`

**`params`**：`session_ns`（必填），`context`（可选）

**`result`**：`{ "monologue": "<text>" }`

**事件**：推送 **`chat.monologue`**，payload 含 `session_ns`、`monologue`、`scene_id`（当前库场景）、`trigger`（默认 `"user_afk"` 或 `context`）。

---

### 5.9 `role.list`

**`params`**：无必填键（空 object 即可）

**`result`**：数组；每项至少含 `role_id` / `id` / `manifestId` / `name`（实现为兼容并列字段）。

---

### 5.10 `role.get_info`

**`params`**：`role_id`（必填），`session_id`（可选，trim 后空视为无）

**`result`**：`RoleInfo` 快照的 JSON（与 `role_info_snapshot::get_role_info_snapshot` 一致）。

---

### 5.11 `role.set_remote_life`

**`params`**：`session_ns`（必填），`enabled`（可选 boolean，**默认 `true`**）

**`result`**：`{ "enabled": <bool> }`

---

### 5.12 `time.get_state`

**`params`**：`session_ns`（必填）

**`result`**（实现 `get_time_state_oocp_value`）：

- `virtual_time_ms`, `virtual_time_label`, `iso_datetime`  
- `speed_multiplier`（当前 **1.0**）, `paused`（当前 **false**）  
- `character_current_scene`, `character_interaction_mode`

---

### 5.13 `time.jump`

**`params`**

| 键 | 必填 | 说明 |
|----|------|------|
| `session_ns` | 是 | |
| `target_time_ms` | 条件 | 与 `preset` **至少其一**；两者皆有时以 **`target_time_ms`** 为准（见实现注释） |
| `preset` | 条件 | 非空 string |

**`result`**：`virtual_time_ms`, `virtual_time_label`, `iso_datetime`, `monologues`, `favorability_delta`, `favorability_current`, `autonomous_scene_from`, `autonomous_scene_to`

---

### 5.14 `agent.call_mcp_tool`

**`params`**

| 键 | 必填 |
|----|------|
| `server_id` | 是 |
| `tool_name` | 是 |
| `arguments` | 否（缺省 **`null`**，由 MCP 层解释） |

**`result`**：`McpToolCallResult` 的 JSON：`server_id`, `tool_name`, `result`（工具返回的任意 JSON）。

**事件**：成功调用后推送 **`trace.append`**，`payload.kind = "mcp_tool_call"`，并含 `arguments` 与序列化后的 `result`。

**特性**：完整 MCP 能力依赖 **`kernel-agent`** 等特性；未启用构建时行为以编译结果为准（可能为空实现或错误）。

---

## 6. WebSocket 行为（`oclive_kernel_runtime`）

| 项 | 行为 |
|----|------|
| 路径 | **`GET /oocp`**，WebSocket 升级 |
| 首帧 | 连接成功后**立即**发送一条 **Text** 帧，内容为 **`capabilities` JSON** |
| 鉴权 | 环境变量 **`OOCP_API_TOKEN`** 非空时：`Authorization: Bearer <token>` **或** 查询参数 **`?token=`** 须匹配；否则 **HTTP 401**，正文为纯文本错误说明 |
| 空文本帧 | 若客户端发送 **trim 后为空** 的 Text，服务端回复 **当前 `capabilities` 的 JSON**（与首帧同源类型） |
| 消息长度 | 单帧 UTF-8 长度超过 **`capabilities.limits.max_message_chars`** 时返回 **`error`**，`INVALID_PARAMS` |
| 非法 JSON | **不发送任何帧**（解析失败返回 `None`）；客户端应使用超时与日志自检 |
| 请求归一 | `msg_type` 空则填 `"request"`；若 `msg_type` 非 `request` 但带 `method`，强制为 `request` |
| 服务端 Ping | 约 **每 15 秒** 发送 **`Ping`**；客户端应 **`Pong`**（与 v0.1 文档「建议 30s」不同处，**以本节为准**） |
| 事件顺序 | 单条 request 处理完后，**额外**发送 `handler.drain_events()` 中积压的 **多条 `event` Text 帧** |

---

## 7. 同进程 HTTP REST（非 OOCP）

与 OOCP **共用** `KernelAppState` 的 HTTP 路由（`http_api::api_router`）包括：

- `GET /health`  
- `POST /chat`  
- `POST|GET /role-feedback` 及子路径  

这些端点的 JSON **不**使用 OOCP 的 `type`/`method` 封装；与发行版 REST 契约以 **`http_api.rs`** 与角色反馈相关文档为准。编写「仅 OOCP」客户端时可忽略本节。

---

## 8. 维护清单（变更协议时）

1. 更新 **`crates/oclive_core/src/capabilities/mod.rs`** 中常量。  
2. 更新 **`oocp_handler::handle_method`** 路由与参数解析。  
3. 更新 **`runtime_oocp_handler`** 的 `result` 与事件。  
4. 同步 **本文**、[`OOCP_SPEC_v0_1.md`](./OOCP_SPEC_v0_1.md)、[`OOCP_TRANSPORTS.md`](./OOCP_TRANSPORTS.md) 中冲突数字。  
5. 根 **`CHANGELOG.md`** 记对外可见差异。  
6. MAJOR 或冻结节奏遵循 [`OOCP_FREEZE_POLICY.md`](./OOCP_FREEZE_POLICY.md)。
