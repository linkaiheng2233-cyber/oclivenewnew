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
        e.source.with_chat_stage(e.stage)
    }
}

pub(crate) type TurnResult<T> = std::result::Result<T, TurnError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;

    #[test]
    fn turn_error_into_app_error_preserves_stage() {
        let err = TurnError::wrap(
            "bot_emotion",
            AppError::DatabaseError("connection lost".into()),
        );
        let app: AppError = err.into();
        let body = app.kernel_error_body();
        assert_eq!(body.code, "DB_ERROR");
        assert!(body.message.contains("send_message[bot_emotion]"));
        assert!(body.message.contains("connection lost"));
    }
}
