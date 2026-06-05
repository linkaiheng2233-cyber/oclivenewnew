//! Shared turn orchestration for co-present and remote-life paths.

mod co_present;
pub(crate) mod persistence;
mod post;
mod pre;
mod remote_life;

use std::time::Instant;

use crate::models::dto::SendMessageResponse;

use super::turn_context::TurnContext;
use super::turn_error::TurnResult;

pub(crate) use pre::{
    build_complex_emotion_turn_input, compute_turn_favor, skipped_complex_emotion,
    worldview_snippet_from_chunks, MiddleOutput, PreLlmOutput, STAGES,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnMode {
    CoPresent,
    RemoteLife,
}

pub async fn execute_turn(ctx: &TurnContext<'_>, mode: TurnMode) -> TurnResult<SendMessageResponse> {
    let path_label = match mode {
        TurnMode::CoPresent => "co_present",
        TurnMode::RemoteLife => "remote_life",
    };
    let t_path0 = Instant::now();

    let pre = pre::pre_llm(ctx).await?;
    let middle = match mode {
        TurnMode::CoPresent => co_present::run_middle(ctx, &pre).await?,
        TurnMode::RemoteLife => remote_life::run_middle(ctx, &pre).await?,
    };

    let pre_main_llm_ms = t_path0.elapsed().as_millis() as u64;
    let llm = post::run_main_llm(ctx, path_label, &pre, &middle).await?;
    post::post_llm(ctx, mode, path_label, &pre, &middle, &llm, pre_main_llm_ms).await
}
