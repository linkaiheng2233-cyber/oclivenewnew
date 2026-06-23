//! Stage-aware await helpers for chat orchestration (replaces repetitive `kernel_stage!`).
#![allow(clippy::missing_errors_doc, dead_code)]

use std::future::Future;
use std::time::Instant;

use tracing::Instrument;

use crate::domain::chat_engine::chat_stage::ChatStage;
use crate::domain::chat_engine::message_error::ProcessMessageError;
use crate::domain::chat_engine::turn_error::{TurnError, TurnResult};
use crate::error::Result;

const TURN_TARGET: &str = "oclive_turn";

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

fn log_stage_elapsed(stage: &str, start: Instant) {
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    tracing::debug!(
        target: TURN_TARGET,
        stage = %stage,
        elapsed_ms,
        "chat stage completed"
    );
}

/// Attach a [`ChatStage`] label to an async turn step.
pub async fn turn_stage<T, Fut>(stage: ChatStage, fut: Fut) -> TurnResult<T>
where
    Fut: Future<Output = Result<T>>,
{
    let stage_name = stage.as_str();
    let start = Instant::now();
    let span = tracing::info_span!(target: TURN_TARGET, "turn_stage", stage = stage_name);
    let result = fut
        .instrument(span)
        .await
        .map_err(|source| TurnError::wrap(stage_name, source));
    log_stage_elapsed(stage_name, start);
    result
}

/// Attach a [`ChatStage`] label to an async `process_message` step.
pub async fn process_message_stage<T, Fut>(
    stage: ChatStage,
    fut: Fut,
) -> std::result::Result<T, ProcessMessageError>
where
    Fut: Future<Output = Result<T>>,
{
    let stage_name = stage.as_str();
    let start = Instant::now();
    let span = tracing::info_span!(target: TURN_TARGET, "turn_stage", stage = stage_name);
    let result = fut
        .instrument(span)
        .await
        .map_err(|source| ProcessMessageError::Stage {
            stage: stage_name,
            source,
        });
    log_stage_elapsed(stage_name, start);
    result
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
