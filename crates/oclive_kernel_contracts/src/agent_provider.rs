//! Agent 调度可替换门面 trait。

use async_trait::async_trait;
use oclive_kernel_types::{AgentInput, AgentOutput, Result};

#[async_trait]
pub trait AgentProvider: Send + Sync {
    async fn process(&self, input: AgentInput) -> Result<AgentOutput>;
}
