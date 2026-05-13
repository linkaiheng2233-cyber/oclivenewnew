# OOCP 协议测试套件（v0.1）

> **目的**：用**语言无关**的场景表描述「测什么」，再用官方 Node 脚本（[`examples/oocp-test-suite`](../../examples/oocp-test-suite/)）做可自动化执行的对照实现。  
> **交叉引用**：索引见 **[OOCP_SPEC_COMPLETE_REFERENCE.md](./OOCP_SPEC_COMPLETE_REFERENCE.md)**；消息与方法语义以 **[OOCP_SPEC_v0_1.md](./OOCP_SPEC_v0_1.md)** 为准。

## 前置条件

1. 启动 **`oclive_kernel_server`**（默认 `http://127.0.0.1:48888`，见 [`crates/oclive_kernel_server/README.md`](../../crates/oclive_kernel_server/README.md)）。
2. 设置 **`OCLIVE_ROLES_DIR`** 指向含至少一个有效角色包的目录（仓库自带 `roles/` 即可）。
3. 若启用 **`OOCP_API_TOKEN`**，客户端须在 WebSocket URL 带 `?token=` 或按实现要求附带鉴权（见 `OOCP_TRANSPORTS.md`）。

## 场景表

以下「预期」指：**成功路径**下 `type: "response"` 的 `result` 应满足的性质；若返回 `type: "error"`，则 `error.code` 须为文档所列可接受错误之一（一般不应出现在健康路径）。

---

### S0 — HTTP 健康检查

| 项 | 值 |
|---|-----|
| **名称** | `http_health_plain` |
| **传输** | HTTP（非 OOCP JSON） |
| **请求** | `GET /health`（无 `verbose`） |
| **预期** | HTTP **200**；响应体为纯文本，包含子串 **`ok`**（见 `http_api.rs` 注释：向后兼容明文探活）。 |

---

### S1 — WebSocket 能力与握手

| 项 | 值 |
|---|-----|
| **名称** | `oocp_capabilities_first_frame` |
| **传输** | WebSocket `GET /oocp`（或宿主等价 URL） |
| **步骤** | 连接成功后，服务端发出的**首帧 JSON** |
| **预期** | `type === "capabilities"`；`version` 与 **`OOCP_VERSION`**（当前 `0.1.0`）一致；`methods` 数组包含下文用到的方法名。 |

---

### S2 — `role.list`

| 项 | 值 |
|---|-----|
| **名称** | `role_list` |
| **OOCP 方法** | `role.list` |
| **请求 params** | `{}` |
| **预期 result** | **JSON 数组**（见 `runtime_oocp_handler::role_list`），长度 ≥ 1（在已配置 `OCLIVE_ROLES_DIR` 的前提下）；元素含 `role_id` / `id` / `manifestId` 之一且与角色包 id 一致。 |

---

### S3 — `role.get_info`

| 项 | 值 |
|---|-----|
| **名称** | `role_get_info` |
| **OOCP 方法** | `role.get_info` |
| **请求 params** | `{ "role_id": "<已知角色 id>", "session_id": null }`（`session_id` 可省略） |
| **预期 result** | 对象含 **`role_id`**、**`role_name`**；**`scenes`** 为非空字符串数组（可用场景 id）。权威字段集见 `RoleInfo`（`dto.rs`）。 |

---

### S4 — `session.create`

| 项 | 值 |
|---|-----|
| **名称** | `session_create` |
| **OOCP 方法** | `session.create` |
| **请求 params** | `{ "role_id": "<同上>", "session_id": null, "scene_id": null }`（后两者可省略） |
| **预期 result** | 含 **`session_ns`**（字符串）；`role` 子对象至少含 `name` 或 `role_id`（实现见 `runtime_oocp_handler::session_create`）。 |

---

### S5 — `session.switch_scene`

| 项 | 值 |
|---|-----|
| **名称** | `session_switch_scene` |
| **OOCP 方法** | `session.switch_scene`（注意：不是裸 `switch_scene`） |
| **请求 params** | `{ "session_ns": "<S4 返回>", "scene_id": "<须存在于角色包 scenes 列表中的 id>" }` |
| **预期 result** | `scene_id` 与请求一致；含 **`scene_name`**（展示名，可为空字符串但字段存在）。非法 `scene_id` 应返回 **`INVALID_PARAMS`**。 |

---

### S6 — `chat.send_message`

| 项 | 值 |
|---|-----|
| **名称** | `chat_send_message` |
| **OOCP 方法** | `chat.send_message` |
| **请求 params** | `{ "session_ns": "<S4>", "user_message": "Hello from OOCP test suite", "scene_id": "<与 S5 一致或当前场景>" }` |
| **预期 result** | 对象含非空字符串 **`reply`**（主对话 DTO；与规范「不用 `response` 字段」一致）。在无可用上游 LLM 时，内核可走 **fallback**（仍应返回 `reply`，可能带 `reply_is_fallback: true`）。 |

---

## 扩展新场景

1. 在 **`OOCP_METHODS`** 白名单与 `oocp_handler::handle_method` 中确认方法已实现。  
2. 在本表追加一行「名称 / 方法 / params / 预期」。  
3. 在 **`examples/oocp-test-suite/run.mjs`** 增加对应 `assert*`，并更新该目录 **`README.md`**。  
