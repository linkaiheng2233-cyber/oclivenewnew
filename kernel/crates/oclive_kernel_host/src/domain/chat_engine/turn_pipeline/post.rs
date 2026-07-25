//! Main LLM call and post-LLM orchestration.

use crate::domain::chat_llm_fallback::{fallback_reply_for_llm_failure, FallbackReplyContext};
use crate::domain::chat_turn_rules::{
    soft_append_guard, strip_hallucination_tokens, trim_template_repeat_reply,
};
use crate::domain::host_profile::bench_telemetry_enabled;
use crate::domain::policy::PolicyContext;
use crate::domain::slot_runner::SlotRunner;
use crate::domain::turn_thinking::{
    effective_turn_thinking_policy, update_turn_thinking_runtime_state,
};
use crate::models::dto::{DisplayMetricsDto, SendMessageResponse};
use crate::models::{Event, PersonalitySource, Role};
use std::sync::Arc;
#[cfg(feature = "dual_core")]
use std::sync::Mutex;
use std::time::Instant;

use super::super::emotion_to_dto;
use super::super::scene::movement_ui_flags;
use super::super::turn_context::TurnContext;
use super::super::turn_error::TurnResult;
use super::persistence::{
    append_turn_to_chat_storage, persist_atomic_movement_portrait,
    persist_non_profile_personality_delta, ChatAppendIds, PostPersistOutcome, PostTurnPolicy,
};
use super::pre::{latest_recent_turn_pair, MainLlmOutput, MiddleOutput, PreLlmOutput, STAGES};
use super::TurnMode;
use crate::domain::chat_engine::chat_stage::ChatStage;
use crate::domain::reply_post_processor::resolve_reply_post_processor;
use oclive_kernel_contracts::reply_post_processor::PostProcessInput;
use oclive_kernel_contracts::LlmGenerateOpts;
#[cfg(feature = "dual_core")]
use oclive_validation::plugin_backends_for_slot_entry;

#[cfg(feature = "dual_core")]
fn selected_lora_llm(
    ctx: &TurnContext<'_>,
) -> Option<(String, String, Arc<dyn oclive_kernel_contracts::LlmClient>)> {
    let plugin_id = ctx.state.session_cache.expert_lora_plugin_id(ctx.srid)?;
    let registry = match ctx.session_config.slot_registry.as_ref() {
        Some(registry) => registry,
        None => {
            tracing::warn!(
                target: "oclive_expert",
                error_code = "LORA_ADAPTER_INVALID",
                session_ns = %ctx.srid,
                plugin_id = %plugin_id,
                "clearing LoRA selection because effective slot_registry is missing"
            );
            ctx.state
                .session_cache
                .set_expert_lora_plugin(ctx.srid, None);
            return None;
        }
    };
    let selection =
        match crate::domain::expert_routing::resolve_lora_llm_selection(registry, &plugin_id) {
            Ok(selection) => selection,
            Err(message) => {
                tracing::warn!(
                    target: "oclive_expert",
                    error_code = "LORA_ADAPTER_INVALID",
                    session_ns = %ctx.srid,
                    plugin_id = %plugin_id,
                    reason = %message,
                    "clearing invalid LoRA selection and using the normal LLM path"
                );
                ctx.state
                    .session_cache
                    .set_expert_lora_plugin(ctx.srid, None);
                return None;
            }
        };
    if let Err(message) = ctx
        .state
        .directory_plugins
        .ensure_rpc_url(&selection.plugin_id)
    {
        tracing::warn!(
            target: "oclive_expert",
            error_code = "LORA_ADAPTER_UNAVAILABLE",
            session_ns = %ctx.srid,
            plugin_id = %selection.plugin_id,
            slot_key = %selection.slot_key,
            reason = %message,
            "LoRA plugin unavailable; using the normal LLM path"
        );
        return None;
    }
    let backends = plugin_backends_for_slot_entry(&selection.entry);
    let llm = ctx.state.plugins.llm_for_plugin_backends(&backends);
    Some((selection.slot_key, selection.plugin_id, llm))
}

