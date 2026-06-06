//! Reply Post-Processor —修饰 LLM 原始回复，不负责持久化（非六槽）。

use oclive_kernel_types::Result;

/// Input for one post-LLM reply polish pass.
pub struct PostProcessInput<'a> {
    pub raw_reply: &'a str,
    pub user_message: &'a str,
    pub role_id: &'a str,
    pub scene_id: &'a str,
    pub srid: &'a str,
    pub locale: &'a str,
}

/// Output of [`ReplyPostProcessor::process_reply`].
pub struct PostProcessOutput {
    pub display_reply: String,
    pub diagnostic: Option<String>,
}

/// Reply Post-Processor Plugin trait (`builtin` / `remote` / `directory` backends).
pub trait ReplyPostProcessor: Send + Sync {
    /// Transform raw LLM text into user-visible display reply.
    ///
    /// # Errors
    ///
    /// Implementations should return `Err` only on unrecoverable failure; host may fall back to raw.
    fn process_reply(&self, input: PostProcessInput<'_>) -> Result<PostProcessOutput>;
}
