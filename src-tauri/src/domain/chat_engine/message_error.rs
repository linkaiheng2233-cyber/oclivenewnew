//! `process_message` / 双核编排共用错误类型。
//!
//! 统一 `send_message[{stage}]` 包装，避免各子模块重复构造 [`ProcessMessageError::Stage`]。

use crate::domain::chat_engine::co_present;
use crate::error::AppError;
use thiserror::Error;

/// `process_message` 主路径阶段名（与 tracing / OOCP 对齐；预留统一包装）。
#[allow(dead_code)]
pub const STAGE_SEND_MESSAGE: &str = "send_message";

/// 实验核 [`DualPipelineRunner`](crate::domain::dual_pipeline::DualPipelineRunner) 阶段名。
pub const STAGE_DUAL_CORE_EXPERIMENTAL: &str = "dual_core_experimental";

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

    /// 实验核步骤失败：无效 action / 未实现 method / 校验错误。
    #[must_use]
    pub fn dual_core_invalid(msg: impl Into<String>) -> Self {
        Self::stage(
            STAGE_DUAL_CORE_EXPERIMENTAL,
            AppError::InvalidParameter(msg.into()),
        )
    }

    /// 实验核步骤失败：槽位 / DB / 插件返回的 [`AppError`] 原样带上阶段前缀。
    #[must_use]
    pub fn dual_core(source: AppError) -> Self {
        Self::stage(STAGE_DUAL_CORE_EXPERIMENTAL, source)
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
