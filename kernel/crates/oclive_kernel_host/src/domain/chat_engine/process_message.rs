//! # Main message processing entry
//!
//! **Role**: orchestration entry for one semantic turn. Tauri / HTTP API use the user-origin wrapper;
//! trusted embedded hosts may use an explicit non-user origin. Agent shortcut, remote-life, and
//! other branches fan out here into [`turn_pipeline`](super::turn_pipeline)
//! ([`TurnMode::CoPresent`](super::turn_pipeline::TurnMode::CoPresent), etc.).
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
use crate::domain::chat_engine::turn_context::{TurnContext, TurnInput};
use crate::domain::chat_engine::turn_prefetch::build_turn_prefetch;
use crate::domain::chat_engine::{
    backend_resolution_summary, context::validate_scene_id, conversation_state_role_id,
    ensure_role_loaded,
};
use crate::domain::event_ring::ProactiveTurnPermit;
use crate::domain::startup_health;
use crate::error::Result;
use crate::models::dto::{SendMessageRequest, SendMessageResponse, TurnOrigin};
use crate::models::plugin_backends::AgentBackend;
use crate::state::AppState;
use crate::state::EffectiveSessionConfig;
use oclive_kernel_contracts::LlmTokenSink;
use std::sync::Arc;
use std::time::Instant;

/// Reference Host transport adapter for an explicit local minimum-role source.
/// Blocking loading and the local (not necessarily Send) Base future stay on one
/// worker. The current Tokio handle supplies I/O; no new scheduler or runtime is
/// created. This does not activate a rich role, persist or retry the text call.
///
/// # Errors
/// Returns existing Host errors. This fixed transport rejects unsupported
/// requirements before loading; it defines no general Base-error wire mapping.
pub async fn process_minimal_local_message(
    state: Arc<AppState>,
    request: oclive_kernel_types::models::dto::MinimalRoleLocalMessageRequest,
) -> Result<oclive_kernel_types::models::dto::MinimalRoleMessageResponse> {
    if request.message.user_message.trim().is_empty() {
        return Err(crate::error::AppError::EmptyMessage);
    }
    if !request.message.requirements.is_empty() {
        return Err(crate::error::AppError::InvalidParameter(
            "the bound minimal transport Prompt does not support additional requirements".into(),
        ));
    }
    let runtime = tokio::runtime::Handle::try_current().map_err(|error| {
        crate::error::AppError::Unknown(format!("minimal transport needs a Tokio runtime: {error}"))
    })?;
    tokio::task::spawn_blocking(move || {
        let role =
            crate::service::role::minimal::PreparedMinimalRole::from_local_source(&request.source)?;
        runtime.block_on(async {
            match process_minimal_message(&state, &role, &request.message).await {
                Ok(response) => Ok(response),
                Err(crate::domain::chat_engine::message_error::MinimalRoleMessageError::Host(
                    error,
                )) => Err(error),
                // With validated content and empty requirements the fixed Literal
                // Prompt is infallible. Preserve any unexpected diagnostic as a
                // Host failure, not a general conversion of Base error semantics.
                Err(
                    crate::domain::chat_engine::message_error::MinimalRoleMessageError::Prompt(
                        error,
                    ),
                ) => Err(crate::error::AppError::Unknown(format!(
                    "bound minimal Prompt failed: {error}"
                ))),
            }
        })
    })
    .await
    .map_err(|error| {
        crate::error::AppError::Unknown(format!("minimal text worker failed: {error}"))
    })?
}

/// Reference Host's additive single-call text path, organized at the canonical
/// turn entry rather than in the public facade. It deliberately does not enter
/// the rich preflight/TurnContext or publish synthetic extension state.
///
/// # Errors
/// Returns the original Host error or the Prompt Base failure carrier.
pub async fn process_minimal_message(
    state: &AppState,
    role: &crate::service::role::minimal::PreparedMinimalRole,
    request: &oclive_kernel_types::models::dto::MinimalRoleMessageRequest,
) -> std::result::Result<
    oclive_kernel_types::models::dto::MinimalRoleMessageResponse,
    crate::domain::chat_engine::message_error::MinimalRoleMessageError,