/// Artifacts produced during post-LLM orchestration, passed to response assembly.
pub(crate) struct TurnArtifacts<'a> {
    pub middle: &'a MiddleOutput,
    pub pre: &'a PreLlmOutput,
    pub llm: &'a MainLlmOutput,
    pub policy: &'a PostTurnPolicy,
    pub persist: &'a PostPersistOutcome,
    pub chat_ids: &'a ChatAppendIds,
}

/// Context for assembling the final [`SendMessageResponse`] after post-LLM work.
pub(crate) struct PostLlmCtx<'a> {
    pub mode: TurnMode,
    pub immersive: bool,
    pub scene_id: &'a str,
    pub role: &'a Role,
    pub user_message: &'a str,
    pub display_reply: String,
    pub raw_reply: Option<String>,
    pub dual_core_degraded: bool,
    pub distro_visual_mode: Option<&'a str>,
    pub movement: bool,
    pub artifacts: TurnArtifacts<'a>,
}

pub(crate) async fn run_main_llm(
    ctx: &TurnContext<'_>,
    path_label: &str,
    pre: &PreLlmOutput,
    middle: &MiddleOutput,
) -> TurnResult<MainLlmOutput> {
    let role = ctx.role;
    let user_message = ctx.req.user_message.as_str();
    let pl = &ctx.pl;
    let t_main_llm = Instant::now();
    let mut main_llm_fallback = false;
    let mut llm_fallback_reason = None;
    let ollama_opts = Some(if middle.use_ollama_prefix_opts {
        LlmGenerateOpts::deep_prefix_cache()
    } else {
        LlmGenerateOpts::interactive()
    });
    #[cfg(feature = "dual_core")]
    let selected_lora = selected_lora_llm(ctx);
    #[cfg(feature = "dual_core")]
    let generation = async {
        if let Some((slot_key, plugin_id, llm)) = selected_lora.as_ref() {
            tracing::info!(
                target: "oclive_expert",
                session_ns = %ctx.srid,
                plugin_id = %plugin_id,
                slot_key = %slot_key,
                "generating reply with selected LoRA directory LLM"
            );
            match SlotRunner::generate_llm_single(
                llm,
                pre.memory.ollama_model.as_str(),
                &middle.prompt,
                ollama_opts.as_ref(),
            )
            .await
            {
                Ok(out) => Ok(out),
                Err(error) => {
                    tracing::warn!(
                        target: "oclive_expert",
                        error_code = "LORA_ADAPTER_GENERATE_FAILED",
                        session_ns = %ctx.srid,
                        plugin_id = %plugin_id,
                        slot_key = %slot_key,
                        reason = %error,
                        "LoRA generation failed; clearing selection and retrying the normal LLM"
                    );
                    ctx.state
                        .session_cache
                        .set_expert_lora_plugin(ctx.srid, None);
                    SlotRunner::generate_llm(
                        pl,
                        pre.memory.ollama_model.as_str(),
                        &middle.prompt,
                        ollama_opts.as_ref(),
                    )
                    .await
                }
            }
        } else {
            SlotRunner::generate_llm(
                pl,
                pre.memory.ollama_model.as_str(),
                &middle.prompt,
                ollama_opts.as_ref(),
            )
            .await
        }
    };
    #[cfg(not(feature = "dual_core"))]
    let generation = SlotRunner::generate_llm(
        pl,
        pre.memory.ollama_model.as_str(),
        &middle.prompt,
        ollama_opts.as_ref(),
    );
    let reply_out = match generation.await {
        Ok(out) => out,
        Err(e) => {
            let reason = e.to_frontend_error();
            tracing::warn!("{path_label} LLM generate failed, fallback: {reason}");
            main_llm_fallback = true;
            llm_fallback_reason = Some(reason);
            let fallback = fallback_reply_for_llm_failure(
                role,
                &middle.personality,
                user_message,
                &FallbackReplyContext {
                    relation_before: pre.relation.relation_before.as_str(),
                    relation_preview: middle.relation_after.as_str(),
                    favorability_before: pre.relation.favorability_before,
                    event_type: &middle.ai_event_type,
                    impact_factor: middle.ai_impact_factor_final,
                },
            );
            oclive_kernel_contracts::LlmGenerateOutcome {
                reply: fallback,
                prompt_eval_ms: None,
            }
        }
    };
    if let (Some(hash), Some(len), Some(hit)) = (
        middle.prompt_stable_hash,
        middle.prompt_stable_len,
        middle.prefix_cache_expected_hit,
    ) {
        tracing::debug!(
            target: "oclive_turn",
            prefix_hash = hash,
            stable_len = len,
            cache_expected_hit = hit,
            prompt_eval_ms = ?reply_out.prompt_eval_ms,
            "prompt prefix cache llm metrics"
        );
    }
    let reply_raw = reply_out.reply;
    let llm_prompt_eval_ms = reply_out.prompt_eval_ms;
    let main_llm_ms = t_main_llm.elapsed().as_millis() as u64;
    let (_, previous_assistant_reply) = latest_recent_turn_pair(&pre.memory.recent_turns);
    let reply = strip_hallucination_tokens(&soft_append_guard(
        &trim_template_repeat_reply(previous_assistant_reply.as_str(), &reply_raw),
        &middle.ai_event_type,
        middle.ai_impact_factor_final,
        middle.relation_after.as_str(),
    ));

    Ok(MainLlmOutput {
        reply,
        main_llm_fallback,
        llm_fallback_reason,
        main_llm_ms,
        llm_prompt_eval_ms,
    })
}

