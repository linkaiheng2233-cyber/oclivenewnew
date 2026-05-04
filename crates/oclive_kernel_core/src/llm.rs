//! 主对话与标签任务的 LLM 调用抽象（无具体 HTTP 实现）。

use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn generate(&self, model: &str, prompt: &str) -> Result<String>;
    /// 低温度短输出（立绘标签等分类任务）
    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String>;
}
