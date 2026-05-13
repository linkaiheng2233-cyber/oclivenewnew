//! 关键词七维情绪分析。
//!
//! - **`classic` feature 开启**（默认）：完整关键词表（`full`）。
//! - **关闭**：恒返回强中性分布的桩，用于裁剪二进制与编译时间。

#[cfg(feature = "classic")]
mod full;
#[cfg(not(feature = "classic"))]
mod stub;

#[cfg(feature = "classic")]
pub use full::*;
#[cfg(not(feature = "classic"))]
pub use stub::*;