pub(crate) async fn run_main_llm_stream(
    ctx: &TurnContext<'_>,
    path_label: &str,
    pre: &PreLlmOutput,
    middle: &MiddleOutput,
    on_token: oclive_kernel_contracts::LlmTokenSink,
) -> TurnResult<MainLlmOutput> {
    let role = ctx.role;
    let user_message = ctx.req.user_message.as_str();
    let pl = &ctx.pl;
    let t_main_llm = Instant::now();
    let mut main_llm_fallback = false;
    let mut llm_fallback_reason = None;
    let ollama_opts = Some(if middle.use_ollama_prefix_opts {
        LlmGenerateOpts::deep_prefix_cache()
    } else {
        LlmGenerateOpts::interactive()
    });
    #[cfg(feature = "dual_core")]
    let selected_lora = selected_lora_llm(ctx);
    #[cfg(feature = "dual_core")]
    let generation = async {
        if let Some((slot_key, plugin_id, llm)) = selected_lora.as_ref() {
            tracing::info!(
                target: "oclive_expert",
                session_ns = %ctx.srid,
                plugin_id = %plugin_id,
                slot_key = %slot_key,
                "streaming reply with selected LoRA directory LLM"
            );
            let streamed = Arc::new(Mutex::new(String::new()));
            let streamed_for_sink = Arc::clone(&streamed);
            let downstream = Arc::clone(&on_token);
            let passthrough_sink: oclive_kernel_contracts::LlmTokenSink = Arc::new(move |token| {
                if let Ok(mut output) = streamed_for_sink.lock() {
                    output.push_str(token);
                }
                downstream(token);
            });
            match SlotRunner::generate_llm_stream_single(
                llm,
                pre.memory.ollama_model.as_str(),
                &middle.prompt,
                passthrough_sink,
                ollama_opts.as_ref(),
            )
            .await
            {
                Ok(out) => Ok(out),
                Err(error) => {
                    let partial = streamed
                        .lock()
                        .map(|output| output.clone())
                        .unwrap_or_default();
                    ctx.state
                        .session_cache
                        .set_expert_lora_plugin(ctx.srid, None);
                    if !partial.is_empty() {
                        tracing::warn!(
                            target: "oclive_expert",
                            error_code = "LORA_ADAPTER_STREAM_PARTIAL",
                            session_ns = %ctx.srid,
                            plugin_id = %plugin_id,
                            slot_key = %slot_key,
                            emitted_bytes = partial.len(),
                            reason = %error,
                            "LoRA stream failed after emitting output; preserving the partial reply without duplicate fallback tokens"
                        );
                        return Ok(oclive_kernel_contracts::LlmGenerateOutcome {
                            reply: partial,
                            prompt_eval_ms: None,
                        });
                    }
                    tracing::warn!(
                        target: "oclive_expert",
                        error_code = "LORA_ADAPTER_GENERATE_FAILED",
                        session_ns = %ctx.srid,
                        plugin_id = %plugin_id,
                        slot_key = %slot_key,
                        reason = %error,
                        "LoRA stream failed before first token; retrying the normal LLM"
                    );
                    SlotRunner::generate_llm_stream(
                        pl,
                        pre.memory.ollama_model.as_str(),
                        &middle.prompt,
                        Arc::clone(&on_token),
                        ollama_opts.as_ref(),
                    )
                    .await
                }
            }
        } else {
            SlotRunner::generate_llm_stream(
                pl,
                pre.memory.ollama_model.as_str(),
                &middle.prompt,
                Arc::clone(&on_token),
                ollama_opts.as_ref(),
            )
            .await
        }
    };
    #[cfg(not(feature = "dual_core"))]
    let generation = SlotRunner::generate_llm_stream(
        pl,
        pre.memory.ollama_model.as_str(),
        &middle.prompt,
        Arc::clone(&on_token),
        ollama_opts.as_ref(),
    );
    let reply_out = match generation.await {
        Ok(out) => out,
        Err(e) => {
            let reason = e.to_frontend_error();
            tracing::warn!("{path_label} LLM generate_stream failed, fallback: {reason}");
            main_llm_fallback = true;
            llm_fallback_reason = Some(reason);
            let fallback = fallback_reply_for_llm_failure(
                role,
                &middle.personality,
                user_message,
                &FallbackReplyContext {
                    relation_before: pre.relation.relation_before.as_str(),
                    relation_preview: middle.relation_after.as_str(),
                    favorability_before: pre.relation.favorability_before,
                    event_type: &middle.ai_event_type,
                    impact_factor: middle.ai_impact_factor_final,
                },
            );
            on_token(fallback.as_str());
            oclive_kernel_contracts::LlmGenerateOutcome {
                reply: fallback,
                prompt_eval_ms: None,
            }
        }
    };
    let reply_raw = reply_out.reply;
    let llm_prompt_eval_ms = reply_out.prompt_eval_ms;
    let main_llm_ms = t_main_llm.elapsed().as_millis() as u64;
    let (_, previous_assistant_reply) = latest_recent_turn_pair(&pre.memory.recent_turns);
    let reply = strip_hallucination_tokens(&soft_append_guard(
        &trim_template_repeat_reply(previous_assistant_reply.as_str(), &reply_raw),
        &middle.ai_event_type,
        middle.ai_impact_factor_final,
        middle.relation_after.as_str(),
    ));

    Ok(MainLlmOutput {
        reply,
        main_llm_fallback,
        llm_fallback_reason,
        main_llm_ms,
        llm_prompt_eval_ms,
    })
}

