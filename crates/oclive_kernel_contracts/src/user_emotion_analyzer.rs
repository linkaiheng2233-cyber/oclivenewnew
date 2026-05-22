//! 用户/文本情绪分析可替换门面 trait。

use oclive_kernel_types::{EmotionResult, Result};

/// Analyzes user text into a seven-dimensional [`EmotionResult`].
///
/// ## When to implement
///
/// - **谁**：情绪分析后端（内置关键词 / LLM、Remote）。
/// - **何时**：共景路径需要分析**用户消息情绪**以驱动事件与 Prompt 时。
///
/// ## When not to implement
///
/// - 使用默认 builtin 分析且无需替换时；或角色禁用 emotion 相关能力时。
pub trait UserEmotionAnalyzer: Send + Sync {
    /// 分析用户文本并产出七维情绪结果。
    ///
    /// # Errors
    ///
    /// Returns an error when the analyzer cannot produce an [`EmotionResult`].
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn analyze(&self, text: &str) -> Result<EmotionResult>;
}
