//! 复杂情感解析可替换门面 trait。

use oclive_kernel_types::{ComplexEmotionInput, ComplexEmotionOutput, Result};

pub trait ComplexEmotionProvider: Send + Sync {
    /// # Errors
    ///
    /// Propagates [`oclive_kernel_types::AppError`] from the underlying implementation.
    fn resolve_turn(&self, input: &ComplexEmotionInput) -> Result<ComplexEmotionOutput>;
}