#[allow(clippy::too_many_arguments)]
async fn analyze_bot_emotion_and_policy(
    state: &crate::state::AppState,
    pl: &crate::domain::plugin_host::ResolvedRolePlugins,
    scene_id: &str,
    srid: &str,
    user_message: &str,
    reply: &str,
    pre: &PreLlmOutput,
    middle: &MiddleOutput,
    snapshot_emotion: Option<String>,
) -> TurnResult<PostTurnPolicy> {
    let policies = state.policies_for_scene(Some(scene_id));
    let bot_emotion_result = STAGES
        .stage(ChatStage::BotReplyEmotionAnalyze, async {
            SlotRunner::analyze_emotion(pl, reply)
        })
        .await?;
    let previous_emotion = if let Some(emotion) = snapshot_emotion {
        Some(emotion)
    } else {
        STAGES
            .stage(
                ChatStage::GetCurrentEmotion,
                state.db_manager.get_current_emotion(srid),
            )
            .await?
    };
    let bot_emotion = policies
        .emotion
        .resolve_current_emotion(previous_emotion.as_deref(), &bot_emotion_result);
    let bot_emotion_str = bot_emotion.to_string();
    let event = Event {
        event_type: middle.ai_event_type,
        user_emotion: pre.hints.user_emotion_str.clone(),
        bot_emotion: bot_emotion_str.clone(),
    };
    let policy_ctx = PolicyContext {
        role_id: srid,
        user_message,
        reply,
        event: &event,
        event_confidence: middle.ai_event_confidence,
    };
    let memory_line = policies.memory.build_memory_entry(&policy_ctx);
    let memory_importance = if policies.memory.should_persist(&policy_ctx) {
        policies.memory.importance(&policy_ctx)
    } else {
        0.0
    };
    let memory_importance = middle.turn_thinking.memory_importance_after_policy(
        &state.host_profile,
        &middle.ai_event_type,
        memory_importance,
    );
    let mut recent_events = Vec::with_capacity(pre.memory.recent_events_for_event.len() + 1);
    recent_events.push(event.clone());
    recent_events.extend(pre.memory.recent_events_for_event.iter().cloned());
    Ok(PostTurnPolicy {
        bot_emotion,
        bot_emotion_str,
        event,
        memory_line,
        memory_importance,
        recent_events,
    })
}

