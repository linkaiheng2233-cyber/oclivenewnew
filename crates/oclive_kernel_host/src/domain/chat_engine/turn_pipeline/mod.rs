//! Shared turn orchestration for co-present and remote-life paths.

mod co_present;
pub(crate) mod common;
mod remote_life;

use std::time::Instant;

use crate::models::dto::SendMessageResponse;

use super::turn_context::TurnContext;
use super::turn_error::TurnResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnMode {
    CoPresent,
    RemoteLife,
}

pub async fn execute_turn(ctx: &TurnContext<'_>, mode: TurnMode) -> TurnResult<SendMessageResponse> {
    let turn_lock = ctx.state.turn_lock_for(ctx.srid);
    let _turn_guard = turn_lock.lock().await;

    let path_label = match mode {
        TurnMode::CoPresent => "co_present",
        TurnMode::RemoteLife => "remote_life",
    };
    let t_path0 = Instant::now();

    let pre = common::pre_llm(ctx).await?;
    let middle = match mode {
        TurnMode::CoPresent => co_present::run_middle(ctx, &pre).await?,
        TurnMode::RemoteLife => remote_life::run_middle(ctx, &pre).await?,
    };

    let pre_main_llm_ms = t_path0.elapsed().as_millis() as u64;
    let llm = common::run_main_llm(ctx, path_label, &pre, &middle).await?;
    common::post_llm(ctx, mode, path_label, &pre, &middle, &llm, pre_main_llm_ms).await
}
