//! Stage-aware await helpers for chat orchestration (replaces repetitive `kernel_stage!`).
#![allow(clippy::missing_errors_doc, dead_code)]

use std::future::Future;

use crate::domain::chat_engine::chat_stage::ChatStage;
use crate::domain::chat_engine::message_error::ProcessMessageError;
use crate::domain::chat_engine::turn_error::{TurnError, TurnResult};
use crate::error::Result;

/// Turn orchestration stage runner (replaces `kernel_stage!(@co_present …)` at call sites).
pub struct StageRunner;

impl StageRunner {
    pub async fn stage<T, Fut>(&self, stage: ChatStage, fut: Fut) -> TurnResult<T>
    where
        Fut: Future<Output = Result<T>>,
    {
        turn_stage(stage, fut).await
    }
}

/// Attach a [`ChatStage`] label to an async turn step.
pub async fn turn_stage<T, Fut>(stage: ChatStage, fut: Fut) -> TurnResult<T>
where
    Fut: Future<Output = Result<T>>,
{
    fut.await
        .map_err(|source| TurnError::wrap(stage.as_str(), source))
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

/// Sync turn stage wrapper.
pub fn stage_turn<T>(stage: ChatStage, result: Result<T>) -> TurnResult<T> {
    result.map_err(|source| TurnError::wrap(stage.as_str(), source))
}