#[allow(clippy::too_many_arguments)]
fn spawn_profile_evolution_after_llm(
    state: &crate::state::AppState,
    primary_llm: Arc<dyn crate::domain::ports::LlmClient>,
    role_arc: Arc<Role>,
    srid: &str,
    path_label: &str,
    pre: &PreLlmOutput,
    middle: &MiddleOutput,
    user_message: &str,
    reply: &str,
) {
    if role_arc.evolution_config.personality_source != PersonalitySource::Profile {
        return;
    }
    let turn_index = state.session_cache.increment_profile_evolution_turn(srid);
    let interval_n = crate::domain::turn_thinking::effective_deep_profile_update_interval(
        state.host_profile.as_ref(),
        role_arc.as_ref(),
    );
    let applies_full = middle
        .turn_thinking
        .applies_full_persistence(state.host_profile.as_ref(), &middle.ai_event_type);
    let radar_pending = state.session_cache.radar_deep_pending(srid);
    if !crate::domain::turn_thinking::should_run_deep_profile_update(
        applies_full,
        turn_index,
        interval_n,
        radar_pending,
    ) {
        return;
    }
    if radar_pending {
        state.session_cache.clear_radar_deep_pending(srid);
    }
    let impact_scaled = (middle.ai_impact_factor_final * pre.memory.event_runtime).clamp(-1.0, 1.0);
    crate::state::profile_evolution::spawn_mutable_profile_evolution(
        state,
        primary_llm,
        role_arc,
        srid.to_string(),
        path_label.to_string(),
        pre.memory.ollama_model.clone(),
        user_message.to_string(),
        reply.to_string(),
        pre.hints.user_emotion_str.clone(),
        middle.ai_event_type,
        impact_scaled,
    );
}

