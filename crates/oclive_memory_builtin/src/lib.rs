//! 内置记忆排序算法与可选的 [`MemoryRetrieval`](oclive_kernel_core::memory_retrieval::MemoryRetrieval) 实现。
//!
//! **始终可用**：[`classic`]（纯函数，无 I/O）。  
//! **可选**（`providers` feature）：[`BuiltinMemoryRetrieval`] / [`BuiltinMemoryRetrievalV2`]。

pub mod classic;

#[cfg(feature = "providers")]
mod providers;

#[cfg(feature = "providers")]
pub use providers::{BuiltinMemoryRetrieval, BuiltinMemoryRetrievalV2};
