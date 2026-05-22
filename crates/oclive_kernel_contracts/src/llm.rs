//! LLM 生成端口：编排层与策略只依赖此 trait，实现由宿主 `infrastructure` 提供。

use async_trait::async_trait;
use oclive_kernel_types::Result;

/// Text generation port used by orchestration and policy (Ollama, remote, mock, etc.).
///
/// ## When to implement
///
/// - **谁**：LLM 后端集成方（Ollama 客户端、Remote HTTP LLM、测试 `MockLlmClient`）。
/// - **何时**：编排需要调用语言模型生成回复或 `generate_tag` 分类输出时。
///
/// ## When not to implement
///
/// - 仅使用宿主内置 Ollama / 已配置的 Remote，且不改推理路径时，**不必**自定义实现。
/// - 不做任何 LLM 调用的工具链（纯规则回复）可跳过。
///
/// # Examples
///
/// ```no_run
/// use oclive_kernel_contracts::LlmClient;
/// use std::sync::Arc;
///
/// async fn ask(llm: Arc<dyn LlmClient>) -> oclive_kernel_types::Result<()> {
///     let reply = llm.generate("qwen2.5:7b", "你好").await?;
///     assert!(!reply.is_empty());
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// 主对话生成（温度由实现默认）。
    ///
    /// # Errors
    ///
    /// 网络失败、上游 4xx/5xx、超时或响应体无法解析时返回 `Err`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    async fn generate(&self, model: &str, prompt: &str) -> Result<String>;

    /// 低温度短输出（立绘标签等分类任务）。
    ///
    /// # Errors
    ///
    /// 与 [`generate`](Self::generate) 相同；额外约束由实现保证（更低温度 / 更短输出）。
    ///
    /// # Panics
    ///
    /// 不 panic。
    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String>;

    /// Optional startup probe (default succeeds; hosts may ping remote LLM).
    ///
    /// # Errors
    ///
    /// 当探测请求失败且宿主配置为「启动必须可用」时返回 `Err`；默认实现恒为 `Ok(())`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    async fn startup_probe(&self) -> Result<()> {
        Ok(())
    }
}
