//! Shared error types for `process_message` / dual-core orchestration.
//!
//! Unifies `send_message[{stage}]` wrapping so submodules do not duplicate [`ProcessMessageError::Stage`].

use crate::domain::chat_engine::turn_error::TurnError;
use crate::error::AppError;
use thiserror::Error;

/// Stage name for experimental core [`DualPipelineRunner`](crate::domain::dual_pipeline::DualPipelineRunner).
pub const STAGE_DUAL_CORE_EXPERIMENTAL: &str = "dual_core_experimental";

/// Main orchestration failure: tagged by stage for logging and troubleshooting.
#[derive(Debug, Error)]
pub enum ProcessMessageError {
    #[error("send_message[{stage}]: {source}")]
    Stage {
        stage: &'static str,
        #[source]
        source: AppError,
    },
    #[error(transparent)]
    Turn(#[from] TurnError),
}

impl ProcessMessageError {
    #[must_use]
    pub fn stage(stage: &'static str, source: AppError) -> Self {
        Self::Stage { stage, source }
    }

    /// Experimental step failure: invalid action / unimplemented method / validation error.
    #[must_use]
    pub fn dual_core_invalid(msg: impl Into<String>) -> Self {
        Self::stage(
            STAGE_DUAL_CORE_EXPERIMENTAL,
            AppError::InvalidParameter(msg.into()),
        )
    }

    /// Experimental step failure: slot / DB / plugin [`AppError`] with stage prefix attached.
    #[must_use]
    pub fn dual_core(source: AppError) -> Self {
        Self::stage(STAGE_DUAL_CORE_EXPERIMENTAL, source)
    }
}

impl From<ProcessMessageError> for AppError {
    fn from(e: ProcessMessageError) -> Self {
        match e {
            ProcessMessageError::Stage { source, .. } => source,
            ProcessMessageError::Turn(c) => c.into(),
        }
    }
}
