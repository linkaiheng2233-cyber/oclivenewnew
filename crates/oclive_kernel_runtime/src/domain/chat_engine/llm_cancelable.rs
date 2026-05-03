//! 主对话 LLM 与用户「取消生成」协同（`KernelAppState::chat_generation_cancel`）。

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
