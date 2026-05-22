//! Agent 调度可替换门面 trait。

use async_trait::async_trait;
use oclive_kernel_types::{AgentInput, AgentOutput, Result};

/// Pluggable agent backend for tool-using or multi-step replies.
///
/// # Examples
///
/// ```no_run
/// use oclive_kernel_contracts::AgentProvider;
/// use oclive_kernel_types::AgentInput;
///
/// async fn run(agent: &dyn AgentProvider) -> oclive_kernel_types::Result<()> {
///     let input = AgentInput {
///         role_id: "demo".into(),
///         session_namespace: "default".into(),
///         message: "查一下北京天气".into(),
///         model: "qwen2.5:7b".into(),
///     };
///     let out = agent.process(input).await?;
///     let _ = out.reply;
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait AgentProvider: Send + Sync {
    /// 处理单轮 Agent 任务（工具调用、多步推理等）。
    ///
    /// # Errors
    ///
    /// 当工具调用被拒绝、MCP/HTTP 失败、LLM 返回非法结构或超时时返回 `Err`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    async fn process(&self, input: AgentInput) -> Result<AgentOutput>;
}
