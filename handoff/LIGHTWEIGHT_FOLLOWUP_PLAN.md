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

---

## 阶段 3：生成物与 SKU 防呆

- **风险**：极简 `invoke` 组合下 `build.rs` 会重写 `src/gen/tauri-invoke-capabilities.ts`；误提交「全 `false`」会破坏默认前端契约。
- **建议**：CI 增加一步（或在 `WEEKLY_DEV_GUIDE` 写清单）：在默认 **`invoke-full`** 下 `cargo check -p oclivenewnew-tauri` 后 `git diff --exit-code src/gen/tauri-invoke-capabilities.ts`。
- **维护**：新增强可选分组命令时同步 **Rust 宏列表** + **`COMMAND_CAPABILITY`**（见 `LIGHTWEIGHT_PROFILE` §4.2）。

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

1. 阶段 3（低成本防呆）可与任意阶段并行。  
2. 阶段 1 独立评审合并后再做阶段 2（减少链接与路由同时大改的风险）。  
3. 阶段 4 独立里程碑，不与 1/2 混在同一 PR。
