# 性能优化阶段（定序与状态）

**v0.2 路线：P1～P3 已收尾**（本文件为单一事实源；子文档见下表「验收」）。P4 为可选后续专项，不阻塞版本与功能交付。远端推送由维护者自行安排。

## 收尾说明（v0.2）

- **纳入收尾**：首包 chunk 策略、locale 按需、Release/tokio 依赖策略、门禁（`npm run check` / `npm run check:release`）与 handoff 互链。
- **不纳入 v0.2 必达**：P4 曾列为可选；现已落地 workspace 去 `reqwest/blocking` 与 runtime 迁移（见下「P4 实施说明」）。
- **维护约定**：新增大体积前端模块时，优先 **`defineAsyncComponent` + `v-if`**；新增文案键时维持 locale 文件结构，无需把两套语言重新打回主包。

| 阶段 | 内容 | 状态 |
|------|------|------|
| **P1** | 前端首包：`App.vue` 大块 `defineAsyncComponent` + `v-if`；`vite` `manualChunks`（vue-flow / i18n / pinia / tauri / scroller / sfc-loader）；`main.js` Sentry 延后 | ✅ |
| **P2** | 文案包：`i18n` 仅 `mergeLocaleMessage` 当前 `effectiveLocale`，另一语言 `import()` 预取 | ✅ |
| **P3** | Rust 发行：`[profile.release]`（`opt-level=z`、`lto=thin`、`codegen-units=1`）；workspace `tokio` 瘦 feature | ✅ |
| **P4** | 去掉 workspace `reqwest` 的 **`blocking`** feature；runtime 内 HTTP 走 **`reqwest::Client` + async**，同步边界经 `blocking_http::block_on` 小运行时桥接 | ✅ workspace 已关 `blocking`；runtime 已迁移（见下） |

## P4 实施说明（避免 silent 回归）

- **同步 API 保留**：`McpClient`、`invoke_directory_plugin_rpc_blocking`、若干 `remote_plugin` 同步 trait 与 **`PluginHost::call_mcp_tool`** 等签名未改；HTTP 在实现内改为 **`reqwest::Client` + `.await`**，由 **`infrastructure::blocking_http::block_on`**（独立多线程 runtime）桥接。**注意**：从 **Tokio 异步任务** 调用这些同步 API 仍会阻塞当前 worker（与原先 `reqwest::blocking` 同类风险）；长路径应继续优先 **`tokio::task::spawn_blocking`** 或在 async 边界直接 **`call_async`**。
- 侧车 `oclive-llama-sidecar` 使用 **独立** `Cargo.toml` 的 reqwest，与 workspace 解耦。
- 门禁保持：`npm run check` / `npm run check:release`；Windows 全量测试可用 `scripts/check.ps1`（`CARGO_BUILD_JOBS=1` 缓解 LNK1104）。

### P4 迁移批次（按模块拆分 PR）

已在 **`crates/oclive_kernel_runtime/src/infrastructure/`** 收敛：**workspace `reqwest` 不再启用 `blocking`**；各模块使用 **`reqwest::Client` + `.await`**，仍对外暴露同步签名的入口（市场同步、`invoke_directory_plugin_rpc_blocking`、同步 `MemoryRetrieval` 等）在内部经 **`blocking_http::block_on`**（专用 Tokio 多线程 runtime）驱动 async 客户端，避免在异步任务中嵌套 `Handle::block_on`。

1. **`mcp_client.rs`** — MCP HTTP：`call_raw_http` 已 async 化 + `block_on`
2. **`plugin_index_sync.rs`**、**`plugin_reviews_index_sync.rs`**、**`role_market_index_sync.rs`** — 市场索引 HTTP
3. **`plugin_install.rs`**、**`role_pack_archive.rs`** — 安装与归档下载
4. **`remote_plugin/`** — `jsonrpc::call_blocking` 委托 `call_async`；`memory` / `emotion` / `prompt` / `complex_emotion` HTTP 与 `invoke_directory_plugin_rpc_blocking` 使用 **`reqwest::Client`**

### `spawn_blocking` 与 async 边界（过渡约定）

- **Tauri `invoke` 命令体**：在 Tokio runtime 上执行；若仍须调用 **长时间** 同步内核 API（含经 `blocking_http` 的 HTTP），宜 **`tokio::task::spawn_blocking`** 包裹；**避免** 在 async 任务内对 **同一** Tokio runtime 再 `Handle::block_on`。
- **内核侧**：`http_api` 等对磁盘与 `RoleStorage` 的阻塞访问已用 `spawn_blocking`（见 `oclive_kernel_runtime::http_api`）；新建纯 async 路径可优先 **`reqwest::Client` + `.await`**（如 `jsonrpc::call_async`），不必再经 `blocking_http`。
- **目标态（已达成）**：workspace `reqwest` 为 **`json` + `default-tls`（+ `gzip` 等）**，**无 `blocking`**；对外同步 API 与 `blocking_http` 的边界见 `crates/oclive_kernel_runtime/src/infrastructure/blocking_http.rs`。
- **runtime 内锚点清单**（`spawn_blocking` 文件表、`KernelAppState` 冷启动分段建议）：[`P1_KERNEL_RUNTIME_BLOCKING_AND_STARTUP.md`](./P1_KERNEL_RUNTIME_BLOCKING_AND_STARTUP.md)。

## 验收与对照

- 前端：`npm run build`、`npm run build:analyze`（`dist/stats.html`）；基线数字见 **`FRONTEND_CHUNK_OPTIMIZATION.md`**、**`PERFORMANCE_BASELINE_ACCEPTANCE.md`**。
- Rust：见 **`RUST_RELEASE_AND_DEPENDENCIES.md`**。

## 周节奏入口

**`WEEKLY_DEV_GUIDE.md`** 中「首包 / chunk」条目指向本文件与上述子文档，避免多处重复长段。
