//! 内置复杂情感关键词模式与七维→效价/掌控感辅助（`classic` feature，默认开）。

pub mod classic;

/// 生产环境由 `feature = "providers"` 开启；单元测试始终编译以便覆盖关键词路径。
#[cfg(any(feature = "providers", test))]
mod providers;

#[cfg(any(feature = "providers", test))]
pub use providers::BuiltinKeywordComplexEmotionProvider;
