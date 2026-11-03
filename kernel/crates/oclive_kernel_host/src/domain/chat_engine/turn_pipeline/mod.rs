//! Shared turn orchestration for co-present and remote-life paths.

mod co_present;
pub(crate) mod persistence;
mod post;
mod pre;
mod remote_life;
mod stopped;

use std::time::Instant;

use crate::models::dto::SendMessageResponse;

use super::turn_context::TurnContext;
use super::turn_error::TurnResult;

pub(crate) use pre::{
    build_complex_emotion_turn_input, compute_turn_favor, latest_recent_turn_pair,
    skipped_complex_emotion, worldview_snippet_from_chunks, MiddleOutput, PreLlmOutput, STAGES,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnMode {
    CoPresent,
    RemoteLife,
}

pub async fn execute_turn(
    ctx: &TurnContext<'_>,
    mode: TurnMode,
) -> TurnResult<SendMessageResponse> {
    execute_cancellable(ctx, mode, None).await
}

pub async fn execute_turn_stream(
    ctx: &TurnContext<'_>,
    mode: TurnMode,
    on_token: oclive_kernel_contracts::LlmTokenSink,
) -> TurnResult<SendMessageResponse> {
    execute_cancellable(ctx, mode, Some(on_token)).await
}

async fn execute_cancellable(
    ctx: &TurnContext<'_>,
    mode: TurnMode,
    on_token: Option<oclive_kernel_contracts::LlmTokenSink>,
) -> TurnResult<SendMessageResponse> {
    let path_label = match mode {
        TurnMode::CoPresent => "co_present",
        TurnMode::RemoteLife => "remote_life",
    };
    let t_path0 = Instant::now();
    let registration = ctx
        .state
        .chat_cancellations
        .register(ctx.req)
        .map_err(|e| super::turn_error::TurnError::wrap("chat_cancellation", e))?;
    let control = &registration.control;
    let generation = async {
        let pre = pre::pre_llm(ctx).await?;
        let middle = match mode {
            TurnMode::CoPresent => co_present::run_middle(ctx, &pre).await?,
            TurnMode::RemoteLife => remote_life::run_middle(ctx, &pre).await?,
        };
        let pre_main_llm_ms = t_path0.elapsed().as_millis() as u64;
        let llm = match on_token {
            Some(sink) => {
                post::run_main_llm_stream(ctx, path_label, &pre, &middle, control.capture(sink))
                    .await?
            }
            None => post::run_main_llm(ctx, path_label, &pre, &middle).await?,
        };
        Ok::<_, super::turn_error::TurnError>((pre, middle, llm, pre_main_llm_ms))
    };
    // Dropping this future closes the in-flight provider request and releases its
    // permits. Never drop post_llm: allow its state/history writes to finish.
    let generated = tokio::select! {
        biased;
        () = control.cancelled() => None,
        outcome = generation => Some(outcome),
    };
    let Some(generated) = generated else {
        return stopped::persist_stopped(ctx, mode, &control.visible()).await;
    };
    if !control.begin_commit() {
        return stopped::persist_stopped(ctx, mode, &control.visible()).await;
    }
    let (pre, middle, llm, pre_main_llm_ms) = generated?;
    post::post_llm(ctx, mode, path_label, &pre, &middle, &llm, pre_main_llm_ms).await
}
