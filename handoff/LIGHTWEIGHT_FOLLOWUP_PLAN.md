# 轻量化与发行版瘦身 — 后续计划

> **状态**：承接 `LIGHTWEIGHT_PROFILE.md`、runtime 可选特性、文档互链、`invoke-*` 分组与 CI（含 Tauri 极简 `check`）已落地后的下一阶段。  
> **最后核对**：分支 `feat/oocp-v0-1`；提交链含 `feat(tauri): optional invoke-*`、文档互链、`docs(ci): lightweight profile matrix` 等。

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

## 阶段 2：`src-tauri` 依赖去重（渐进）

- **依据**：`LIGHTWEIGHT_PROFILE` §5.1。
- **做法**：逐项核对与 kernel 重复的 `sqlx` / `zip` / `axum` / `tower-http` / `reqwest` / `ed25519-dalek` 等；壳层独有（`notify`、`sysinfo`、Tauri 插件）单独列表保留。多小 PR，每步 `cargo check -p oclivenewnew-tauri` + 工作区 clippy。
- **进展（首批）**：`pack_plugin` 已委托 `oclive_kernel_runtime::infrastructure::plugin_archive::pack_plugin_directory_to_zip_deflated`；壳层移除未用 `reqwest` / `ed25519-dalek` / `base64` 及直连 `zip` / `sha2` / `walkdir`；`sqlx` 仅保留为集成测试 **`dev-dependencies`**。余量见 §5.1 表（`axum`、可选 `chrono`/`uuid` 收紧等）。

---

## 阶段 3：生成物与 SKU 防呆 — ✅ CI 已实施（持续维护）

- **风险**：极简 `invoke` 组合下 `build.rs` 会重写 `src/gen/tauri-invoke-capabilities.ts`；误提交「全 `false`」会破坏默认前端契约。
- **CI 现状**（`.github/workflows/ci.yml`）：在极简 Tauri `check` 后执行 `git checkout -- src/gen/tauri-invoke-capabilities.ts`，再于默认 **`invoke-full`** 下 `cargo check -p oclivenewnew-tauri`，最后 `git diff --exit-code src/gen/tauri-invoke-capabilities.ts`，防止漂移入库。
- **维护**：新增强可选分组命令时同步 **Rust 宏列表** + **`COMMAND_CAPABILITY`**（见 `LIGHTWEIGHT_PROFILE` §4.2）；阶段 3 以后以 **CI 与文档** 为主，不挡阶段 2 / 4 排期。

---

## 阶段 4：`reqwest::blocking` 收敛（P4）

- **依据**：runtime `README.md`、`KERNEL_API_IMPLEMENTATION_MATRIX` 模糊地带、`handoff/PERF_PHASES.md` P4。
- **范围**：远程插件、市场同步、MCP HTTP 等；与 Tauri 异步命令的 `spawn_blocking` 策略统一后再改代码。

---

## 非代码

- 择机 **`git push`** `feat/oocp-v0-1`，避免长期仅本地 ahead。
- 姊妹仓（launcher / pack-editor）若需极简宿主说明，链回主仓 `LIGHTWEIGHT_PROFILE` 即可，避免重复维护 feature 表。

---

## 建议执行顺序

1. **阶段 1、阶段 3**：已完成（`http_api` 单源；生成物防呆已在 CI 落地）。  
2. **主路径**：**阶段 2**（`src-tauri` 依赖去重 / 走 `oclive_kernel_runtime` 公开 API）→ **阶段 4**（`reqwest::blocking` 收敛，见 `PERF_PHASES.md` P4）。  
3. **阶段 3**：仅随 `invoke-*` / `build.rs` 变更维护 CI 与文档。  
4. **壳层 `axum` / OOCP WS**：中长期单独子计划，见 `handoff/LIGHTWEIGHT_OOCP_WS_AXUM_FOLLOWUP.md`（不与阶段 2 批量删依赖绑在同一 PR）。

相关展开：`handoff/PERF_PHASES.md`（P4 按模块 PR）、`creator-docs/kernel/LIGHTWEIGHT_PROFILE.md` §5.1（壳层依赖快照表）。

---

## 下一阶段速览（供 Cursor / 子 Agent 接单）

1. **阶段 2 余量**：收紧壳层 `chrono` / `uuid`（若可）；每次变更后更新 `LIGHTWEIGHT_PROFILE` §5.1 表。  
2. **`axum` / OOCP WS**：只走 `LIGHTWEIGHT_OOCP_WS_AXUM_FOLLOWUP.md`，不与大批量删依赖混 PR。  
3. **阶段 4（P4）**：`kernel_runtime` 内按模块处理 `reqwest::blocking`，对齐 `PERF_PHASES.md`。  
4. **索引**：`handoff/README.md` 已链 `LIGHTWEIGHT_OOCP_WS_AXUM_FOLLOWUP.md`。
