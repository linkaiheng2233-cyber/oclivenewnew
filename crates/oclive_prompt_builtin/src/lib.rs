//! 内置提示词组装与可选的 [`PromptAssembler`](oclive_kernel_core::prompt::PromptAssembler) 实现。
//!
//! **可选**（`classic` feature，默认开）：[`PromptBuilder`] 算法；关闭时用轻量桩。  
//! **可选**（`providers` feature）：[`BuiltinPromptAssembler`] / [`BuiltinPromptAssemblerV2`]（隐含 `classic`）。

pub mod classic;

#[cfg(feature = "providers")]
mod providers;

#[cfg(feature = "providers")]
pub use providers::{BuiltinPromptAssembler, BuiltinPromptAssemblerV2, PROMPT_BACKEND_V2_PREFIX};

pub use classic::PromptBuilder;
