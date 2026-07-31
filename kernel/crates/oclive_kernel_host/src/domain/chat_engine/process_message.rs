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

use crate::domain::agent::AgentOutput;
use crate::domain::agent_context::build_agent_input;
use crate::domain::chat_engine::chat_stage::ChatStage;
use crate::domain::chat_engine::dispatch::{
    dispatch_turn, dispatch_turn_stream, resolve_dual_core_degraded,
};
use crate::domain::chat_engine::message_error::ProcessMessageError;
use crate::domain::chat_engine::minimal_response::build_minimal_response;
use crate::domain::chat_engine::presence::user_is_remote_from_character;
use crate::domain::chat_engine::staged::{process_message_stage, stage_process_message};
use crate::domain::chat_engine::turn_context::TurnContext;
use crate::domain::chat_engine::turn_prefetch::build_turn_prefetch;
use crate::domain::chat_engine::{
    backend_resolution_summary, context::validate_scene_id, conversation_state_role_id,
    ensure_role_loaded,
};
use crate::domain::startup_health;
use crate::error::Result;
use crate::models::dto::{SendMessageRequest, SendMessageResponse};
use crate::models::plugin_backends::AgentBackend;
use crate::state::AppState;
use crate::state::EffectiveSessionConfig;
use oclive_kernel_contracts::LlmTokenSink;
use std::sync::Arc;
use std::time::Instant;

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn process_message(
    state: &AppState,
    req: &SendMessageRequest,
) -> Result<SendMessageResponse> {
    match run(state, req, None).await {
        Ok(v) => Ok(v),
        Err(e) => {
            tracing::error!(target: "oclive_chat", "{}", e);
            Err(e.into())
        }
    }
}

