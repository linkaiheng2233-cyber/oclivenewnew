//! Turn orchestration errors (co-present, remote-life, dual-core fallback).

use crate::error::AppError;
use thiserror::Error;

/// Turn orchestration failure with `stage` for log alignment.
#[derive(Debug, Error)]
#[error("回合({stage}): {source}")]
pub struct TurnError {
    pub(crate) stage: &'static str,
    #[source]
    pub(crate) source: AppError,
}

impl TurnError {
    pub fn wrap(stage: &'static str, source: AppError) -> Self {
        Self { stage, source }
    }
}

impl From<TurnError> for AppError {
    fn from(e: TurnError) -> Self {
        e.source
    }
}

pub(crate) type TurnResult<T> = std::result::Result<T, TurnError>;
