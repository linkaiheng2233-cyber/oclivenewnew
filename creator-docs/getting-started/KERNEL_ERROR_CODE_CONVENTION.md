# 内核错误码与 JSON 体规范（单一事实来源）

**状态**：现行契约（与 `oclive_kernel_runtime::KernelErrorBody`、`AppError::code` 实现一致）。

## 1. 机器码 `code`（唯一命名规则）

- **形态**：仅使用 **`SCREAMING_SNAKE_CASE`**（大写 ASCII + 下划线）。
- **来源**：
  - 绝大多数错误对应 [`AppError`](../../kernel/crates/oclive_kernel_runtime/src/error.rs) 变体，**`code` 必须与 `AppError::code()` 返回值一致**（如 `ROLE_NOT_FOUND`、`LLM_ERROR`、`TXN_*`）。
  - **目录插件宿主 `ApiError`**（`distros/desktop-tauri/src/api/error.rs`）：与 `KernelErrorBody` **同形 JSON 单行**（`code` 仍为 `SCREAMING_SNAKE_CASE`，如 **`API_PLUGIN_NOT_FOUND`**）。
  - **HTTP `POST /chat` 路由边界**（请求校验、`spawn_blocking` panic 等）与 **`AppError::EmptyMessage`**（Tauri `send_message` / HTTP 空消息校验共用 **`EMPTY_MESSAGE`**）使用 crate 内常量模块 **`http_chat_codes`**（与实现同仓，避免字面量漂移）：
    - `EMPTY_MESSAGE`
    - `INVALID_ROLE_PATH`
    - `LOAD_ROLE_TASK_PANIC`
- **禁止**：在对外 JSON 的 `code` 字段使用 **camelCase / snake_case 小写**（历史 OOCP 试聊码已废弃）；新码不得再引入第二套命名风格。

## 2. 载荷形状 `KernelErrorBody`

| 字段 | 规则 |
|------|------|
| `code` | 上节机器码。 |
| `message` | 技术向英文句（`AppError` 的 `Display` 或路由层构造）；用户可见本地化由宿主用 **`code` → i18n**（如前端 `apiErrors.*`）。 |
| `hint` | 可选；内核默认省略；HTTP 试聊可为中文「下一步」。 |

## 3. 传输层（仅包装不同，字段相同）

| 通道 | 形式 |
|------|------|
| **Tauri `invoke` 失败** | 失败字符串为 **单行 JSON**，即 `serde_json` 序列化后的单个 `KernelErrorBody`（不是 `[CODE] message` 主格式）。 |
| **HTTP `POST /chat` 失败** | JSON 对象 **`{ "error": KernelErrorBody }`**。 |

宿主与脚本应 **先 `JSON.parse`**；解析失败时再回退 legacy **`[CODE]`** 前缀（旧日志/旧构建）。

## 4. 与 JSON-RPC 侧车错误的关系

侧车协议层使用 **JSON-RPC `code`（整数）+ `message`（小写 snake 名）** 等约定，见 [ERROR_CODES.md §2](ERROR_CODES.md)。**不得**把 RPC 整数码写进本规范的 `KernelErrorBody.code`；两套体系并存时各用其字段，不在同一 payload 混用。

## 5. 变更纪律

- 新增 **`AppError` 变体** 时：实现 `code()`、**补前端 `apiErrors`（中英）**、必要时补本文档姊妹表 [ERROR_CODES.md](ERROR_CODES.md)。
- 新增 **HTTP 边界专有码**：先在 `http_chat_codes` 增加 `pub const`，再在路由中使用；并在 [ERROR_CODES.md](ERROR_CODES.md) §1 表格登记。

## 6. 相关实现与补丁说明

- Rust：`kernel/crates/oclive_kernel_runtime/src/error.rs`（`KernelErrorBody`、`AppError`、`http_chat_codes`）。
- HTTP：`kernel/crates/oclive_kernel_host/src/http_api.rs`。
- 补丁摘要：`handoff/A2_KERNEL_JSON_ERROR_PATCH.md`。
- **A3（崩溃上报与用户可见错误扫尾）**：[`handoff/A3_CLOSURE_SUMMARY.md`](../../handoff/A3_CLOSURE_SUMMARY.md) · [`handoff/A3_CLOSURE_SUMMARY.en.md`](../../handoff/A3_CLOSURE_SUMMARY.en.md)。

## 7. 目录插件 RPC 字符串 → `ApiError` 映射

`map_directory_rpc_url_error`（`oclive_kernel_host::command_error`）将目录插件 spawn 相关 plain-text 失败映射为：

| 子串 / 前缀 | `ApiError` → `code` |
|-------------|---------------------|
| `directory plugin spawn not granted` | `HIGH_RISK_CAPABILITY_NOT_GRANTED` |
| `directory plugin spawn not permitted` | `HIGH_RISK_CAPABILITY_NOT_GRANTED` |
| `plugin disabled:` | `API_PERMISSION_DENIED` |
| `unknown directory plugin_id=` | `API_PLUGIN_NOT_FOUND` |

[English mirror](../../creator-docs-en/getting-started/KERNEL_ERROR_CODE_CONVENTION.md)
