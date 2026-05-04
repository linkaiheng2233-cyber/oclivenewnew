//! 记忆排序 / 上下文 / 搜索纯函数（无 I/O）。
//!
//! - **`classic` feature 开启**（默认）：与历史行为一致的完整算法（`full`）。
//! - **关闭**：轻量桩（`stub`），用于极简宿主；语义见仓库 `FACILITY_CLASSIC_ALGORITHMS_AUDIT.md`。

#[cfg(feature = "classic")]
mod full;
#[cfg(not(feature = "classic"))]
mod stub;

#[cfg(feature = "classic")]
pub use full::*;
#[cfg(not(feature = "classic"))]
pub use stub::*;
