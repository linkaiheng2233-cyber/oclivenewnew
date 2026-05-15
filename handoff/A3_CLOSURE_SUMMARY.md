# A3 结论汇总（崩溃与诊断，2026-05-15）

## A3.1 Sentry（默认关、可关、与 README 一致）

- **构建期**：仍仅当设置 **`VITE_SENTRY_DSN`** 时才有可能初始化（未配置则零上报）。
- **运行时**：新增 **`src/utils/telemetrySentry.ts`**，`localStorage` 键 **`oclive.telemetry.sentryOptOut`** 为 **`1`** 时跳过 `Sentry.init`（与设置页同步）。
- **设置 UI**：若构建带 DSN，**设置 → 常规** 显示 **「崩溃诊断（Sentry）」** 区：勾选 **禁用崩溃上报** 即 `Sentry.close` + 写 opt-out；取消勾选提示 **重启应用** 后恢复。
- **隐私**：`sendDefaultPii: false`、`tracesSampleRate: 0`、`beforeSend` 去掉请求 URL 的 **query**。
- **文档**：根 **`README.md`** / **`README.en.md`**（Observability 小节）已对齐上述行为。

## A3.2 用户可见错误（JSON `code` + 前端映射）

- **目录插件 `ApiError`**：`src-tauri/src/api/error.rs` 与 **`KernelErrorBody` 同源单行 JSON**（不再主路径依赖 `[CODE]`）；`map_directory_rpc_url_error`、插件桥 **`Result<_, String>`** 路径统一 **`.into()`**。
- **遗漏码兜底**：`apiErrors.UNKNOWN_WITH_CODE`（中/英）+ **`toFriendlyErrorMessage`** 在无词条时展示带 **`{code}`** 的友好句。
- **扫尾**：`reset_plugin_state_to_role_default` 中 `load_role` 失败改为 **`to_frontend_error()`**（与内核 JSON 一致）。

## 验收建议

- `npm run test:unit`
- `cargo test -p oclivenewnew-tauri`（至少 `api::error` 单测）
- 手工：带 DSN 的构建打开设置，切换禁用/启用并确认重启提示与 opt-out 键行为。

## 相关

- 发版勾选：`handoff/PRODUCT_RELEASE_CHECKLIST.md` §A3  
- 产品缺口表：`handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md` §A3  
- 错误码规范：`creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md`
