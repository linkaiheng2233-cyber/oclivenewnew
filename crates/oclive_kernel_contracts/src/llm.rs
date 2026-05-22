//! LLM 生成端口：编排层与策略只依赖此 trait，实现由宿主 `infrastructure` 提供。

use async_trait::async_trait;
use oclive_kernel_types::Result;

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn generate(&self, model: &str, prompt: &str) -> Result<String>;
    /// 低温度短输出（立绘标签等分类任务）
    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String>;
    /// 启动期可选探活（默认成功；实现可 ping 远端）。
    async fn startup_probe(&self) -> Result<()> {
        Ok(())
    }
}
