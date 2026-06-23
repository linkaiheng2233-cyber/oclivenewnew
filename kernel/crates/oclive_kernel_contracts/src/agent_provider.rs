//! Replaceable facade trait for agent dispatch.

use async_trait::async_trait;
use oclive_kernel_types::{AgentInput, AgentOutput, Result};

/// Pluggable agent backend for tool-using or multi-step replies.
///
/// ## When to implement
///
/// - **Who**: agent / tool-calling backends (builtin ReAct, Remote, directory-plugin combinations).
/// - **When**: when **MCP / function calling / multi-step tasks** are needed and handled as a short-circuit at the `process_message` entry.
///
/// ## When not to implement
///
/// - When the role sets the `agent` slot to `none` or only uses plain co-present LLM dialogue.
/// - Role packs that need no tool capabilities do not need to implement this.
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
    /// Processes a single-turn agent task (tool calls, multi-step reasoning, etc.).
    ///
    /// # Errors
    ///
    /// Returns `Err` when a tool call is denied, MCP/HTTP fails, the LLM returns an invalid structure, or a timeout occurs.
    ///
    /// # Panics
    ///
    /// Does not panic.
    async fn process(&self, input: AgentInput) -> Result<AgentOutput>;
}
