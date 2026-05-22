//! 复杂情感解析可替换门面 trait。

use oclive_kernel_types::{ComplexEmotionInput, ComplexEmotionOutput, Result};

/// Resolves complex emotion labels and narrative hints for co-present turns.
///
/// ## When to implement
///
/// - **谁**：复杂情感 / `narrative_hint` 提供方（内置关键词、Remote、目录插件）。
/// - **何时**：角色需要**抽象情感叙事提示**写入下一轮 Prompt 时。
///
/// ## When not to implement
///
/// - `complex_emotion` 槽为 `none` 或不需要 `narrative_hint` 的简单角色。
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
