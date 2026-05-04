use crate::error::Result;
pub use oclive_kernel_core::agent::{
    AgentDebugTrace, AgentInput, AgentOutput, AgentProvider, AgentToolCallTrace,
};
use async_trait::async_trait;

#[cfg(feature = "kernel-agent")]
#[path = "agent_builtin.rs"]
mod agent_builtin;

#[cfg(feature = "kernel-agent")]
pub use agent_builtin::BuiltinReActAgent;

/// Agent 能力关闭时的占位实现（不走 MCP / ReAct）。
pub struct NoopAgent;

#[async_trait]
impl AgentProvider for NoopAgent {
    async fn process(&self, _: AgentInput) -> Result<AgentOutput> {
        Ok(AgentOutput {
            handled: false,
            reply: String::new(),
        })
    }
}
