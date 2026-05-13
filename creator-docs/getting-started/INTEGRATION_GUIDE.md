# 如何接入 Oclive 内核

> 面向 **二次开发 / 侧车 / 硬件宿主**：在自有程序中复用与官方桌面同一套对话编排与插件体系。  
> 契约权威：**[`crates/oclive_kernel_runtime/src/models/dto.rs`](../../crates/oclive_kernel_runtime/src/models/dto.rs)**（Tauri 与 HTTP 共用）、**[`OOCP_SPEC_v0_1.md`](../oocp/OOCP_SPEC_v0_1.md)**、**[`KERNEL_BOUNDARY.md`](../kernel/KERNEL_BOUNDARY.md)**。

---

## 1. 概述

**Oclive 内核**（`oclive_kernel_runtime` 及依赖的 `oclive_kernel_core` / `oclive_kernel_models` / 设施 crate）提供 **角色包加载、插件路由、对话编排、持久化与 OOCP 能力面**；**不包含** Tauri 窗口与桌面快捷键。接入方只需选定 **内嵌**、**HTTP** 或 **OOCP WebSocket** 之一即可与同一语义对齐。

---

## 2. 环境要求

| 项 | 说明 |
|----|------|
| **Rust** | 与仓库 `rust-version` / CI 一致；建议 **stable**，并安装 **`rustfmt` / `clippy`**。 |
| **操作系统** | **Windows / macOS / Linux** 均可开发与运行；生产无头服务见 **[`docs/LINUX_KERNEL_DEPLOY.md`](../../docs/LINUX_KERNEL_DEPLOY.md)**。 |
| **可选** | **SQLite**（内嵌/服务默认）、**Ollama 或目录/远程 LLM 插件**（否则需自备 `LlmClient` 实现或 Mock）。 |

---

## 3. 三种接入方式

### 3.1 内嵌模式（`KernelAppState`）

在同一进程内持有状态并调用 `process_message`，无 HTTP 开销，适合 **设备侧、测试、自定义宿主**。

**依赖**：在自有 crate 的 `Cargo.toml` 中 `path` 或未来 crates.io 版本引用 `oclive_kernel_runtime`（feature 见 **[`LIGHTWEIGHT_PROFILE.md`](../kernel/LIGHTWEIGHT_PROFILE.md)**）。

**最小片段**（逻辑与 **[`examples/kernel_embed_minimal`](../../examples/kernel_embed_minimal)** 一致）：

```rust
use std::sync::Arc;
use oclive_kernel_runtime::domain::chat_engine::process_message;
use oclive_kernel_runtime::infrastructure::llm::MockLlmClient;
use oclive_kernel_runtime::models::dto::{SendMessageRequest, SendMessageResponse};
use oclive_kernel_runtime::state::KernelAppState;

// async 内：
let state = KernelAppState::new_in_memory_with_llm(
    Arc::new(MockLlmClient { reply: "你好。".into() }),
    roles_dir_path.into(),
)
.await?;

let req = SendMessageRequest {
    role_id: "your_role_id".into(),
    user_message: "你好".into(),
    scene_id: None,
    session_id: Some("sess-1".into()),
};
let res: SendMessageResponse = process_message(&state, &req).await?;
// res.reply 为角色回复（契约字段名固定为 reply）
```

持久化与生产路径：`KernelAppState::new(db_path, roles_dir, app_data_dir).await` — 详见 **[`KERNEL_SDK.md`](../kernel/KERNEL_SDK.md)**。

---

### 3.2 远程模式（HTTP REST）

由 **`oclive_kernel_server`** 或自建进程调用 **`oclive_kernel_runtime::http_api`**（需 **`kernel-http-api`** feature）暴露 REST，客户端任意语言可用 **HTTP** 调用。

**启动**（仓库根）：

```bash
cargo run -p oclive_kernel_server
```

**环境变量（常用）**：`OOCP_API_PORT`（默认 `48888`）、`OOCP_API_BIND`（默认 `127.0.0.1`）、`OOCP_API_TOKEN`（非空则要求 Bearer）、`OCLIVE_ROLES_DIR` / `OCLIVE_DB_PATH` / `OCLIVE_APP_DATA_DIR` — 见 **[`crates/oclive_kernel_server/README.md`](../../crates/oclive_kernel_server/README.md)**。

