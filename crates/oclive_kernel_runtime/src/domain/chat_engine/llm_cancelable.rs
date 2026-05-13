//! 主对话 LLM 与用户「取消生成」协同（`KernelAppState::chat_generation_cancel`）。
//!
//! **取消语义**：`select!` 在取消分支返回后，对仍在运行的 `work` 调用 `abort` 并 `await` 其 `JoinHandle`，避免泄漏 detached 任务；正常完成路径由 `join` 消费 handle，不再 `abort`。
//! **不变式**：`LlmClient::generate` 须容忍取消（底层 HTTP 客户端在 drop 时中断）；不要在持有 `state` 短锁时调用本函数（本模块不持锁）。

use crate::error::{AppError, Result};
use crate::infrastructure::llm::LlmClient;
use crate::state::KernelAppState;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

/// 在独立任务中执行 `LlmClient::generate`，与取消标志竞争；取消时 `abort` 生成任务以尽快释放 HTTP。
pub async fn run_llm_generate_cancelable(
    state: &KernelAppState,
    llm: Arc<dyn LlmClient>,
    model: &str,
    prompt: &str,
) -> Result<String> {
    let model_owned = model.to_string();
    let prompt_owned = prompt.to_string();
    let flag = Arc::clone(&state.chat_generation_cancel);
    let llm_c = Arc::clone(&llm);
    let mut work = tokio::spawn(async move {
        llm_c
            .generate(model_owned.as_str(), prompt_owned.as_str())
            .await
    });

    tokio::select! {
        r = &mut work => {
            return match r {
                Ok(Ok(s)) => Ok(s),
                Ok(Err(e)) => Err(e),
                Err(j) if j.is_cancelled() => Err(AppError::ChatGenerationCancelled),
                Err(j) => Err(AppError::InvalidParameter(format!(
                    "[LLM_TASK_JOIN] {}",
                    j
                ))),
            };
        }
        _ = async {
            loop {
                if flag.load(Ordering::Acquire) {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        } => {}
    }

    work.abort();
    let _ = work.await;
    Err(AppError::ChatGenerationCancelled)
}
