//! 复杂情感解析可替换门面 trait。

use oclive_kernel_types::{ComplexEmotionInput, ComplexEmotionOutput, Result};

/// Resolves complex emotion labels and narrative hints for co-present turns.
pub trait ComplexEmotionProvider: Send + Sync {
    /// 解析本回合复杂情感标签与叙事提示。
    ///
    /// # Errors
    ///
    /// Propagates [`oclive_kernel_types::AppError`] from the underlying implementation.
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn resolve_turn(&self, input: &ComplexEmotionInput) -> Result<ComplexEmotionOutput>;
}