fn assemble_send_message_response(ctx: &PostLlmCtx<'_>) -> SendMessageResponse {
    use crate::models::dto::{DetectedEventDto, PresenceMode, API_VERSION, SCHEMA_VERSION};

    let PostLlmCtx {
        mode,
        immersive,
        scene_id,
        role,
        user_message,
        display_reply,
        raw_reply,
        dual_core_degraded,
        distro_visual_mode,
        movement,
        artifacts,
    } = ctx;
    let middle = artifacts.middle;
    let pre = artifacts.pre;
    let llm = artifacts.llm;
    let policy = artifacts.policy;
    let persist = artifacts.persist;
    let chat_ids = artifacts.chat_ids;
    let reply = display_reply.clone();

    let (mut offer_destination_picker, mut offer_together_travel) =
        movement_ui_flags(*movement, user_message);
    if matches!(mode, TurnMode::CoPresent) && !immersive {
        offer_destination_picker = false;
        offer_together_travel = false;
    }
    let (presence_mode, events) = match mode {
        TurnMode::CoPresent => (
            PresenceMode::CoPresent,
            vec![DetectedEventDto {
                event_type: policy.event.event_type.as_ref().to_string(),
                confidence: middle.ai_event_confidence,
            }],
        ),
        TurnMode::RemoteLife => (PresenceMode::RemoteLife, vec![]),
    };
    SendMessageResponse {
        api_version: API_VERSION,
        schema: SCHEMA_VERSION,
        presence_mode,
        display_metrics: Some(DisplayMetricsDto {
            favor: persist.favor_current,
            relation_summary: middle.relation_after.as_str().to_string(),
            traits: middle.personality.to_vec7(),
        }),
        relation_state: middle.relation_after.as_str().to_string(),
        reply,
        emotion: emotion_to_dto(&pre.hints.emotion_result),
        bot_emotion: policy.bot_emotion_str.clone(),
        portrait_emotion: persist.portrait_emotion_str.clone(),
        visual_state_id: persist.visual_state_id.clone(),
        performance_directive: persist.visual_state_id.as_deref().and_then(|id| {
            crate::domain::visual_presentation::materialize_directive_gated(
                role,
                id,
                *distro_visual_mode,
            )
        }),
        favorability_delta: middle.favor_delta as f32,
        favorability_current: persist.favor_current as f32,
        events,
        scene_id: scene_id.to_string(),
        offer_destination_picker,
        offer_together_travel,
        reply_is_fallback: llm.main_llm_fallback,
        llm_fallback_reason: llm.llm_fallback_reason.clone(),
        knowledge_chunks_in_prompt: middle.knowledge_chunk_count,
        timestamp: chrono::Utc::now().timestamp_millis(),
        user_message_id: chat_ids.user_message_id.clone(),
        assistant_message_id: chat_ids.assistant_message_id.clone(),
        user_message_timestamp: chat_ids.user_message_timestamp.clone(),
        assistant_message_timestamp: chat_ids.assistant_message_timestamp.clone(),
        chat_persist_failed: chat_ids.chat_persist_failed,
        chat_persist_error: chat_ids.chat_persist_error.clone(),
        dual_core_degraded: (*dual_core_degraded).then_some(true),
        raw_reply: raw_reply.clone(),
        llm_prompt_eval_ms: if bench_telemetry_enabled() {
            llm.llm_prompt_eval_ms
        } else {
            None
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_reply_post_processor(
    state: &crate::state::AppState,
    role: &Role,
    mrid: &str,
    scene_id: &str,
    srid: &str,
    user_message: &str,
    reply: &str,
    include_raw_reply: bool,
) -> (String, Option<String>) {
    let raw_reply_before = reply.to_string();
    let processor = resolve_reply_post_processor(state, role);
    let display_reply = match processor.process_reply(PostProcessInput {
        raw_reply: reply,
        user_message,
        role_id: mrid,
        scene_id,
        srid,
        locale: "zh",
    }) {
        Ok(out) => out.display_reply,
        Err(e) => {
            tracing::warn!(
                target: "oclive_reply_post_processor",
                role_id = %mrid,
                error = %e,
                "reply post-processor failed; using raw reply"
            );
            raw_reply_before.clone()
        }
    };
    let raw_reply =
        (include_raw_reply && display_reply != raw_reply_before).then_some(raw_reply_before);
    (display_reply, raw_reply)
}

pub(crate) async fn post_llm(
    ctx: &TurnContext<'_>,
    mode: TurnMode,
    path_label: &str,
    pre: &PreLlmOutput,
    middle: &MiddleOutput,
    llm: &MainLlmOutput,
    pre_main_llm_ms: u64,
) -> TurnResult<SendMessageResponse> {
    let state = ctx.state;
    let role = ctx.role;
    let scene_id = ctx.scene_id;
    let scenes = std::sync::Arc::clone(&ctx.scenes);
    let mrid = ctx.mrid;
    let srid = ctx.srid;
    let t0 = ctx.t0;
    let preflight_ms = ctx.preflight_ms;
    let immersive = ctx.immersive;
    let pl = &ctx.pl;
    let user_message = ctx.req.user_message.as_str();
    let primary_llm = SlotRunner::primary_llm(pl);

    let t_post_llm = Instant::now();
    let reply = llm.reply.clone();

    let policy = analyze_bot_emotion_and_policy(
        state,
        pl,
        scene_id,
        srid,
        user_message,
        &reply,
        pre,
        middle,
        ctx.runtime_snapshot.emotion.clone(),
    )
    .await?;

    spawn_profile_evolution_after_llm(
        state,
        Arc::clone(&primary_llm),
        Arc::clone(&ctx.role_arc),
        srid,
        path_label,
        pre,
        middle,
        user_message,
        &reply,
    );

    let persist_out = persist_atomic_movement_portrait(
        state,
        mode,
        primary_llm,
        role,
        ctx.ids(),
        scenes,
        user_message,
        pre,
        middle,
        &policy,
        &reply,
    )
    .await?;

    if matches!(mode, TurnMode::CoPresent)
        && !middle.complex_emotion_out.source.eq("turn_thinking_fast")
    {
        let hint = middle.complex_emotion_out.narrative_hint.clone();
        if !hint.trim().is_empty() {
            crate::domain::complex_emotion_store::persist_stored_narrative_hint(state, srid, hint)
                .await;
        }
    }

    let (display_reply, raw_reply) = apply_reply_post_processor(
        state,
        role,
        mrid,
        scene_id,
        srid,
        user_message,
        &reply,
        ctx.req.include_raw_reply == Some(true),
    );

    let chat_ids = append_turn_to_chat_storage(
        state,
        mode,
        ctx.ids(),
        role,
        pre,
        llm,
        &policy,
        user_message,
        &display_reply,
    )
    .await;

    if matches!(mode, TurnMode::CoPresent) {
        if let Err(error) = crate::domain::narrative_continuity::update_after_reply(
            state,
            role,
            srid,
            scene_id,
            display_reply.as_str(),
        )
        .await
        {
            tracing::warn!(
                target: "oclive_continuity",
                role_id = %srid,
                scene_id,
                error = %error,
                "narrative continuity transition failed; preserving previous state"
            );
        }
    }

    persist_non_profile_personality_delta(state, role, srid, middle).await;

    if matches!(mode, TurnMode::CoPresent) {
        let policy = effective_turn_thinking_policy(&state.host_profile, role);
        if let Err(e) = update_turn_thinking_runtime_state(
            &state.turn_thinking_state(),
            srid,
            &policy,
            middle.ai_event_type,
            user_message,
        )
        .await
        {
            tracing::warn!(
                target: "oclive_turn",
                role_id = %srid,
                error = %e,
                "turn_thinking runtime state update failed"
            );
        }
    }

    let response = assemble_send_message_response(&PostLlmCtx {
        mode,
        immersive,
        scene_id,
        role,
        user_message,
        display_reply,
        raw_reply,
        dual_core_degraded: ctx.dual_core_degraded,
        distro_visual_mode: ctx.state.host_profile.visual_presentation_mode.as_deref(),
        movement: persist_out.movement,
        artifacts: TurnArtifacts {
            middle,
            pre,
            llm,
            policy: &policy,
            persist: &persist_out,
            chat_ids: &chat_ids,
        },
    });

    let post_llm_ms = t_post_llm.elapsed().as_millis() as u64;
    let duration_ms = t0.elapsed().as_millis() as u64;
    tracing::info!(
        target: "oclive_chat",
        path_label = path_label,
        role_id = %mrid,
        scene_id = %scene_id,
        duration_ms = duration_ms,
        main_llm_fallback = llm.main_llm_fallback,
        offer_destination_picker = response.offer_destination_picker,
        offer_together_travel = response.offer_together_travel,
        "send_message end",
    );
    tracing::debug!(
        target: "oclive_chat",
        path_label = path_label,
        preflight_ms = preflight_ms,
        pre_main_llm_ms = pre_main_llm_ms,
        main_llm_ms = llm.main_llm_ms,
        post_llm_ms = post_llm_ms,
        duration_ms = duration_ms,
        "send_message stage timing",
    );

    Ok(response)
}
