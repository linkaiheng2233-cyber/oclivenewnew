//! 经典提示词组装（纯逻辑，无 I/O）。
//!
//! - **`classic` feature 开启**（默认）：与历史行为一致的完整算法（`full`）。
//! - **关闭**：轻量桩（`stub`），用于极简宿主。

#[cfg(feature = "classic")]
mod full;
#[cfg(not(feature = "classic"))]
mod stub;

#[cfg(feature = "classic")]
pub use full::PromptBuilder;
#[cfg(not(feature = "classic"))]
pub use stub::PromptBuilder;
