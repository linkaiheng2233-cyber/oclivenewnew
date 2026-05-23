//! # 共景模式编排（同屏对话主路径）
//!
//! **角色**：用户与角色**同屏**时的回合编排——委托 [`super::turn_pipeline`] 执行共享主流程。
//!
//! **上游**：[`process_message`](super::process_message) 在排除 Agent 短路与异地分支后调用本模块。

use crate::models::dto::SendMessageResponse;
use crate::error::AppError;
use thiserror::Error;

use super::turn_context::TurnContext;
use super::turn_pipeline::{execute_turn, TurnMode};

/// 共景路径中的失败，带 `stage` 便于与日志对齐。
#[derive(Debug, Error)]
#[error("共景({stage}): {source}")]
pub struct CoPresentError {
    pub(crate) stage: &'static str,
    #[source]
    pub(crate) source: AppError,
}

impl CoPresentError {
    pub fn wrap(stage: &'static str, source: AppError) -> Self {
        Self { stage, source }
    }
}

impl From<CoPresentError> for AppError {
    fn from(e: CoPresentError) -> Self {
        e.source
    }
}

pub(crate) type CoPresentResult<T> = std::result::Result<T, CoPresentError>;

pub(crate) async fn process_co_present(
    ctx: &TurnContext<'_>,
) -> CoPresentResult<SendMessageResponse> {
    execute_turn(ctx, TurnMode::CoPresent).await
}
