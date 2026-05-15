# A2 结论汇总（全标准收口，2026-05-15）

## A2.1 invoke 可见路径（全码表）

- **内核**：`crates/oclive_kernel_runtime/src/error.rs` 定义 **`KernelErrorBody` `{ code, message, hint? }`**；**`AppError::to_kernel_json()` / `to_frontend_error()`** 均返回 **单行 JSON**（与 HTTP `error` 内层同形）；**`kernel_error_body()`** 供 HTTP 组装。
- **调用点**：（略，同前版）`ROLE_RUNTIME_NOT_READY`、`STARTUP_HEALTH_FAILED` 等。
- **前端**：`src/utils/tauri-api.ts` **`parseBackendError`** 优先解析 JSON，**保留 `[CODE]` 回退**；`FriendlyError` 增加可选 **`kernel`**；`apiErrors` 增补 **`EMPTY_MESSAGE` / `INVALID_ROLE_PATH` / `LOAD_ROLE_TASK_PANIC`**。
- **HTTP `--api`**：`src-tauri/src/http_api.rs` 的 **`error`** 直接使用 **`KernelErrorBody`**；`POST /chat` 专有码 **`EMPTY_MESSAGE`**、**`INVALID_ROLE_PATH`**；加载/引擎失败使用 **`AppError::code()`**（如 **`ROLE_NOT_FOUND`**、**`LLM_ERROR`**），不再使用 `chat_engine_failed` 等第二层笼统码。
- **OOCP / 测试**：`examples/oocp-test-suite/run.mjs`、`src-tauri/tests/http_api_chat.rs` 断言已对齐 **`SCREAMING_SNAKE_CASE`**。
- **文档**：`ERROR_CODES.md`（中/英）§1、`OOCP_TEST_SUITE.md`、`bug_report.yml` 已更新。

## A2 补丁：内核 JSON 错误体（权威契约，2026-05-15）

- **目标**：传输层不各自发明字符串；**Tauri `invoke` 失败载荷**与 **`POST /chat` 的 `error` 对象**字段同源（`KernelErrorBody`）。
- **实现要点**：见 **`handoff/A2_KERNEL_JSON_ERROR_PATCH.md`**（与本文互链）。

## A2.2 环境自检

- 见上一版：`run_environment_diagnostics` + 设置 → 常规 → 环境自检。

## A2.3 离线 / 弱网（全产品可见性增强）

- **插件索引**：同步失败 → 缓存 + 工作台 i18n（既有）+ **`pluginStore.syncPluginMarket` 成功后驱动 `uiStore` 顶栏下全局提示条**（`App.vue` `connectivity-banner`），可关闭；在线同步成功则清除 **`plugin_index_offline`** 条。
- **`uiStore.persist`** 改为 **`pick: ['sceneId','experimentalPluginManagerV2']`**，避免把临时 **`connectivityBanner`** 写入 localStorage。
- **`check_plugin_updates`**：去掉硬编码英文 `message`，由 Toast 文案 **`pluginWorkbench.toast.checkDone`** 说明在线比对未接线。

## 验收

- `cargo check -p oclivenewnew-tauri`；`cargo test -p oclive_kernel_runtime`
- `npm run test:unit`（含 i18n parity）
- 手动：断网同步社区索引 → 工作台提示 + 顶栏下横幅；联网同步成功 → 横幅消失。

## 发版清单

- `handoff/PRODUCT_RELEASE_CHECKLIST.md`：**A2.1 / A2.2 / A2.3** 已按全标准描述勾选。