**最小片段**（`curl`）：

```bash
curl -sS "http://127.0.0.1:48888/health"
curl -sS -X POST "http://127.0.0.1:48888/chat" \
  -H "Content-Type: application/json; charset=utf-8" \
  -d '{"role_path":"/path/to/role/dir","message":"你好","session_id":null,"scene_id":null}'
```

Python 封装见 **[`sdk/python/README.md`](../../sdk/python/README.md)** 与 **`examples/kernel_remote_simple/`**。

---

### 3.3 OOCP 协议（WebSocket）

内核在同一端口提供 **`GET /oocp`** WebSocket：连接后按 **[`OOCP_SPEC_v0_1.md`](../oocp/OOCP_SPEC_v0_1.md)** 交换 **capabilities**，再发 **`request` / `response`**（如 `chat.send_message`）。  
若设置了 **`OOCP_API_TOKEN`**，须在首帧或握手约定中携带 **`Authorization: Bearer`**（实现细节见运行时 `http_api` 与 **`OOCP_TRANSPORTS.md`**）。

**概念最小片段**（伪代码）：

```text
1. 连接 ws://127.0.0.1:48888/oocp
2. 读取服务端下发的 `type: "capabilities"` 帧（或按实现约定完成鉴权）
3. 发送 {"type":"request","id":"1","method":"chat.send_message","params":{...}}
4. 读取 type=response，result.reply 即为回复
```

完整字段与错误码以 **OOCP 规范** 为准；桌面 Tauri 侧通过适配层走同一套方法名。

---

## 4. 常用 HTTP API 参考

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/health` | 纯文本探活，默认 **`ok`**（不设 token 时通常仍可用）。 |
| GET | `/health?verbose=true` | JSON 多维度健康检查。 |
| GET | `/health/db` | 仅验证 SQLite（`SELECT 1`），适合监控。 |
| POST | `/chat` | 试聊 JSON，body 含 **`role_path` / `message`** 等；响应含 **`reply`**。 |
| WS | `/oocp` | OOCP WebSocket（capabilities + 方法调用）。 |

更多路由与实现注释见 **`crates/oclive_kernel_runtime/src/http_api.rs`**。

---

## 5. 常见问题与故障排除

| 现象 | 排查 |
|------|------|
| **`401` / 鉴权失败** | 是否设置了 **`OOCP_API_TOKEN`** 却未带 **`Authorization: Bearer`**；**`/health`** 探活一般仍可用，**`/chat` 与 WS** 在 token 启用时需一致携带。 |
| **连接被拒绝** | **`OOCP_API_BIND`** 是否仅为 `127.0.0.1` 却从其它机器访问；容器内需 `0.0.0.0` 并配合防火墙与 token。 |
| **角色路径错误** | **`role_path`** 须指向**角色包根目录**（含 `manifest.json`）；或内嵌模式使用 **`roles_dir` + `role_id`**。生产建议 **`OCLIVE_REQUIRE_EXPLICIT_PATHS=1`**。 |
| **插件 / MCP 无权限** | 目录插件首次 **`process:spawn`** 等需用户授权；见 **`DIRECTORY_PLUGINS.md`** 与权限枚举。 |
| **裁剪 feature 后某能力缺失** | 使用 **`--no-default-features`** 时按需打开子 feature；对照 **`LIGHTWEIGHT_PROFILE.md`**。 |

---

## 6. 延伸阅读

| 文档 | 用途 |
|------|------|
| **[`KERNEL_SDK.md`](../kernel/KERNEL_SDK.md)** | 库模式 / 服务 / 示例索引 |
| **[`KERNEL_BOUNDARY.md`](../kernel/KERNEL_BOUNDARY.md)** | 内核 vs 发行版职责 |
| **[`LIGHTWEIGHT_PROFILE.md`](../kernel/LIGHTWEIGHT_PROFILE.md)** | Cargo feature 与 SKU |
| **[`DOCUMENTATION_INDEX.md`](DOCUMENTATION_INDEX.md)** | 全库文档索引 |
