//! `process_message` / 双核编排共用错误类型。

use crate::domain::chat_engine::co_present;
use crate::error::AppError;
use thiserror::Error;

/// 主编排失败：按阶段标注，便于日志与排障。
#[derive(Debug, Error)]
pub enum ProcessMessageError {
    #[error("send_message[{stage}]: {source}")]
    Stage {
        stage: &'static str,
        #[source]
        source: AppError,
    },
    #[error(transparent)]
    CoPresent(#[from] co_present::CoPresentError),
}

impl ProcessMessageError {
    #[must_use]
    pub fn stage(stage: &'static str, source: AppError) -> Self {
        Self::Stage { stage, source }
    }
}

impl From<ProcessMessageError> for AppError {
    fn from(e: ProcessMessageError) -> Self {
        match e {
            ProcessMessageError::Stage { source, .. } => source,
            ProcessMessageError::CoPresent(c) => c.into(),
        }
    }
}
