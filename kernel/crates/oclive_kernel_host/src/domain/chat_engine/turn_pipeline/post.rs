//! Main LLM call entrypoints; post-LLM orchestration lives in [`post_llm`].

use crate::domain::chat_llm_fallback::{fallback_reply_for_llm_failure, FallbackReplyContext};
use crate::domain::chat_turn_rules::{
    soft_append_guard, strip_hallucination_tokens, trim_template_repeat_reply,
};
use crate::domain::slot_runner::SlotRunner;
use std::sync::Arc;
#[cfg(feature = "dual_core")]
use std::sync::Mutex;
use std::time::Instant;

use super::super::turn_context::TurnContext;
use super::super::turn_error::TurnResult;
use super::pre::{latest_recent_turn_pair, MainLlmOutput, MiddleOutput, PreLlmOutput};
use oclive_kernel_contracts::LlmGenerateOpts;
#[cfg(feature = "dual_core")]
use oclive_validation::plugin_backends_for_slot_entry;

mod post_llm;

pub(super) use post_llm::post_llm;

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
