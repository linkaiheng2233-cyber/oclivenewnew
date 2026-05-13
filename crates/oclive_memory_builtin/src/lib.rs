//! 内置记忆排序算法与可选的 [`MemoryRetrieval`](oclive_kernel_core::memory_retrieval::MemoryRetrieval) 实现。
//!
//! **可选**（`classic` feature，默认开）：[`classic`] 纯函数；关闭时用轻量桩。  
//! **可选**（`providers` feature）：[`BuiltinMemoryRetrieval`] / [`BuiltinMemoryRetrievalV2`]（隐含 `classic`）。

pub mod classic;

#[cfg(feature = "providers")]
mod providers;

#[cfg(feature = "providers")]
pub use providers::{BuiltinMemoryRetrieval, BuiltinMemoryRetrievalV2};
