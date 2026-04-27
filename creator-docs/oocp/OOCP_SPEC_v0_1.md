# OOCP v0.1 — OClive Open Control Protocol

> **版本**：v0.1（草案，可变更）  
> **生效范围**：oclivenewnew 内核 → 各发行版适配层（Tauri / VSCode / CLI / HTTP）  
> **语义版本**：协议自身版本由 `capabilities.version` 携带；方法/事件增删视为 MINOR；兼容破坏视为 MAJOR。  
> **硬约束**：回复字段为 **`reply`**，不是 `response`。

---

## 1. 设计原则

1. **传输无关**：方法定义与 WS / HTTP / stdin 等具体传输解耦；每种传输由一个 adapter 负责编解码与会话管理。
2. **请求-响应 + 事件流双通道**：
   - 请求-响应（request/response）：一次调用一次结果，带 `id` 匹配。
   - 事件流（event）：服务端向客户端推送，无需请求。
3. **capabilities 协商**：客户端 connect 后先获取 `capabilities`，确认协议版本与方法白名单。
4. **最小实现优先**：v0.1 仅覆盖核心对话 + 会话生命周期；其余方法逐步添加。

---

## 2. 传输层要求

| 属性 | 要求 |
|------|------|
| 编码 | JSON（UTF-8） |
| 消息边界 | 传输自行保证（如 WebSocket frame、HTTP body、\n-terminated line for stdio） |
| 错误 | 统一使用 `error` 结构体（见 §7） |
| 心跳 | 建议 30s ping/pong，由传输层 adapter 实现 |
| 并发 | 客户端可并发发送多个 request（`id` 区分）；服务端按序处理，不保证响应顺序 |
| 鉴权 | v0.1 最小实现使用共享 token（`Authorization: Bearer <token>` 或 WS 首帧 token）；后续版本实现 OAuth2 |

---

## 3. 消息类型

### 3.1 Request（客户端 → 服务端）

```json
{
  "type": "request",
  "id": "u64 或 string（唯一，由客户端生成）",
  "method": "chat.send_message",
  "params": { /* 方法特定参数 */ }
}
```

### 3.2 Response（服务端 → 客户端，对应一个 request）

```json
{
  "type": "response",
  "id": "对应 request 的 id",
  "result": { /* 方法特定返回值 */ }
}
```

### 3.3 Event（服务端 → 客户端，无关联 request）

```json
{
  "type": "event",
  "event": "chat.monologue",
  "payload": { /* 事件特定数据 */ }
}
```

#### 3.3.1 `trace.append`（最小调试事件）

用于发行版显示“内核正在做什么”（例如 MCP tool 调用、agent trace、插件调用链）。

```json
{
  "type": "event",
  "event": "trace.append",
  "payload": {
    "kind": "mcp_tool_call",
    "server_id": "weather",
    "tool_name": "get_weather",
    "arguments": { "city": "Hangzhou" },
    "result": { "content": "...", "is_error": false }
  }
}
```

### 3.4 Error（服务端 → 客户端，替代 response）

```json
{
  "type": "error",
  "id": "对应 request 的 id（若无法解析则置为 null）",
  "error": {
    "code": "UNSUPPORTED_METHOD",
    "message": "方法 'chat.send_message' 未在 capabilities 白名单中",
    "data": {}
  }
}
```

### 3.5 Capabilities（客户端 connect 后首个 response）

```json
{
  "type": "capabilities",
  "version": "0.1.0",
  "methods": ["session.create", "chat.send_message", "..."],
  "events": ["chat.monologue", "..."],
  "limits": {
    "max_concurrent_requests": 8,
    "max_message_chars": 4096
  },
  "auth_required": true
}
```

---

## 4. v0.1 方法清单

### 4.1 `session.create` — 创建会话

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `params.role_id` | string | ✅ | 角色 manifest id |
| `params.session_id` | string | ❌ | 可选会话命名空间（不填则用默认） |
| `params.scene_id` | string | ❌ | 初始场景，默认 `"default"` |

**Result**:
```json
{
  "session_ns": "role_a__sess__uuid",
  "role": { "name": "...", "scenes": ["default", "..."], "interaction_mode": "chat" }
}
```

### 4.2 `session.destroy` — 销毁会话

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `params.session_ns` | string | ✅ | 由 `session.create` 返回 |

**Result**: `{}`

---

### 4.3 `chat.send_message` — 发消息（核心对话）

**Params**:
```json
{
  "session_ns": "role_a__sess__uuid",
  "user_message": "用户输入文本",
  "scene_id": "default"
}
```

**Result** (对应 `SendMessageResponse` DTO):
```json
{
  "api_version": 1,
  "schema": 1,
  "presence_mode": "CoPresent | RemoteStub | RemoteLife",
  "relation_state": "Stranger",
  "reply": "角色回复文本",
  "emotion": {
    "joy": 0.0, "sadness": 0.0, "anger": 0.0,
    "fear": 0.0, "surprise": 0.0, "disgust": 0.0, "neutral": 1.0
  },
  "bot_emotion": "neutral",
  "portrait_emotion": "neutral",
  "favorability_delta": 0.0,
  "favorability_current": 0.0,
  "events": [],
  "scene_id": "default",
  "offer_destination_picker": false,
  "offer_together_travel": false,
  "reply_is_fallback": false,
  "knowledge_chunks_in_prompt": 0,
  "timestamp": 1717000000000
}
```

---

### 4.4 `chat.generate_monologue` — 生成独白

