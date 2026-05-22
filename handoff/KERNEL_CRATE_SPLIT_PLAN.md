# 内核 crate 拆分计划（`kernel_types` / `kernel_contracts` / `kernel_runtime`）

**状态**：**已完成**（2026-05-20）。Workspace 成员：`oclive_kernel_types`、`oclive_kernel_contracts`、`oclive_kernel_runtime`（编排实现 + 过渡期 re-export）。

## 目标

| 新 crate | 职责 | 依赖方向 |
|----------|------|----------|
| **`oclive_kernel_types`** | DTO、`AppError` / `KernelErrorBody`、纯数据结构（含 `PromptInput`、`MemoryRetrievalInput`、策略配置、本地插件描述符） | `serde`、`thiserror`、`chrono`、`oclive_validation`（共享磁盘契约） |
| **`oclive_kernel_contracts`** | 全部 `trait` 端口（含 `PluginHostPort`、`LlmClient`、`SlotRegistryResolver`、`EventEstimator`、`AgentProvider` 等） | `kernel_types` + `async-trait`（`SlotRegistryEntry` / `PersonalitySource` 经 types 再导出） |
| **`oclive_kernel_runtime`** | 领域引擎实现、HTTP 边界常量；**`pub use oclive_kernel_types::*`** + trait 根 re-export | `kernel_types` + `kernel_contracts` + 运行时依赖 |

**`oclivenewnew-tauri`** 继续依赖 `oclive_kernel_runtime` + `oclive_kernel_contracts`；**`domain/ports/` 无 trait 定义**（仅 re-export 与 `impl`）。

## 已完成步骤

| # | 内容 | 提交主题 |
|---|------|----------|
| 1 | 新建 `oclive_kernel_types`，迁入 `error.rs`、`models/`、`ComplexEmotion*`、`EmotionResult`、策略/插件描述符 | `refactor: extract oclive_kernel_types crate with pure data structures` |
| 2 | 新建 `oclive_kernel_contracts`，迁入 repository / retrieval / policy / prompt / emotion / local_plugin trait | `refactor: extract oclive_kernel_contracts crate with core traits` |
| 3 | 收窄 `oclive_kernel_runtime` `lib.rs`，移除 crate 内 trait 定义 | `refactor: clean up oclive_kernel_runtime after type and trait extraction` |
| 4 | 过渡期 `pub use oclive_kernel_types::*` + trait 根导出 + `kernel_types` / `kernel_contracts` 别名 | `feat: add compatibility re-exports from kernel_runtime for smooth migration` |
| 5 | 依赖图审计、`cargo tree`、本文档 | `refactor: audit dependency graph after crate split` |
| 6 | `EventEstimator` / `AgentProvider` trait + `EventImpactEstimate` / `AgentInput`·`Output` 类型 | `refactor(contracts): abstract EventEstimator…` / `…AgentProvider…` |
| 7 | `domain/ports/` 零 trait、`rg '^pub trait'` 为空 | `refactor(contracts): finalize port cleanup…` |

## 非目标（本阶段）

- 不移动 `src-tauri/domain/*` 到 workspace 外（另见 [ARCHITECTURE_LAYERING.md](ARCHITECTURE_LAYERING.md)）。
- 不在拆分同时改 `API_VERSION` / 错误码形状。

## 迁移建议

- **新代码**：`oclive_kernel_types::AppError`、`oclive_kernel_contracts::MemoryRetrieval`。
- **旧路径（过渡期）**：`oclive_kernel_runtime::AppError` 或 `oclive_kernel_runtime::kernel_types::…`。
- **勿**对 `oclive_kernel_contracts::*` 做 crate 根 glob re-export（与 `types` 子模块名 `complex_emotion` / `policy` / `memory_retrieval` 冲突）。

## 依赖图

```text
oclive_validation
       ^
       |
oclive_kernel_types
       ^
       |
oclive_kernel_contracts
       ^
       |
oclive_kernel_runtime ──► oclivenewnew-tauri
```

审计命令：

```bash
cargo tree -p oclive_kernel_runtime -d
cargo tree -p oclive_kernel_contracts -d
cargo tree -p oclive_kernel_types -d
```

## 模块可见性（并行）

- 各 crate 内：`pub` 仅保留稳定 API；实现模块 `pub(crate)`。
- 用 `cargo doc --document-private-items` 与 `rg '^pub use'` 审计对外表面。