> {
    process_minimal_message_using_prompt(state, role, request, None).await
}

/// The same minimal text path with a Host-selected, unconfigured Prompt Base.
/// The shared consumer prepares role materials; selection and the capability's
/// requirement agreement remain the caller's responsibility.
///
/// # Errors
/// Returns the original Host error or complete selected Prompt failure.
pub async fn process_minimal_message_with_prompt(
    state: &AppState,
    role: &crate::service::role::minimal::PreparedMinimalRole,
    request: &oclive_kernel_types::models::dto::MinimalRoleMessageRequest,
    prompt: &dyn oclive_kernel_contracts::PromptBase,
) -> std::result::Result<
    oclive_kernel_types::models::dto::MinimalRoleMessageResponse,
    crate::domain::chat_engine::message_error::MinimalRoleMessageError,
> {
    process_minimal_message_using_prompt(state, role, request, Some(prompt)).await
}

async fn process_minimal_message_using_prompt(
    state: &AppState,
    role: &crate::service::role::minimal::PreparedMinimalRole,
    request: &oclive_kernel_types::models::dto::MinimalRoleMessageRequest,
    selected_prompt: Option<&dyn oclive_kernel_contracts::PromptBase>,
) -> std::result::Result<
    oclive_kernel_types::models::dto::MinimalRoleMessageResponse,
    crate::domain::chat_engine::message_error::MinimalRoleMessageError,