**Params**:
```json
{
  "session_ns": "role_a__sess__uuid",
  "context": "可选上下文"
}
```

**Result**: `{ "monologue": "角色独白文本" }`

---

### 4.5 `session.get_state` — 获取会话状态快照

**Params**: `{ "session_ns": "..." }`

**Result**:
```json
{
  "role_id": "...",
  "current_scene": "default",
  "current_favorability": 0.0,
  "relation_state": "Stranger",
  "current_emotion": "neutral",
  "interaction_mode": "chat",
  "remote_life_enabled": false,
  "user_presence_scene": "default",
  "virtual_time_ms": null
}
```

---

### 4.6 `session.switch_scene` — 切换角色场景

**Params**: `{ "session_ns": "...", "scene_id": "garden" }`

**Result**: `{ "scene_id": "garden", "scene_name": "花园" }`

---

### 4.7 `session.switch_interaction_mode` — 切换交互模式

**Params**: `{ "session_ns": "...", "mode": "chat | immersive" }`

**Result**: `{ "mode": "immersive" }`

---

### 4.8 `session.export_chat_logs` — 导出聊天日志

**Params**: `{ "session_ns": "...", "format": "txt | json", "path": "可选导出路径" }`

**Result**: `{ "path": "/path/to/export.txt", "size_bytes": 12345 }`

---

### 4.9 `role.list` — 列出所有已加载角色

**Params**: `{}`

**Result**: `[{ "id": "...", "name": "...", "avatar_path": "...", "interaction_mode": "chat" }]`

---

### 4.10 `role.get_info` — 获取角色详情

**Params**: `{ "role_id": "..." }`

**Result**: `{ "name": "...", "description": "...", "interaction_mode": "...", "scenes": [...], "default_personality": {...}, "life_schedule": {...} }`

---

### 4.11 `role.set_remote_life` — 开关异地心声

**Params**: `{ "session_ns": "...", "enabled": true }`

**Result**: `{ "enabled": true }`

---

### 4.12 `time.get_state` — 获取时间状态

**Params**: `{}`

**Result**:
```json
{
  "virtual_time_ms": 1717000000000,
  "virtual_time_label": "2024-05-29 20:00",
  "speed_multiplier": 1.0,
  "paused": false,
  "character_current_scene": "default",
  "character_interaction_mode": "chat"
}
```

---

### 4.13 `time.jump` — 时间跳跃

**Params**: `{ "session_ns": "...", "target_time_ms": 1717000000000 }`

**Result**: `{ "virtual_time_ms": 1717000000000, "virtual_time_label": "2024-05-29 20:00" }`

---

### 4.14 `agent.call_mcp_tool` — 调用 MCP 工具

**Params**: `{ "server_id": "...", "tool_name": "get_weather", "arguments": { "city": "北京" } }`

**Result**: `{ "content": "...", "is_error": false }`

---

## 5. v0.1 事件清单

### 5.1 `chat.monologue` — 角色主动独白

```json
{
  "type": "event",
  "event": "chat.monologue",
  "payload": {
    "session_ns": "...",
    "monologue": "角色自言自语...",
    "scene_id": "default",
    "trigger": "timeout"  // "timeout" | "scene_change" | "user_afk"
  }
}
```

### 5.2 `session.time_tick` — 虚拟时间推进

```json
{
  "type": "event",
  "event": "session.time_tick",
  "payload": {
    "session_ns": "...",
    "virtual_time_ms": 1717000000000,
    "virtual_time_label": "2024-05-29 20:05"
  }
}
```

### 5.3 `agent.debug_trace` — Agent 调试追踪

```json
{
  "type": "event",
  "event": "agent.debug_trace",
  "payload": {
    "session_ns": "...",
    "trace_id": "...",
    "step": "tool_call",
    "tool_name": "get_weather",
    "input": {"city": "北京"},
    "output": "...",
    "duration_ms": 234
  }
}
```

---

## 6. 错误码

| Code | 含义 | HTTP 等效 |
|------|------|-----------|
| `UNSUPPORTED_METHOD` | 方法不在 capabilities 白名单 | 501 |
| `INVALID_PARAMS` | params 格式/类型错误 | 400 |
| `SESSION_NOT_FOUND` | session_ns 不存在 | 404 |
| `ROLE_NOT_FOUND` | role_id 不存在 | 404 |
| `LLM_FAILURE` | LLM 调用失败 | 502 |
| `INTERNAL` | 内部错误 | 500 |
| `AUTH_REQUIRED` | 需要鉴权 | 401 |
| `AUTH_FAILED` | 鉴权失败 | 403 |
| `RATE_LIMITED` | 频率限制 | 429 |

---

## 7. 扩展规范（v0.x 期间）

新增方法/事件/错误码时：
1. 先更新本文档（版本号不变，仅追加）
2. 实现传输无关 handler
3. 更新 capabilities 声明
4. 至少一个 adapter 提供通路

禁止：
- ❌ 删除或重命名已有方法（仅 `deprecated` 标记）
- ❌ 修改已有 response/event 字段类型（仅追加新字段）
- ❌ `reply` 字段改名

---

## 8. 版本路线

| 版本 | 状态 | 说明 |
|------|------|------|
| **v0.1** | 草案 | 核心对话 + 会话生命周期 + 时间 + 独白 + 角色查询 + MCP 工具调用 |
| **v0.2** | 计划 | 全部 Tauri invoke 命令映射为 OOCP 方法；事件补齐；stream 大结果分片 |
| **v1.0** | 计划 | 冻结协议面，进入 Deprecation + 迁移周期 |