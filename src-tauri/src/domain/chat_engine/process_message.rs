//! # ???????????????
//!
//! **??**?Tauri / HTTP ??????????**???**???????????????Agent ?????/????????????? [`co_present`](super::co_present)?
//!
//! **??**?`api` / `http_api` ? `AppState`??? `Role`?`plugin_backends`???? `slot_registry` ???
//! **??**?[`co_present::process_co_present`](super::co_present)?`process_remote_*`?????? [`PluginHostPort`](crate::domain::ports::PluginHostPort) ???**????????????**?
//!
//! **????**?????? **Rust ??**?`co_present` + [`SlotRunner`](../slot_runner.rs)????**??** `pipeline.ocblueprint` ???????????? `slot_registry` / `groups` ?????????????????????
//!
//! ??????? [`domain/README.md`](../README.md)?

use crate::domain::agent::AgentInput;
use crate::domain::chat_engine::chat_stage::ChatStage;
use crate::domain::chat_engine::co_present;
use crate::domain::chat_engine::message_error::ProcessMessageError;
use crate::domain::chat_engine::minimal_response::build_minimal_response;
use crate::domain::chat_engine::presence::user_is_remote_from_character;
use crate::domain::chat_engine::turn_context::TurnContext;
use crate::domain::chat_engine::{
    backend_resolution_summary, context::validate_scene_id, conversation_state_role_id,
    ensure_role_loaded, process_remote_life, process_remote_stub,
};
use crate::domain::dual_pipeline::DualPipelineRunner;
use crate::domain::startup_health;
use crate::error::Result;
use crate::models::dto::{SendMessageRequest, SendMessageResponse};
use crate::state::AppState;
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
    let (scene_id, scenes) = crate::kernel_stage!(
        @process_message ChatStage::ValidateSceneId,
        validate_scene_id(state, mrid, requested_scene_id)
    )?;
    let t0 = Instant::now();
    tracing::debug!(
        target: "oclive_chat",
        "send_message start role_id={} scene_id={} session_ns={}",
        mrid,
        scene_id,
        srid
    );

    crate::kernel_stage!(
        @process_message ChatStage::EnsureRoleRuntime,
        state.db_manager.ensure_role_runtime(srid).await
    )?;

    let role = crate::kernel_stage!(
        @process_message ChatStage::EnsureRoleLoaded,
        ensure_role_loaded(state, mrid).await
    )?;
    crate::kernel_stage!(
        @process_message ChatStage::EnsureInteractionModeSeeded,
        state
            .db_manager
            .ensure_interaction_mode_seeded(srid, role.interaction_mode.as_deref())
            .await
    )?;
    let effective_backends = state.effective_plugin_backends_for_session(role.as_ref(), srid);
    let effective_sources =
        state.effective_plugin_backend_sources_for_session(role.as_ref(), srid);
    tracing::debug!(
        target: "oclive_chat",
        "send_message backends role_id={} scene_id={} session_ns={} {}",
        mrid,
        scene_id,
        srid,
        backend_resolution_summary(&effective_backends, &effective_sources)
    );

    crate::kernel_stage!(
        @process_message ChatStage::StartupHealth,
        startup_health::ensure_once(state, role.as_ref(), &effective_backends).await
    )?;

    let pl = crate::domain::chat_engine::plugin_resolve::resolve_plugins_for_session(
        state.plugin_host_port(),
        role.as_ref(),
        Some(srid),
        &effective_backends,
        state
            .effective_slot_registry_for_session(role.as_ref(), srid)
            .as_ref(),
    );
    let agent_out = crate::kernel_stage!(
        @process_message ChatStage::AgentProcess,
        pl.agent
            .process(AgentInput {
                role_id: mrid.to_string(),
                session_namespace: srid.to_string(),
                message: req.user_message.clone(),
                model: role.resolve_ollama_model(state.ollama_model.as_str()),
            })
            .await
    )?;
    if agent_out.handled {
        return build_minimal_response(
            state,
            &pl,
            role.as_ref(),
            srid,
            scene_id,
            req.user_message.as_str(),
            agent_out.reply,
        )
        .await
        .map_err(|source| ProcessMessageError::Stage {
            stage: ChatStage::AgentMinimalResponse.as_str(),
            source,
        });
    }

    crate::kernel_stage!(
        @process_message ChatStage::SetUserPresenceScene,
        state
            .db_manager
            .set_user_presence_scene(srid, scene_id.as_str())
            .await
    )?;

    let current_scene = crate::kernel_stage!(
        @process_message ChatStage::GetCurrentScene,
        state.db_manager.get_current_scene(srid).await
    )?;
    let immersive = crate::kernel_stage!(
        @process_message ChatStage::GetInteractionMode,
        state.db_manager.get_interaction_mode(srid).await
    )?
    .is_immersive();
    let remote_life_enabled = crate::kernel_stage!(
        @process_message ChatStage::GetRemoteLifeEnabled,
        state.db_manager.get_remote_life_enabled(srid).await
    )?;
    let is_remote =
        immersive && user_is_remote_from_character(scene_id.as_str(), current_scene.as_deref());
    let preflight_ms = t0.elapsed().as_millis() as u64;
    let char_scene = current_scene
        .as_deref()
        .unwrap_or("default")
        .to_string();
    let turn = TurnContext {
        state,
        req,
        role: role.as_ref(),
        scene_id: scene_id.clone(),
        scenes,
        mrid,
        srid,
        t0,
        preflight_ms,
        effective_backends,
        immersive,
        character_scene_id: if is_remote {
            Some(char_scene)
        } else {
            None
        },
    };
    if is_remote {
        if !remote_life_enabled {
            return Ok(crate::kernel_stage!(
                @process_message ChatStage::RemoteStub,
                process_remote_stub(&turn).await
            )?);
        }
        return Ok(crate::kernel_stage!(
            @process_message ChatStage::RemoteLife,
            process_remote_life(&turn).await
        )?);
    }

    if role.dual_core_gated() {
        return DualPipelineRunner::run_with_fallback(&turn).await;
    }

    Ok(co_present::process_co_present(&turn).await?)
}
