//! Stage-aware await helpers for chat orchestration (replaces repetitive `kernel_stage!`).
#![allow(clippy::missing_errors_doc, dead_code)]

use std::future::Future;

use crate::domain::chat_engine::chat_stage::ChatStage;
use crate::domain::chat_engine::co_present::{CoPresentError, CoPresentResult};
use crate::domain::chat_engine::message_error::ProcessMessageError;
use crate::error::Result;

/// Attach a [`ChatStage`] label to an async co-present step.
pub async fn co_present_stage<T, Fut>(stage: ChatStage, fut: Fut) -> CoPresentResult<T>
where
    Fut: Future<Output = Result<T>>,
{
    fut.await
        .map_err(|source| CoPresentError::wrap(stage.as_str(), source))
}

/// Attach a [`ChatStage`] label to an async `process_message` step.
pub async fn process_message_stage<T, Fut>(
    stage: ChatStage,
    fut: Fut,
) -> std::result::Result<T, ProcessMessageError>
where
    Fut: Future<Output = Result<T>>,
{
    fut.await.map_err(|source| ProcessMessageError::Stage {
        stage: stage.as_str(),
        source,
    })
}

/// Sync stage wrapper for `process_message` steps that return [`AppError`].
pub fn stage_process_message<T>(
    stage: ChatStage,
    result: Result<T>,
) -> std::result::Result<T, ProcessMessageError> {
    result.map_err(|source| ProcessMessageError::Stage {
        stage: stage.as_str(),
        source,
    })
}

/// Sync co-present stage wrapper.
pub fn stage_co_present<T>(stage: ChatStage, result: Result<T>) -> CoPresentResult<T> {
    result.map_err(|source| CoPresentError::wrap(stage.as_str(), source))
}
