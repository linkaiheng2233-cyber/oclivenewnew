//! 内置七维情绪分析（关键词，`classic` feature 默认开）与可选 `UserEmotionAnalyzer` 实现（`providers`）。

pub mod classic;

#[cfg(feature = "providers")]
mod providers;

#[cfg(feature = "providers")]
pub use providers::{BuiltinUserEmotionAnalyzer, BuiltinUserEmotionAnalyzerV2};
