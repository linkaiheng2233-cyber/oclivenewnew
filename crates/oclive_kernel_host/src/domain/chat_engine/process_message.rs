//! # Main message processing entry
//!
//! **Role**: orchestration entry for a **single user message** from Tauri / HTTP API; Agent shortcut, remote-life, and other branches fan out here into [`turn_pipeline`](super::turn_pipeline) ([`TurnMode::CoPresent`](super::turn_pipeline::TurnMode::CoPresent), etc.).
//!
//! **Upstream**: `api` / `http_api` load `Role`, `plugin_backends`, and session-level `slot_registry` overrides via `AppState`.
//! **Downstream**: enters the turn pipeline via [`turn_pipeline::execute_turn`](super::turn_pipeline::execute_turn) / `process_remote_*`; invokes plugins via [`PluginHostPort`](crate::domain::ports::PluginHostPort); **does not** use `pipeline.ocblueprint` DSL for first-turn scheduling.
//!
//! **Architecture**: main path is **Rust orchestration** (`turn_pipeline` + [`TurnMode::CoPresent`](super::turn_pipeline::TurnMode::CoPresent) + [`SlotRunner`](../slot_runner.rs)); slot resolution depends on `slot_registry` / `groups` and the `PluginHost` registry.
//!
//! See [`domain/README.md`](../README.md).

use crate::domain::agent::{AgentInput, AgentOutput};
use crate::domain::chat_engine::chat_stage::ChatStage;
use crate::domain::chat_engine::dispatch::{dispatch_turn, resolve_dual_core_degraded};
use crate::domain::chat_engine::message_error::ProcessMessageError;
use crate::domain::chat_engine::minimal_response::build_minimal_response;
use crate::domain::chat_engine::presence::user_is_remote_from_character;
use crate::domain::chat_engine::staged::{process_message_stage, stage_process_message};
use crate::domain::chat_engine::turn_context::TurnContext;
use crate::domain::chat_engine::{
    backend_resolution_summary, context::validate_scene_id, conversation_state_role_id,
    ensure_role_loaded,
};
use crate::domain::startup_health;
use crate::error::Result;
use crate::models::dto::{SendMessageRequest, SendMessageResponse};
use crate::state::AppState;
use std::sync::Arc;
use std::time::Instant;

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn process_message(
    state: &AppState,
    req: &SendMessageRequest,
) -> Result<SendMessageResponse> {
    match run(state, req).await {
        Ok(v) => Ok(v),
        Err(e) => {
            tracing::error!(target: "oclive_chat", "{}", e);
            Err(e.into())
        }
    }
}

