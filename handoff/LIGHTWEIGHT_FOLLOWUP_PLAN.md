# 轻量化与发行版瘦身 — 后续计划

> **状态**：轻量化主里程碑（runtime 可选特性、`http_api` 单源、壳层依赖收敛、invoke SKU、P4 `reqwest`）**已落地**；本文档转为 **防回归与维护清单**。  
> **最后核对**：以仓库 `feat/oocp-v0-1` 与 `creator-docs/kernel/LIGHTWEIGHT_PROFILE.md` §5.1、`handoff/PERF_PHASES.md` 为准。

---

## 已核对（无需重复开工）

| 项 | 结论 |
|----|------|
| Runtime `Cargo` 特性 | `full` 聚合 `kernel-http-api`、`role-pack-zip`、`market-sync`、`kernel-agent`；`--no-default-features` 在 CI 全矩阵执行。 |
| Tauri invoke SKU | `invoke_registry` + `invoke_lists/*.txt` + `build.rs` → `src/gen/tauri-invoke-capabilities.ts`；前端 `tauriInvokeCapabilities` / `tauri-api` 守卫；CI：`oclivenewnew-tauri --no-default-features --features tauri-app,custom-protocol`。 |
| 文档 | `LIGHTWEIGHT_PROFILE` §4.2 与实现对齐；`DOCUMENTATION_INDEX` / MATRIX / ENTRY_CHECKLIST / `KERNEL_MIGRATION_COMPLETE` / `handoff/README` 已互链。 |

---

## 阶段 1：`http_api` 单源（高价值，单独 PR）— ✅ 已完成

- **目标**：以 `crates/oclive_kernel_runtime/src/http_api` 为唯一路由实现；`src-tauri/src/http_api` 缩为端口解析、`KernelAppState` 构造与委托（或薄 re-export）。
- **结果**：实现已迁入 runtime（含 `/role-feedback` 与 `Query` 列表）；`src-tauri/src/http_api.rs` 为对 `oclive_kernel_runtime::http_api` 的 **re-export**；`http_api_chat` 测试改为引用 **`oclive_kernel_runtime`**；壳层 **`tower-http`** 已移除。
- **验收**：`cargo test -p oclive_kernel_runtime`（`http_api` 单测）、`cargo test -p oclivenewnew-tauri --test http_api_chat`、`cargo check -p oclive_kernel_server`；`LIGHTWEIGHT_PROFILE` §5.2 已更新为「已合并」。

---

## 阶段 2：`src-tauri` 依赖去重 — ✅ 核心已完成（维护余量）

- **依据**：`LIGHTWEIGHT_PROFILE` §5.1。
- **做法**（历史）：逐项核对与 kernel 重叠的直连依赖；壳层独有（`notify`、`sysinfo`、Tauri 插件）保留。
- **结果摘要**：`pack_plugin` 走 `plugin_archive::pack_plugin_directory_to_zip_deflated`；壳层已移除直连 `zip` / `reqwest` / `ed25519-dalek` / `base64` / `sha2` / `walkdir` / **`chrono` / `uuid`** / 生产路径 **`axum`**（`tower-http` 已随 `http_api` 迁出）；**`sqlx`** 与 **`axum`** 仅保留在 **`dev-dependencies`**（集成测试）。OOCP WS 仅 **`oclive_kernel_runtime::http_api`**（壳层 `domain/adapters/oocp_ws` 已移除）。**可选低优先级余量**：`sqlx` 与 kernel 的链接重复是否进一步收紧。

---

## 阶段 3：生成物与 SKU 防呆 — ✅ CI 已实施（持续维护）

- **风险**：极简 `invoke` 组合下 `build.rs` 会重写 `src/gen/tauri-invoke-capabilities.ts`；误提交「全 `false`」会破坏默认前端契约。
- **CI 现状**（`.github/workflows/ci.yml`）：在极简 Tauri `check` 后执行 `git checkout -- src/gen/tauri-invoke-capabilities.ts`，再于默认 **`invoke-full`** 下 `cargo check -p oclivenewnew-tauri`，最后 `git diff --exit-code src/gen/tauri-invoke-capabilities.ts`，防止漂移入库。
- **维护**：新增强可选分组命令时同步 **Rust 宏列表** + **`COMMAND_CAPABILITY`**（见 `LIGHTWEIGHT_PROFILE` §4.2）；与 **阶段 2 / 4** 无阻塞关系。

---

## 阶段 4：`reqwest::blocking` 收敛（P4）— ✅ 已落地（持续遵守边界）

- **依据**：runtime `README.md`、`KERNEL_API_IMPLEMENTATION_MATRIX` 模糊地带、`handoff/PERF_PHASES.md` P4。
- **现状**：workspace **`reqwest` 已无 `blocking`**；runtime 内 HTTP 为 **`reqwest::Client` + async**，对外同步 API 经 **`blocking_http::block_on`**。Tauri 侧长耗时路径仍须遵守 **`spawn_blocking`**、避免在 async worker 内嵌套 **`Handle::block_on`**（全文见 `PERF_PHASES.md`）。

---

## 非代码

- 择机 **`git push`** `feat/oocp-v0-1`，避免长期仅本地 ahead。
- 姊妹仓（launcher / pack-editor）若需极简宿主说明，链回主仓 `LIGHTWEIGHT_PROFILE` 即可，避免重复维护 feature 表。

---

## 建议执行顺序（维护期）

1. **阶段 1～4**：主交付已完成；日常以 **防回归** 为主。  
2. **阶段 3**：随 `invoke-*` / `build.rs` 变更维护 CI 与 **`src/gen/tauri-invoke-capabilities.ts`** 一致性。  
3. **阶段 2 余量**：仅低优先级（如 `sqlx` 链接收紧）；改前改后更新 **`LIGHTWEIGHT_PROFILE` §5.1**。  
4. **`LIGHTWEIGHT_OOCP_WS_AXUM_FOLLOWUP.md`**：保留为决策与回归说明；壳层生产依赖已不再直连 `axum`。

相关展开：`handoff/PERF_PHASES.md`、`creator-docs/kernel/LIGHTWEIGHT_PROFILE.md` §5.1。

---

## 下一阶段速览（防回归清单）

1. **新增 Tauri 命令**：同步 **`invoke_registry`**、**`tauriInvokeCapabilities` / `COMMAND_CAPABILITY`**、**`KERNEL_ENTRY_CHECKLIST` / MATRIX`**；默认 feature 下 `cargo check -p oclivenewnew-tauri` 保证 **`src/gen/tauri-invoke-capabilities.ts`** 与仓库一致（CI 已 `git diff --exit-code`）。  
2. **新增壳层 `Cargo` 依赖**：先对照 **`LIGHTWEIGHT_PROFILE` §5.1**，避免重新堆叠与 kernel 重叠的无谓直连。  
3. **长耗时同步 HTTP / 磁盘**：遵守 **`PERF_PHASES.md`**（`spawn_blocking`、`blocking_http`、勿在 Tokio async 内嵌套 `block_on`）。  
4. **远端**：择机 **`git push`**，减少与 `origin` 分叉。  
5. **索引**：`handoff/README.md` 已链 **`LIGHTWEIGHT_OOCP_WS_AXUM_FOLLOWUP.md`**。
