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

### S7 — 会话关闭（`session.destroy`）

| 项 | 值 |
|---|-----|
| **名称** | `session_destroy` |
| **OOCP 方法** | **`session.destroy`**（v0.1 无 `session.close` 别名；若文档写「关闭」均指本方法） |
| **请求 params** | `{ "session_ns": "<S4 返回的 session_ns>" }` |
| **预期 result** | `type: "response"` 且 `result` 为对象（当前运行时为 `{}` 占位）。 |
| **实现说明** | 当前 `runtime_oocp_handler::session_destroy` **不吊销**后续 `chat.send_message` 能力；本场景仅验证**协议面可调用且不崩溃**。若产品语义要求「销毁后不可再用」，需在运行时补强后再收紧断言。 |

---

### S8 — 多次消息往返

| 项 | 值 |
|---|-----|
| **名称** | `chat_send_message_multi_turn` |
| **OOCP 方法** | `chat.send_message` **连续 3 次**（与 S6 的首条消息合计共 4 轮用户输入；脚本对每轮均断言 `reply` 非空） |
| **请求 params** | 同 S6，仅 `user_message` 每次不同 |
| **预期 result** | 每轮均返回非空 `reply`。不强制要求文本互异（stub/兜底可能重复）。 |

---

### S9 — 插件槽 / 会话状态探针

| 项 | 值 |
|---|-----|
| **名称** | `session_state_probe` |
| **OOCP 方法（v0.1 实际）** | **`session.get_state`** |
| **背景** | **`plugin.list_slots` 不在 OOCP v0.1 方法白名单**（见 `crates/oclive_core/src/capabilities/mod.rs`）。宿主 UI 的插件槽由 Tauri / 市场数据面提供，**不在 `oclive_kernel_server` 本 job 覆盖范围**。 |
| **请求 params** | `{ "session_ns": "<S4>" }` |
| **预期 result** | `result.role_id` 为非空字符串；对象含会话快照字段（与 `session.get_state` DTO 一致）。 |

---

### S10 — 无效方法错误处理

| 项 | 值 |
|---|-----|
| **名称** | `unsupported_method` |
| **OOCP 方法** | 任意**不在** `capabilities.methods` 中的名称（脚本使用 `oclive.__nonexistent_method__`） |
| **请求 params** | `{}` |
| **预期** | 客户端收到 **`type: "error"`** 或 SDK 以 **reject/throw** 形式暴露；`error.code` 为 **`UNSUPPORTED_METHOD`**（或中文错误信息包含「未在 capabilities」「未知方法」等实现细节）。进程不崩溃。 |

---

### S11 — 角色包元数据（经 `role.get_info`）

| 项 | 值 |
|---|-----|
| **名称** | `role_pack_metadata` |
| **OOCP 方法** | **`role.get_info`**（v0.1 无独立 `role.get_pack_info`） |
| **请求 params** | 同 S3 |
| **预期 result** | `version`、`author`、`description` 均为**非空字符串**（与 `RoleInfo` / manifest 对齐；`description` 允许为短文案但不可缺字段）。 |

---

## 扩展新场景

1. 在 **`OOCP_METHODS`** 白名单与 `oocp_handler::handle_method` 中确认方法已实现。  
2. 在本表追加一行「名称 / 方法 / params / 预期」。  
3. 在 **`examples/oocp-test-suite/run.mjs`** 增加对应 `assert*`，并更新该目录 **`README.md`**。  
