//! Prompt 组装可替换门面 trait。

use oclive_kernel_types::{PromptInput, Result, Role};

/// Builds the final LLM prompt string from role, scene, and turn context.
pub trait PromptAssembler: Send + Sync {
    /// 组装本回合 Prompt 正文。
    ///
    /// # Errors
    ///
    /// 模板或输入校验失败时返回 [`Result`] 的 `Err` 变体。
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn build_prompt(&self, input: &PromptInput<'_>) -> Result<String>;

    /// 返回当前场景下的主题提示（可选）。
    ///
    /// # Errors
    ///
    /// 无；本方法不返回 `Result`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn top_topic_hint(&self, role: &Role, scene_id: &str) -> Option<String>;
}
