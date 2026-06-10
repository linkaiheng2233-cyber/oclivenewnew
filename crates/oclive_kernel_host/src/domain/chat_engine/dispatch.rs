//! Turn dispatch after [`TurnContext`] is built: remote stub / remote life / dual-core / co-present.

use crate::domain::chat_engine::chat_stage::ChatStage;
use crate::domain::chat_engine::message_error::ProcessMessageError;
use crate::domain::chat_engine::staged::stage_process_message;
use crate::domain::chat_engine::turn_context::TurnContext;
use crate::domain::chat_engine::turn_pipeline::{execute_turn, execute_turn_stream, TurnMode};
use crate::domain::chat_engine::{process_remote_life, process_remote_stub};
#[cfg(feature = "dual_core")]
use crate::domain::dual_pipeline::DualPipelineRunner;
use crate::models::dto::SendMessageResponse;
use crate::models::Role;

/// Whether dual-core is requested by blueprint but unavailable in this build.
pub(crate) fn resolve_dual_core_degraded(role: &Role) -> bool {
    #[cfg(not(feature = "dual_core"))]
    {
        let degraded = role.dual_core_gated();
        if degraded {
            tracing::warn!(
                target: "oclive_dual_core",
                role_id = %role.id,
                "dual_core feature disabled; role blueprint has dual_core enabled — co_present fallback"
            );
        }
        degraded
    }
    #[cfg(feature = "dual_core")]
    {
        let _ = role;
        false
    }
}

/// Routes one turn to remote stub, remote life, dual-core, or co-present.
pub(crate) async fn dispatch_turn(
    turn: &TurnContext<'_>,
    is_remote: bool,
    remote_life_enabled: bool,
) -> std::result::Result<SendMessageResponse, ProcessMessageError> {
    if is_remote {
        if !remote_life_enabled {
            return stage_process_message(ChatStage::RemoteStub, process_remote_stub(turn).await);
        }
        return stage_process_message(ChatStage::RemoteLife, process_remote_life(turn).await);
    }

    #[cfg(feature = "dual_core")]
    if turn.role.dual_core_gated() {
        return DualPipelineRunner::run_with_fallback(turn).await;
    }

    Ok(execute_turn(turn, TurnMode::CoPresent).await?)
}

/// Streaming variant: co-present path streams LLM tokens; other branches emit the full reply once.
pub(crate) async fn dispatch_turn_stream(
    turn: &TurnContext<'_>,
    is_remote: bool,
    remote_life_enabled: bool,
    on_token: oclive_kernel_contracts::LlmTokenSink,
) -> std::result::Result<SendMessageResponse, ProcessMessageError> {
    if is_remote {
        let res = if !remote_life_enabled {
            stage_process_message(ChatStage::RemoteStub, process_remote_stub(turn).await)
        } else {
            stage_process_message(ChatStage::RemoteLife, process_remote_life(turn).await)
        }?;
        on_token(res.reply.as_str());
        return Ok(res);
    }

    #[cfg(feature = "dual_core")]
    if turn.role.dual_core_gated() {
        let res = DualPipelineRunner::run_with_fallback(turn).await?;
        on_token(res.reply.as_str());
        return Ok(res);
    }

    Ok(execute_turn_stream(turn, TurnMode::CoPresent, on_token).await?)
}
