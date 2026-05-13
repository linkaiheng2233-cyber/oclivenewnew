use crate::error::Result;
use async_trait::async_trait;
pub use oclive_kernel_core::agent::{
    AgentDebugTrace, AgentInput, AgentOutput, AgentProvider, AgentToolCallTrace,
};

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

/// `plugin_backends.agent = none` 时的固定提示（中文产品文案；与 `MODULE_NONE_SEMANTICS.md` §7 对齐）。
pub const AGENT_BACKEND_NONE_REPLY: &str = "Agent 模块未启用（backend=none）。如需工具编排，请将 plugin_backends.agent 设为 builtin、remote 或 directory。";

/// `AgentBackend::None` 的进程内实现：不触发 MCP / HTTP，返回确定文案（非空、非用户输入回显）。
pub struct DisabledAgentProvider;

#[async_trait]
impl AgentProvider for DisabledAgentProvider {
    async fn process(&self, _: AgentInput) -> Result<AgentOutput> {
        Ok(AgentOutput {
            handled: false,
            reply: AGENT_BACKEND_NONE_REPLY.to_string(),
        })
    }
}
