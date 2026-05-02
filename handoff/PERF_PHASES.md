# 性能优化阶段（定序与状态）

**v0.2 路线：P1～P3 已收尾**（本文件为单一事实源；子文档见下表「验收」）。P4 为可选后续专项，不阻塞版本与功能交付。远端推送由维护者自行安排。

## 收尾说明（v0.2）

- **纳入收尾**：首包 chunk 策略、locale 按需、Release/tokio 依赖策略、门禁（`npm run check` / `npm run check:release`）与 handoff 互链。
- **不纳入 v0.2 必达**：P4（workspace 去掉 `reqwest` `blocking`）；动刀前须单独设计 async 边界（见下「P4 未动原因」）。
- **维护约定**：新增大体积前端模块时，优先 **`defineAsyncComponent` + `v-if`**；新增文案键时维持 locale 文件结构，无需把两套语言重新打回主包。

| 阶段 | 内容 | 状态 |
|------|------|------|
| **P1** | 前端首包：`App.vue` 大块 `defineAsyncComponent` + `v-if`；`vite` `manualChunks`（vue-flow / i18n / pinia / tauri / scroller / sfc-loader）；`main.js` Sentry 延后 | ✅ |
| **P2** | 文案包：`i18n` 仅 `mergeLocaleMessage` 当前 `effectiveLocale`，另一语言 `import()` 预取 | ✅ |
| **P3** | Rust 发行：`[profile.release]`（`opt-level=z`、`lto=thin`、`codegen-units=1`）；workspace `tokio` 瘦 feature | ✅ |
| **P4** | 去掉 workspace `reqwest` 的 **`blocking`** feature，全链路改 **`reqwest::Client` + async** | ⏸ 未做（见下） |

## P4 未动原因（避免 silent 回归）

- **`McpClient`**、`invoke_directory_plugin_rpc_blocking`、若干 `remote_plugin/*_http` 与 **`PluginHost::call_mcp_tool`** 等为 **同步 API**，由 Tauri **`invoke`** 与 OOCp 路径直接调用；改为 async 需 **trait / `PluginHost` / 相关 `invoke` 命令** 一并改为 async 或 `spawn_blocking` 策略，并核对 **不在 async 上下文中 `block_on`**。
- 侧车 `oclive-llama-sidecar` 使用 **独立** `Cargo.toml` 的 reqwest，与 workspace 解耦。
- 门禁保持：`npm run check` / `npm run check:release`；Windows 全量测试可用 `scripts/check.ps1`（`CARGO_BUILD_JOBS=1` 缓解 LNK1104）。

## 验收与对照

- 前端：`npm run build`、`npm run build:analyze`（`dist/stats.html`）；基线数字见 **`FRONTEND_CHUNK_OPTIMIZATION.md`**、**`PERFORMANCE_BASELINE_ACCEPTANCE.md`**。
- Rust：见 **`RUST_RELEASE_AND_DEPENDENCIES.md`**。

## 周节奏入口

**`WEEKLY_DEV_GUIDE.md`** 中「首包 / chunk」条目指向本文件与上述子文档，避免多处重复长段。
