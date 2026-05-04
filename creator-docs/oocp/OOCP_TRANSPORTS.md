# OOCP v0.1 — 传输层与入口约定

> **版本**：v0.1（草案）  
> **关联规范**：[OOCP_SPEC_v0_1.md](./OOCP_SPEC_v0_1.md)  
> **生效范围**：OOCP 服务端 + 所有客户端（VSCode / CLI / 编写器试聊）

---

## 1. WebSocket 端点

| 属性 | 值 |
|------|-----|
| **URL** | `ws://127.0.0.1:<port>/oocp` |
| **编码** | JSON（UTF-8），每帧一条完整消息 |
| **心跳** | 服务端每 15s 发送 Ping；客户端应回复 Pong（超时 5s 后服务端可能断开） |
| **默认端口** | `48888`（可通过环境变量 `OOCP_API_PORT` 覆盖） |

### 连接流程

```
客户端                               OOCP 服务端
  |                                      |
  |--- WS 握手（/oocp）                -->|
  |<-- capabilities 首帧（§2）         ---|
  |--- request（id=1, method="role.list"）-->|
  |<-- response（id=1, result=[...]）    ---|
  |                                      |
```

1. 客户端建立 WebSocket 连接到 `ws://127.0.0.1:<port>/oocp`。
2. 服务端立即发送 **capabilities** 首帧（无需客户端请求）。
3. 此后客户端可发送 `request` 帧，服务端回复 `response` 或 `error`。
4. 服务端可主动推送 `event` 帧。

---

## 2. Capabilities 首帧

连接后服务端自动发送，格式见 [OOCP_SPEC_v0_1.md §3.5](./OOCP_SPEC_v0_1.md#35-capabilities客户端-connect-后首个-response)。

示例：

```json
{
  "type": "capabilities",
  "version": "0.1.0",
  "methods": [
    "session.create", "session.destroy", "session.get_state",
    "session.switch_scene", "session.switch_interaction_mode",
    "session.export_chat_logs",
    "chat.send_message", "chat.generate_monologue",
    "role.list", "role.get_info", "role.set_remote_life",
    "time.get_state", "time.jump",
    "agent.call_mcp_tool"
  ],
  "events": ["chat.monologue", "session.time_tick", "agent.debug_trace", "trace.append"],
  "limits": {
    "max_concurrent_requests": 8,
    "max_message_chars": 4096
  },
  "auth_required": false
}
```

- `auth_required`：`false` 表示无需鉴权；`true` 时须携带 Bearer token。
- `version`：协议语义版本，客户端应检查兼容性。
- `limits.max_message_chars`：单条 `chat.send_message` 的 `user_message` 最大字符数；超出返回 `INVALID_PARAMS`。

---

## 3. 鉴权（方案 A：Bearer Token + 查询参数）

### 3.1 服务端配置

- 环境变量 `OOCP_API_TOKEN`：
  - **未设置或为空** → `auth_required = false`，放行所有连接。
  - **已设置** → `auth_required = true`，客户端必须携带匹配的 token。

### 3.2 客户端携带 token

两种方式（优先级从高到低）：

1. **HTTP Header**：`Authorization: Bearer <token>`
2. **查询参数**：`ws://127.0.0.1:48888/oocp?token=<token>`

> 若同时提供，优先使用 Header。

### 3.3 鉴权失败响应

- HTTP 状态：`401 Unauthorized`
- Header：`WWW-Authenticate: Bearer`
- Body：`"OOCP 鉴权失败：token 不匹配"`

---

## 4. 开发启动方式

### 4.1 npm 脚本（推荐）

```bash
npm run oocp:serve
```

等效于：`cargo run -p oclivenewnew-tauri -- --api 48888`

### 4.2 直接 cargo 命令

```bash
# 使用默认端口（优先读 OOCP_API_PORT，其次 48888）
cargo run -p oclivenewnew-tauri -- --api

# 指定端口
cargo run -p oclivenewnew-tauri -- --api --port 48888
```

### 4.3 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `OOCP_API_PORT` | WS/HTTP 监听端口 | `48888`（`--api` 模式） |
| `OOCP_API_TOKEN` | 共享鉴权令牌 | 无（允许所有连接） |
| `OCLIVE_ROLES_DIR` | 角色目录（优先于仓库默认值） | 仓库 `roles/` |

### 4.4 预期启动输出

```
INFO  oclive_api > HTTP API listening http://127.0.0.1:48888
```

成功后在另一个终端可连接 `ws://127.0.0.1:48888/oocp`。

---

## 5. HTTP API 端点（`--api` 模式包含）

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/health` | 健康检查，返回 `"ok"` |
| `GET` | `/oocp` | WebSocket 升级（OOCP 协议） |
| `POST` | `/chat` | 编写器试聊（JSON body：`{ role_path, message, session_id?, scene_id? }`） |

---

## 6. 关于端口策略

- **默认端口** `48888` 为固定值；编写器/客户端默认连接此端口。
- **允许多实例**：通过 `--port <N>` 或 `OOCP_API_PORT=<N>` 在同一机器上并行多个 core。
- **仅绑定 127.0.0.1**：不暴露到 LAN/WAN；如需远端访问，请配合反向代理（如 nginx）并自行评估安全风险。

---

## 7. 客户端对接清单

参考本文档实现 OOCP 客户端时，需要正确处理：

- [x] WebSocket 连接与重连
- [x] 读取 capabilities 首帧（检查 `version` 兼容性、读取 `methods`/`limits`）
- [x] 鉴权（通过 `auth_required` 判断 + Bearer token / `?token=` 查询参数）
- [x] 请求-响应匹配（通过 `id` 字段）
- [x] 事件监听（`type: "event"` 帧）
- [x] 最小调试事件（`trace.append`）：用于展示 MCP/Agent/插件调用链
- [x] 错误处理（`type: "error"` 帧，错误码见 spec §6）
- [x] 心跳响应（Ping → Pong）