/// Streaming variant: invokes `on_token` during main LLM generation; post-LLM side effects run after the stream completes.
///
/// # Errors
///
/// Same failure modes as [`process_message`] (health, turn pipeline, LLM stream).
pub async fn process_message_stream(
    state: &AppState,
    req: &SendMessageRequest,
    on_token: LlmTokenSink,
) -> Result<SendMessageResponse> {
    match run(state, req, Some(on_token)).await {
        Ok(v) => Ok(v),
        Err(e) => {
            tracing::error!(target: "oclive_chat", "{}", e);
            Err(e.into())
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn try_agent_shortcut(
    state: &AppState,
    req: &SendMessageRequest,
    role: &crate::models::Role,
    srid: &str,
    scene_id: &str,
    mrid: &str,
    effective_backends: &crate::models::plugin_backends::PluginBackends,
    pl: &crate::domain::plugin_host::ResolvedRolePlugins,
    prefetch: &crate::domain::chat_engine::turn_prefetch::TurnPrefetch,
    on_token: Option<&LlmTokenSink>,
) -> std::result::Result<Option<SendMessageResponse>, ProcessMessageError> {
    let agent_enabled =
        !state.host_profile.skip_agent && !matches!(effective_backends.agent, AgentBackend::None);
    let agent_out: AgentOutput = if agent_enabled {
        let model = role.resolve_ollama_model(state.global_ollama_model().as_str());
        let agent_input = build_agent_input(
            state,
            role,
            srid,
            scene_id,
            req.user_message.as_str(),
            model.as_str(),
            state.plugins.agent_mcp_bridge().as_ref(),
            Some(prefetch),
        )
        .await
        .map_err(|source| ProcessMessageError::Stage {
            stage: ChatStage::AgentProcess.as_str(),
            source,
        })?;
        process_message_stage(ChatStage::AgentProcess, pl.agent.process(agent_input)).await?
    } else {
        AgentOutput {
            handled: false,
            reply: String::new(),
        }
    };
    if agent_out.handled {
        if let Some(sink) = on_token {
            sink(agent_out.reply.as_str());
        }
        return build_minimal_response(
            state,
            pl,
            role,
            mrid,
            srid,
            scene_id.to_string(),
            req.user_message.as_str(),
            agent_out.reply,
        )
        .await
        .map(Some)
        .map_err(|source| ProcessMessageError::Stage {
            stage: ChatStage::AgentMinimalResponse.as_str(),
            source,
        });
    }
    Ok(None)
}

async fn load_turn_runtime_snapshot(
    state: &AppState,
    srid: &str,
    scene_id: &str,
) -> std::result::Result<
    crate::domain::role_runtime_snapshot::RoleRuntimeSnapshot,
    ProcessMessageError,
> {
    let seed_interaction_mode = !state.session_cache.is_interaction_mode_seeded(srid);
    let runtime_snapshot = process_message_stage(
        ChatStage::GetRoleRuntimeSnapshot,
        state
            .db_manager
            .preflight_turn_runtime(srid, scene_id, seed_interaction_mode),
    )
    .await?;
    if seed_interaction_mode {
        state.session_cache.mark_interaction_mode_seeded(srid);
    }
    Ok(runtime_snapshot)
}

struct ImmersiveVirtualTimeState {
    remote_life_enabled: bool,
    immersive: bool,
    is_remote: bool,
    character_scene_id: Option<String>,
    preflight_ms: u64,
    virtual_time_ms: i64,
}

async fn apply_immersive_virtual_time(
    state: &AppState,
    role: &crate::models::Role,
    srid: &str,
    scene_id: &str,
    runtime_snapshot: &crate::domain::role_runtime_snapshot::RoleRuntimeSnapshot,
    preflight_started_at: Instant,
    staged: bool,
) -> std::result::Result<ImmersiveVirtualTimeState, ProcessMessageError> {
    let current_scene = runtime_snapshot.scene.clone();
    let interaction_mode = runtime_snapshot
        .interaction_mode
        .unwrap_or(crate::models::InteractionMode::Immersive);
    let remote_life_enabled = runtime_snapshot.remote_life_enabled.unwrap_or(false);
    let immersive = interaction_mode.is_immersive();
    if immersive && !staged {
        process_message_stage(
            ChatStage::IdlePersonalityDecay,
            crate::domain::virtual_time_sync::apply_idle_personality_decay(state, role, srid),
        )
        .await?;
    }
    let is_remote = immersive && user_is_remote_from_character(scene_id, current_scene.as_deref());
    let preflight_ms = preflight_started_at.elapsed().as_millis() as u64;
    let character_scene_id =
        is_remote.then(|| current_scene.as_deref().unwrap_or("default").to_string());
    let virtual_time_ms = if staged {
        process_message_stage(
            ChatStage::VirtualTimeMs,
            state.db_manager.get_virtual_time_ms(srid),
        )
        .await?
        .unwrap_or_default()
    } else {
        process_message_stage(
            ChatStage::VirtualTimeMs,
            crate::domain::virtual_time_sync::sync_and_persist_virtual_time(
                state.db_manager.as_ref(),
                role,
                srid,
                immersive,
            ),
        )
        .await?
    };
    Ok(ImmersiveVirtualTimeState {
        remote_life_enabled,
        immersive,
        is_remote,
        character_scene_id,
        preflight_ms,
        virtual_time_ms,
    })
}

struct PreflightOutput {
    // Keep the per-session mutex alive for the complete turn. A plain
    // `MutexGuard` scoped inside `preflight_turn` used to release here.
    _turn_guard: tokio::sync::OwnedMutexGuard<()>,
    state_rid: String,
    scene_id: String,
    role: Arc<crate::models::Role>,
    t0: Instant,
    session_config: Arc<EffectiveSessionConfig>,
    effective_backends: Arc<crate::models::plugin_backends::PluginBackends>,
    pl: crate::domain::plugin_host::ResolvedRolePlugins,
    prefetch: crate::domain::chat_engine::turn_prefetch::TurnPrefetch,
    runtime_snapshot: crate::domain::role_runtime_snapshot::RoleRuntimeSnapshot,
    immersive_virtual_time: ImmersiveVirtualTimeState,
}

async fn preflight_turn(
    state: &AppState,
    req: &SendMessageRequest,
) -> std::result::Result<PreflightOutput, ProcessMessageError> {
    let mrid = req.role_id.as_str();
    let state_rid = conversation_state_role_id(mrid, req.session_id.as_deref());
    let srid = state_rid.as_str();
    let requested_scene_id = req
        .scene_id
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let t0 = Instant::now();
    let staged = req
        .adult
        .as_ref()
        .and_then(|adult| adult.stage.as_ref())
        .is_some();

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
    let turn_guard = turn_lock.lock_owned().await;
    tracing::debug!(
        target: "oclive_chat",
        role_id = %mrid,
        scene_id = %scene_id,
        session_ns = %srid,
        "send_message start",
    );

    stage_process_message(
        ChatStage::ApplyUserLlmEnv,
        crate::domain::user_llm_env::apply_user_llm_env(state).await,
    )?;

    let session_config = state.effective_session_config_for(role.as_ref(), srid);
    let effective_backends = Arc::clone(&session_config.backends);
    let effective_sources = session_config.sources.clone();
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
        session_config.slot_registry.as_ref(),
    );

    let include_adult_memory = req
        .adult
        .as_ref()
        .is_some_and(crate::models::dto::AdultInteractionRequest::gates_open);
    let mut prefetch = build_turn_prefetch(
        state,
        role.as_ref(),
        srid,
        scene_id.as_str(),
        include_adult_memory,
    )
    .await
    .map_err(|source| ProcessMessageError::Stage {
        stage: ChatStage::LoadRecentContext.as_str(),
        source,
    })?;
    if let Some(stage) = req.adult.as_ref().and_then(|adult| adult.stage.as_ref()) {
        let staged_transcripts = state
            .db_manager
            .pending_adult_stage_transcripts_before(stage.generation_id.as_str(), stage.sequence)
            .await
            .map_err(|source| ProcessMessageError::Stage {
                stage: ChatStage::LoadRecentContext.as_str(),
                source,
            })?;
        for transcript in staged_transcripts {
            prefetch.recent_turns.push((
                crate::domain::adult_stage::ADULT_CONTINUATION_INPUT.to_string(),
                transcript,
            ));
        }
    }

    let runtime_snapshot = load_turn_runtime_snapshot(state, srid, scene_id.as_str()).await?;
    let immersive_virtual_time = apply_immersive_virtual_time(
        state,
        role.as_ref(),
        srid,
        scene_id.as_str(),
        &runtime_snapshot,
        t0,
        staged,
    )
    .await?;

    Ok(PreflightOutput {
        _turn_guard: turn_guard,
        state_rid,
        scene_id,
        role,
        t0,
        session_config,
        effective_backends,
        pl,
        prefetch,
        runtime_snapshot,
        immersive_virtual_time,
    })
}

async fn run(
    state: &AppState,
    req: &SendMessageRequest,
    on_token: Option<LlmTokenSink>,
) -> std::result::Result<SendMessageResponse, ProcessMessageError> {
    let mrid = req.role_id.as_str();
    let pre = preflight_turn(state, req).await?;
    let srid = pre.state_rid.as_str();
    let scene_id = pre.scene_id.as_str();

    let staged = req
        .adult
        .as_ref()
        .and_then(|adult| adult.stage.as_ref())
        .is_some();
    if !staged {
        if let Some(response) = try_agent_shortcut(
            state,
            req,
            pre.role.as_ref(),
            srid,
            scene_id,
            mrid,
            &pre.effective_backends,
            &pre.pl,
            &pre.prefetch,
            on_token.as_ref(),
        )
        .await?
        {
            return Ok(response);
        }
    }

    // Staged adult continuation is a co-present structured beat. It must not
    // enter remote-life or agent branches that do not understand staged commit.
    let is_remote = !staged && pre.immersive_virtual_time.is_remote;
    let scenes = Arc::clone(&pre.role.scene_ids);
    let dual_core_degraded = resolve_dual_core_degraded(pre.role.as_ref());
    let turn = TurnContext {
        state,
        req,
        role: pre.role.as_ref(),
        scene_id,
        scenes,
        mrid,
        srid,
        t0: pre.t0,
        preflight_ms: pre.immersive_virtual_time.preflight_ms,
        session_config: pre.session_config,
        effective_backends: pre.effective_backends,
        pl: pre.pl.clone(),
        immersive: pre.immersive_virtual_time.immersive,
        character_scene_id: pre.immersive_virtual_time.character_scene_id,
        virtual_time_ms: pre.immersive_virtual_time.virtual_time_ms,
        dual_core_degraded,
        runtime_snapshot: pre.runtime_snapshot,
        role_arc: Arc::clone(&pre.role),
        prefetch: pre.prefetch,
    };
    if let Some(sink) = on_token {
        dispatch_turn_stream(
            &turn,
            is_remote,
            pre.immersive_virtual_time.remote_life_enabled,
            sink,
        )
        .await
    } else {
        dispatch_turn(
            &turn,
            is_remote,
            pre.immersive_virtual_time.remote_life_enabled,
        )
        .await
    }
}
