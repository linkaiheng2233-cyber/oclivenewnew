//! Oclive **kernel V2 核心层**（`oclive_kernel_core`）：协议与数据访问端口的共享基础类型。
//!
//! 当前阶段迁入：`AppError`、记忆模型、`MemoryRepository` / `FavorabilityRepository` / `ExpertModelsRepository`。
//! 编排、SQLite、HTTP、插件运行时等仍在 `oclive_kernel_runtime` crate。

pub mod error;
pub mod models;
pub mod repository;
