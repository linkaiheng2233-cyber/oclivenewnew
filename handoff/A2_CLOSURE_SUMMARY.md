# A2 结论汇总（全标准收口，2026-05-15）

## A2.1 invoke 可见路径（全码表）

- **内核**：`crates/oclive_kernel_runtime/src/error.rs` 新增 **`RoleRuntimeNotReady`**、**`StartupHealthFailed(String)`**；`to_frontend_error()` 仍为 `[CODE] message`。
- **调用点**：`load_role` 前未就绪、`role_runtime` 更新行缺失等由 **`ROLE_RUNTIME_NOT_READY`** 统一表达，替代泛型 `INVALID_PARAMETER`；`startup_health` 缓存失败回放为 **`STARTUP_HEALTH_FAILED`**（缓存存 `e.to_string()`，避免双层方括号）。
- **前端**：`src/i18n/locales/fragments/apiErrors.{zh,en}.ts` 补全 **扩展事务码**（`TXN_MEMORY_ID_FETCH_FAILED` 等）、**`ROLE_RUNTIME_NOT_READY`**、**`STARTUP_HEALTH_FAILED`**、**`PLUGIN_BACKENDS_DIRECTORY_SLOT`**；**`LLM_ERROR`** 区分 ollama / **remote**；**`UNKNOWN_ERROR`** 增加弱网/日志指引。
- **`toFriendlyErrorMessage`**（`src/utils/tauri-api.ts`）：`STARTUP_HEALTH_FAILED` 插值 `{detail}`；`INVALID_PARAMETER` 且含 `plugin_backends:` 时映射 **`PLUGIN_BACKENDS_DIRECTORY_SLOT`**。
- **文档**：`creator-docs/getting-started/ERROR_CODES.md` / 英文镜像 **§1.6** 增补 Tauri 方括号码一行。  
- **说明**：**HTTP `--api`** 仍为 JSON **`error.code`（snake_case）**，与桌面 **Tauri `[BRACKET_CODE]`** 并存为契约预期，非 bug。

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