async fn run(
    state: &AppState,
    req: &SendMessageRequest,
) -> std::result::Result<SendMessageResponse, ProcessMessageError> {
    let mrid = req.role_id.as_str();
    let state_rid = conversation_state_role_id(mrid, req.session_id.as_deref());
    let srid = state_rid.as_str();
    let requested_scene_id = req
        .scene_id
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let t0 = Instant::now();

    let (_, role) = tokio::try_join!(
        async {
            stage_process_message(
                ChatStage::EnsureRoleRuntime,
                state.db_manager.ensure_role_runtime(srid).await,
            )
        },
        async {
            ensure_role_loaded(state, mrid)
                .await
                .map_err(|source| ProcessMessageError::Stage {
                    stage: ChatStage::EnsureRoleLoaded.as_str(),
                    source,
                })
        },
    )?;

    let scene_id = validate_scene_id(mrid, &role.scene_ids, requested_scene_id);
    let turn_lock = state.turn_lock_for(srid);
    let _turn_guard = turn_lock.lock().await;
    tracing::debug!(
        target: "oclive_chat",
        role_id = %mrid,
        scene_id = %scene_id,
        session_ns = %srid,
        "send_message start",
    );

    stage_process_message(
        ChatStage::EnsureInteractionModeSeeded,
        state
            .db_manager
            .ensure_interaction_mode_seeded(srid, role.interaction_mode.as_deref())
            .await,
    )?;

    stage_process_message(
        ChatStage::ApplyUserLlmEnv,
        crate::domain::user_llm_env::apply_user_llm_env(state).await,
    )?;

    let effective_backends = state.effective_plugin_backends_for_session(role.as_ref(), srid);
    let effective_sources = state.effective_plugin_backend_sources_for_session(role.as_ref(), srid);
    tracing::debug!(
        target: "oclive_chat",
        role_id = %mrid,
        scene_id = %scene_id,
        session_ns = %srid,
        backends = %backend_resolution_summary(&effective_backends, &effective_sources),
        "send_message backends",
    );

    startup_health::ensure_once(state, role.as_ref(), &effective_backends)
        .await
        .map_err(|source| ProcessMessageError::Stage {
            stage: ChatStage::StartupHealth.as_str(),
            source,
        })?;

    let pl = crate::domain::chat_engine::plugin_resolve::resolve_plugins_for_session(
        state.plugin_host_port(),
        role.as_ref(),
        Some(srid),
        &effective_backends,
        state
            .effective_slot_registry_for_session(role.as_ref(), srid)
            .as_ref(),
    );
    let agent_out: AgentOutput = if state.host_profile.skip_agent {
        AgentOutput {
            handled: false,
            reply: String::new(),
        }
    } else {
        process_message_stage(
            ChatStage::AgentProcess,
            pl.agent.process(AgentInput {
                role_id: mrid.to_string(),
                session_namespace: srid.to_string(),
                message: req.user_message.clone(),
                model: role.resolve_ollama_model(state.ollama_model.as_str()),
            }),
        )
        .await?
    };
    if agent_out.handled {
        return build_minimal_response(
            state,
            &pl,
            role.as_ref(),
            mrid,
            srid,
            scene_id.clone(),
            req.user_message.as_str(),
            agent_out.reply,
        )
        .await
        .map_err(|source| ProcessMessageError::Stage {
            stage: ChatStage::AgentMinimalResponse.as_str(),
            source,
        });
    }

    let (_, current_scene, interaction_mode, remote_life_enabled) = tokio::try_join!(
        process_message_stage(
            ChatStage::SetUserPresenceScene,
            state
                .db_manager
                .set_user_presence_scene(srid, scene_id.as_str()),
        ),
        process_message_stage(
            ChatStage::GetCurrentScene,
            state.db_manager.get_current_scene(srid),
        ),
        process_message_stage(
            ChatStage::GetInteractionMode,
            state.db_manager.get_interaction_mode(srid),
        ),
        process_message_stage(
            ChatStage::GetRemoteLifeEnabled,
            state.db_manager.get_remote_life_enabled(srid),
        ),
    )?;
    let immersive = interaction_mode.is_immersive();
    if immersive {
        process_message_stage(
            ChatStage::IdlePersonalityDecay,
            crate::domain::virtual_time_sync::apply_idle_personality_decay(
                state,
                role.as_ref(),
                srid,
            ),
        )
        .await?;
    }
    let is_remote =
        immersive && user_is_remote_from_character(scene_id.as_str(), current_scene.as_deref());
    let preflight_ms = t0.elapsed().as_millis() as u64;
    let char_scene = current_scene.as_deref().unwrap_or("default").to_string();
    let virtual_time_ms = process_message_stage(
        ChatStage::VirtualTimeMs,
        crate::domain::virtual_time_sync::sync_and_persist_virtual_time(
            state.db_manager.as_ref(),
            role.as_ref(),
            srid,
            immersive,
        ),
    )
    .await?;
    let scenes = Arc::clone(&role.scene_ids);
    let dual_core_degraded = resolve_dual_core_degraded(role.as_ref());
    let turn = TurnContext {
        state,
        req,
        role: role.as_ref(),
        scene_id: scene_id.as_str(),
        scenes,
        mrid,
        srid,
        t0,
        preflight_ms,
        effective_backends,
        pl: pl.clone(),
        immersive,
        character_scene_id: if is_remote { Some(char_scene) } else { None },
        virtual_time_ms,
        dual_core_degraded,
    };
    dispatch_turn(&turn, is_remote, remote_life_enabled).await
}
