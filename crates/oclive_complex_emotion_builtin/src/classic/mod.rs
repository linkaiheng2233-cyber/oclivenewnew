//! 七维 → 效价 / 掌控感辅助函数。
//!
//! - **`classic` feature 开启**（默认）：完整公式（`full`）。
//! - **关闭**：恒返回 `(0.0, 0.0)` 的桩。

#[cfg(feature = "classic")]
mod full;
#[cfg(not(feature = "classic"))]
mod stub;

#[cfg(feature = "classic")]
pub use full::*;
#[cfg(not(feature = "classic"))]
pub use stub::*;
