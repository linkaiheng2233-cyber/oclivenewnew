use crate::error::Result;
pub use oclive_kernel_core::agent::{
    AgentDebugTrace, AgentInput, AgentOutput, AgentProvider, AgentToolCallTrace,
};
use async_trait::async_trait;

#[cfg(all(feature = "kernel-agent", feature = "default-agent-providers"))]
pub use oclive_agent_builtin::BuiltinReActAgent;

#[cfg(all(feature = "kernel-agent", not(feature = "default-agent-providers")))]
#[path = "agent_mcp_shell.rs"]
mod agent_mcp_shell;

#[cfg(all(feature = "kernel-agent", not(feature = "default-agent-providers")))]
pub use agent_mcp_shell::McpShellAgent;

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
