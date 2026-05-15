# A2 补丁：内核 JSON 错误体（权威契约）

**日期**：2026-05-15  
**动机**：以无头内核为单一事实来源；发行版（Tauri 壳、HTTP `--api`、将来其他宿主）**只消费同一 JSON 形状**，避免为每条传输层维护一套错误字符串。

## 契约

### `KernelErrorBody`（`oclive_kernel_runtime`）

| 字段 | 说明 |
|------|------|
| `code` | 机器码，**`SCREAMING_SNAKE_CASE`**，与 `AppError::code()` 或 HTTP 路由专有码一致。 |
| `message` | `AppError` 的 `Display`（默认英文技术句）；本地化由壳用 `code` → `apiErrors.*`。 |
| `hint` | 可选；内核默认 `None`；HTTP `/chat` 可为试聊附加中文提示。 |

### Tauri `invoke`

- 失败时载荷为 **`AppError::to_frontend_error()`**，即 **`to_kernel_json()`** 的 **单行 JSON 字符串**（非 `[CODE]` 前缀格式）。
- 前端 **`parseBackendError`**：优先 `JSON.parse`；失败则回退 legacy **`[CODE]`** 解析（兼容旧日志/脚本）。

### HTTP `POST /chat`

- 响应体 **`{ "error": KernelErrorBody }`**，字段与上表一致。
- **路由层专有**：`EMPTY_MESSAGE`、`INVALID_ROLE_PATH`、`LOAD_ROLE_TASK_PANIC`。
- **加载/引擎失败**：直接使用 **`e.kernel_error_body()`**，`code` 为真实内核码（如 **`ROLE_NOT_FOUND`**、**`LLM_ERROR`**），不再使用已移除的笼统码 `chat_engine_failed` / `load_role_failed` / snake_case 旧名。

## 验收

- `cargo test -p oclivenewnew-tauri http_api`
- `npm run test:unit`
- `node examples/oocp-test-suite/run.mjs`（对正在运行的 `--api` 实例）

## 相关文档

- **单一规范（命名 + 形状 + 传输层）**：`creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md`（英文镜像 `creator-docs-en/getting-started/KERNEL_ERROR_CODE_CONVENTION.md`）
- `creator-docs/getting-started/ERROR_CODES.md` §1（中）及英文镜像（排障表）
- `creator-docs/testing/OOCP_TEST_SUITE.md`  
- 总览：`handoff/A2_CLOSURE_SUMMARY.md`
