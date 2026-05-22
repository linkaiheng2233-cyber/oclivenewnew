//! Agent 调度可替换门面 trait。

use async_trait::async_trait;
use oclive_kernel_types::{AgentInput, AgentOutput, Result};

/// Pluggable agent backend for tool-using or multi-step replies.
///
/// ## When to implement
///
/// - **谁**：Agent / 工具调用后端（内置 ReAct、Remote、目录插件组合）。
/// - **何时**：需要 **MCP / 函数调用 / 多步任务** 并在 `process_message` 入口短路处理时。
///
/// ## When not to implement
///
/// - 角色将 `agent` 槽设为 `none` 或仅走普通共景 LLM 对话时。
/// - 不需要工具能力的角色包无需实现。
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