> {
    use oclive_kernel_contracts::PromptBase;
    use oclive_kernel_runtime::domain::minimal_role_consumer::MinimalRolePromptConsumer;
    use oclive_kernel_runtime::domain::minimal_role_prompt::MinimalRolePrompt;
    use oclive_kernel_types::models::dto::{
        MinimalRoleMessageResponse, MinimalRoleProductExtensionStatus,
    };
    use oclive_kernel_types::PromptBaseRequest;

    if request.user_message.trim().is_empty() {
        return Err(crate::error::AppError::EmptyMessage.into());
    }
    let invalid_role =
        |errors: Vec<String>| crate::error::AppError::InvalidParameter(errors.join("; "));
    let materials = ["User: ", request.user_message.as_str()];
    let prompt_request = PromptBaseRequest {
        materials: &materials,
        requirements: &request.requirements,
    };
    let prepared = match selected_prompt {
        Some(prompt) => {
            MinimalRolePromptConsumer::new(role.definition(), prompt)
                .map_err(invalid_role)?
                .assemble(prompt_request)
                .await?
        }
        None => {
            MinimalRolePrompt::new(role.definition())
                .map_err(invalid_role)?
                .assemble(prompt_request)
                .await?
        }
    };
    crate::domain::user_llm_env::apply_user_llm_env(state).await?;
    let model = state.ollama_model.read().clone();
    let generated = crate::domain::slot_runner::SlotRunner::generate_llm_single(
        &state.llm, &model, &prepared, None,
    )
    .await?;
    Ok(MinimalRoleMessageResponse {
        role_id: role.technical_id().to_owned(),
        reply: generated.reply,
        product_extensions: MinimalRoleProductExtensionStatus::Unavailable,
    })
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn process_message(
    state: &AppState,
    req: &SendMessageRequest,
) -> Result<SendMessageResponse> {
    process_message_with_origin(state, req, TurnOrigin::User).await
}

/// Read or wait for an existing logical user turn. Never starts a new turn,
/// including when talking to a different database after a host restart.
///
/// # Errors
/// Returns conflict for a changed payload, or unconfirmed if no result is available.
pub async fn recover_message(
    state: &AppState,
    req: &SendMessageRequest,
) -> Result<SendMessageResponse> {
    super::request_receipt::recover(state.db_manager.as_ref(), req).await
}

/// Processes one message with an explicit semantic origin.
///
/// Embedded hosts use this entrypoint for sensor/system turns. Those origins may read role context
/// and generate a role response, but they do not commit user-chat state. HTTP and Tauri chat
/// transports intentionally keep calling [`process_message`] so external payloads cannot select a
/// lower-persistence origin.
///
/// # Errors
///
/// Returns [`Err`] with the same failure modes as [`process_message`].
pub async fn process_message_with_origin(
    state: &AppState,
    req: &SendMessageRequest,
    origin: TurnOrigin,
) -> Result<SendMessageResponse> {
    match run_with_origin_boundary(state, req, None, origin).await {
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
    process_message_stream_with_origin(state, req, on_token, TurnOrigin::User).await
}

/// Streaming counterpart of [`process_message_with_origin`].
///
/// # Errors
///
/// Same failure modes as [`process_message_stream`].
pub async fn process_message_stream_with_origin(
    state: &AppState,
    req: &SendMessageRequest,
    on_token: LlmTokenSink,
    origin: TurnOrigin,
) -> Result<SendMessageResponse> {
    match run_with_origin_boundary(state, req, Some(on_token), origin).await {
        Ok(v) => Ok(v),
        Err(e) => {
            tracing::error!(target: "oclive_chat", "{}", e);
            Err(e.into())
        }
    }
}

/// Executes one Event-authorized proactive turn.
///
/// Only an opaque [`ProactiveTurnPermit`] produced by the Event Ring is accepted. The external
/// observation remains separate from `SendMessageRequest.user_message`, and the turn reuses the
/// Event Ring correlation ID without dispatching from inside an Event handler.
///
/// # Errors
///
/// Returns the same role-loading, prompt, model, and post-processing errors as [`process_message`].
pub async fn process_proactive_turn(
    state: &AppState,
    permit: ProactiveTurnPermit,
) -> Result<SendMessageResponse> {
    let (authorization, event_id, correlation_id) = permit.into_parts();
    let proposal = authorization.proposal;
    let req = SendMessageRequest {
        client_request_id: None,
        role_id: proposal.role_id,
        user_message: String::new(),
        scene_id: proposal.scene_id,
        session_id: proposal.session_id,
        include_raw_reply: None,
        adult: None,
    };
    tracing::debug!(
        target: "oclive_proactive",
        authorization_event_id = %event_id,
        correlation_id = %correlation_id,
        influence_bps = authorization.reply_influence_bps,
        "executing Event-authorized proactive turn"
    );
    match run(
        state,
        &req,
        None,
        proposal.origin,
        TurnInput::ExternalObservation(proposal.observation.as_str()),
        Some(correlation_id.as_str()),
    )
    .await
    {
        Ok(response) => Ok(response),
        Err(error) => {
            tracing::error!(target: "oclive_chat", "{}", error);
            Err(error.into())
        }
    }
}

async fn run_with_origin_boundary(
    state: &AppState,
    req: &SendMessageRequest,
    on_token: Option<LlmTokenSink>,
    origin: TurnOrigin,
) -> std::result::Result<SendMessageResponse, ProcessMessageError> {
    if origin == TurnOrigin::User {
        if req.client_request_id.is_none() {
            return run(
                state,
                req,
                on_token,
                origin,
                TurnInput::UserMessage(req.user_message.as_str()),
                None,
            )
            .await;
        }
        return super::request_receipt::execute(state.db_manager.as_ref(), req, async {
            run(
                state,
                req,
                on_token,
                origin,
                TurnInput::UserMessage(req.user_message.as_str()),
                None,
            )
            .await
            .map_err(Into::into)
        })
        .await
        .map_err(|source| ProcessMessageError::stage("request_receipt", source));
    }

    if req.client_request_id.is_some() {
        return Err(ProcessMessageError::stage(
            "request_receipt",
            crate::error::AppError::InvalidParameter(
                "client_request_id is only valid for user turns".into(),
            ),
        ));
    }
    let sanitized_req = SendMessageRequest {
        client_request_id: None,
        role_id: req.role_id.clone(),
        user_message: String::new(),
        scene_id: req.scene_id.clone(),
        session_id: req.session_id.clone(),
        include_raw_reply: req.include_raw_reply,
        adult: None,
    };
    run(
        state,
        &sanitized_req,
        on_token,
        origin,
        TurnInput::ExternalObservation(req.user_message.as_str()),
        None,
    )
    .await
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
    persist_user_state: bool,
) -> std::result::Result<
    crate::domain::role_runtime_snapshot::RoleRuntimeSnapshot,
    ProcessMessageError,
> {
    if !persist_user_state {
        return process_message_stage(ChatStage::GetRoleRuntimeSnapshot, async {
            state
                .db_manager
                .get_role_runtime_snapshot(srid)
                .await?
                .ok_or(crate::error::AppError::RoleRuntimeNotReady)
        })
        .await;
    }
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
    persist_user_state: bool,
) -> std::result::Result<ImmersiveVirtualTimeState, ProcessMessageError> {
    let current_scene = runtime_snapshot.scene.clone();
    let interaction_mode = runtime_snapshot
        .interaction_mode
        .unwrap_or(crate::models::InteractionMode::Immersive);
    let remote_life_enabled = runtime_snapshot.remote_life_enabled.unwrap_or(false);
    let immersive = interaction_mode.is_immersive();
    if immersive && persist_user_state {
        process_message_stage(
            ChatStage::IdlePersonalityDecay,
            crate::domain::virtual_time_sync::apply_idle_personality_decay(state, role, srid),
        )
        .await?;
    }
    let is_remote = persist_user_state
        && immersive
        && user_is_remote_from_character(scene_id, current_scene.as_deref());
    let preflight_ms = preflight_started_at.elapsed().as_millis() as u64;
    let character_scene_id =
        is_remote.then(|| current_scene.as_deref().unwrap_or("default").to_string());
    let virtual_time_ms = if !persist_user_state {
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
    origin: TurnOrigin,
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
    let persist_user_state = origin.persists_user_state() && !staged;

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

    let runtime_snapshot =
        load_turn_runtime_snapshot(state, srid, scene_id.as_str(), persist_user_state).await?;
    let immersive_virtual_time = apply_immersive_virtual_time(
        state,
        role.as_ref(),
        srid,
        scene_id.as_str(),
        &runtime_snapshot,
        t0,
        persist_user_state,
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

async fn run<'a>(
    state: &'a AppState,
    req: &'a SendMessageRequest,
    on_token: Option<LlmTokenSink>,
    origin: TurnOrigin,
    input: TurnInput<'a>,
    correlation_id: Option<&str>,
) -> std::result::Result<SendMessageResponse, ProcessMessageError> {
    let mrid = req.role_id.as_str();
    let pre = preflight_turn(state, req, origin).await?;
    let srid = pre.state_rid.as_str();
    let scene_id = pre.scene_id.as_str();

    let staged = req
        .adult
        .as_ref()
        .and_then(|adult| adult.stage.as_ref())
        .is_some();
    if !staged && origin == TurnOrigin::User {
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
    // Remote-life prompts currently model a user-addressed message. External observations stay on
    // the co-present path until that prompt family receives the same origin-aware evidence shape.
    let is_remote = !staged && input.is_user_message() && pre.immersive_virtual_time.is_remote;
    let scenes = Arc::clone(&pre.role.scene_ids);
    let dual_core_degraded = resolve_dual_core_degraded(pre.role.as_ref());
    let correlation_id = match correlation_id {
        Some(value) => value.to_string(),
        None => uuid::Uuid::new_v4().to_string(),
    };
    let turn = TurnContext {
        state,
        req,
        correlation_id,
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
        origin,
        input,
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
