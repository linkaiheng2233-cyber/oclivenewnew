# 内核 crate 拆分计划（`kernel_types` / `kernel_contracts` / `kernel_runtime`）

**状态**：路线图（未开工）。当前仍使用单一 **`oclive_kernel_runtime`**。

## 目标

| 新 crate | 职责 | 依赖方向 |
|----------|------|----------|
| **`oclive_kernel_types`** | DTO、`AppError` / `KernelErrorBody`、纯数据结构 | 仅 `serde` 等基础库 |
| **`oclive_kernel_contracts`** | `trait` 端口（`LlmClient`、`MemoryRetrieval` 等） | `kernel_types` |
| **`oclive_kernel_runtime`** | 编排辅助、HTTP 边界、无 Tauri 的运行时胶水 | `contracts` + `infrastructure` 适配 |

**`oclivenewnew-tauri`** 继续依赖 `runtime`；**`oclive_validation`** 保持独立，避免与 `types` 循环。

## 非目标（本阶段）

- 不移动 `src-tauri/domain/*` 到 workspace 外（另见 [ARCHITECTURE_LAYERING.md](ARCHITECTURE_LAYERING.md)）。
- 不在拆分同时改 `API_VERSION` / 错误码形状。

## 建议步骤（每步可独立 PR）

1. 新建 `crates/oclive_kernel_types`，迁入 `models/` + `error.rs`（`AppError`）。
2. 新建 `crates/oclive_kernel_contracts`，迁入 `domain/ports/*` 与插件 trait 定义。
3. 收窄 `oclive_kernel_runtime` 为 re-export 兼容层（一个发版周期 `pub use` 旧路径）。
4. `cargo udeps` + 文档索引更新；Breaking 走 [BREAKING_CHANGE_PROCESS.md](BREAKING_CHANGE_PROCESS.md)。

## 模块可见性（并行）

- 各 crate 内：`pub` 仅保留稳定 API；实现模块 `pub(crate)`。
- 用 `cargo doc --document-private-items` 与 `rg '^pub use'` 审计对外表面。

## 依赖图

- 消除「A 直接依赖 C 且 B 也依赖 C」的冗余：统一经 `kernel_types` 或 `validation` 再导出。
- 工具：`cargo tree -d`、`cargo udeps`（nightly）。
