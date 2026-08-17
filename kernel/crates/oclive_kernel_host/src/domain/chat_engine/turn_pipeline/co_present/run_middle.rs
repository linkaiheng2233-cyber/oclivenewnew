//! Co-present middle stage: event estimate, thinking router, prompt build.

use crate::domain::event_impact_ai::estimate_event_impact_rules_only;
use crate::domain::host_profile::{prompt_prefix_cache_effective, DISTRO_CONCISE_PROMPT_OVERLAY};
use crate::domain::life_schedule::{format_life_prompt_line, pick_life_state};
use crate::domain::model_tier::{
    persona_override_for_source, resolve_model_tier, resolve_persona_source, PersonaSource,
};
use crate::domain::personality_engine::PersonalityEngine;
use crate::domain::prompt_builder::{
    effective_reply_quality_anchor, hash_stable_prefix, PromptInput,
};
use crate::domain::reply_mode::{effective_reply_mode, reply_output_format_instruction};
use crate::domain::slot_runner::SlotRunner;
use crate::domain::turn_thinking::{resolve_turn_thinking, TurnThinkingMode};
use crate::models::knowledge::KnowledgeIndex;
use crate::models::Memory;
use crate::models::PersonalitySource;
use oclive_kernel_types::PromptExtraSection;

use super::super::super::turn_context::TurnContext;
use super::super::super::turn_error::TurnResult;
use super::super::{
    build_complex_emotion_turn_input, compute_turn_favor, latest_recent_turn_pair,
    skipped_complex_emotion, worldview_snippet_from_chunks, MiddleOutput, PreLlmOutput, STAGES,
};
use super::{
    apply_adult_output_boundary, resolve_fast_complex_emotion, should_use_stable_prompt_segments,
    ComplexEmotionOutput,
};
use crate::domain::chat_engine::chat_stage::ChatStage;
use crate::state::SessionCache;

pub(crate) async fn run_middle(
    ctx: &TurnContext<'_>,
    pre: &PreLlmOutput,
) -> TurnResult<MiddleOutput> {
    let state = ctx.state;
    let req = ctx.req;
    let role = ctx.role;
    let scene_id = ctx.scene_id;
    let mrid = ctx.mrid;
    let virtual_time_ms = ctx.virtual_time_ms;
    let immersive = ctx.immersive;
    let pl = &ctx.pl;
    let user_message = req.user_message.as_str();

    let deep_latch_active = ctx.runtime_snapshot.deep_latch_active.unwrap_or(false);

    let rules_estimate = STAGES
        .stage(ChatStage::EventEstimate, async {
            estimate_event_impact_rules_only(user_message, &pre.hints.user_emotion, None)
        })
        .await?;
    let this_turn_event = rules_estimate.event_type;

    let thinking = STAGES
        .stage(ChatStage::TurnThinkingRouter, async {
            Ok(resolve_turn_thinking(
                &state.host_profile,
                role,
                user_message,
                &pre.hints.emotion_result,
                &pre.memory.recent_events_for_event,
                this_turn_event,
                deep_latch_active,
            ))
        })
        .await?;
    tracing::debug!(
        target: "oclive_turn",
        mode = ?thinking.mode,
        reasons = ?thinking.reasons,
        "turn_thinking resolved"
    );

    let complex_emotion_input = build_complex_emotion_turn_input(
        mrid,
        scene_id,
        user_message,
        &pre.hints.emotion_result,
        pre.hints.prev_stored_narrative_hint.clone(),
        &pre.memory.recent_turns,
    );
    let complex_emotion_out: ComplexEmotionOutput =
        if thinking.skip_complex_emotion(&state.host_profile) {
            // Keep Fast turns local and deterministic, but do not erase the
            // emotion signal: portrait intensity and other downstream users
            // still need a meaningful mild/moderate value.
            resolve_fast_complex_emotion(&complex_emotion_input)
        } else {
            // B M1 slice 1: complex emotion resolution moved to post-LLM.
            // The main LLM declares [EMO]{...}[/EMO] in its reply; post_llm
            // parses it (plugin-chain / keep fallback when the marker is absent).
            skipped_complex_emotion()
        };

    let knowledge_limit = thinking.knowledge_retrieve_limit(&state.host_profile);
    let knowledge_chunks = role
        .knowledge_index
        .as_ref()
        .map(|idx| idx.retrieve(user_message, Some(scene_id), knowledge_limit))
        .unwrap_or_default();
    let knowledge_chunk_count = knowledge_chunks.len() as u32;

    let knowledge_augment_opt = {
        let aug = KnowledgeIndex::merge_event_augment(knowledge_chunks.as_slice());
        if aug.is_empty() {
            None
        } else {
            Some(aug)
        }
    };

    let use_event_llm = thinking.use_event_impact_llm(&state.host_profile);
    let estimate = if use_event_llm {
        STAGES
            .stage(
                ChatStage::EventEstimate,
                SlotRunner::estimate_event(
                    pl,
                    pre.memory.ollama_model.as_str(),
                    user_message,
                    &pre.hints.user_emotion,
                    &pre.memory.personality,
                    role.evolution_config.personality_source,
                    &pre.memory.recent_turns_for_event,
                    &pre.memory.recent_events_for_event,
                    knowledge_augment_opt.as_ref(),
                    true,
                ),
            )
            .await?
    } else {
        rules_estimate
    };
    let ai_event_type = estimate.event_type;
    let ai_impact_factor_final = estimate.impact_factor;
    let ai_event_confidence = estimate.confidence;

    let mut personality = pre.memory.personality.clone();
    if role.evolution_config.personality_source != PersonalitySource::Profile
        && thinking.applies_full_persistence(&state.host_profile, &ai_event_type)
    {
        personality = PersonalityEngine::evolve_by_event(
            personality,
            ai_impact_factor_final * pre.memory.event_runtime,
            &role.evolution_bounds,
        );
    }

    let (favor_delta, relation_after) = compute_turn_favor(
        pre,
        role,
        &ai_event_type,
        ai_impact_factor_final,
        ai_event_confidence,
    );
    let favor_scale = thinking.favor_delta_scale(&state.host_profile, &ai_event_type);
    let synthetic_adult_action = req.adult.as_ref().is_some_and(|adult| {
        adult.gates_open()
            && !matches!(
                adult.action,
                crate::models::dto::AdultInteractionAction::Message
            )
    });
    let (favor_delta, relation_after) = if favor_scale == 0.0 || synthetic_adult_action {
        (
            0.0,
            oclive_kernel_runtime::domain::relation_engine::RelationState::parse(
                pre.relation.relation_before.as_str(),
            ),
        )
    } else {
        (favor_delta, relation_after)
    };

    let memory_cap = thinking.memory_cap(&state.host_profile);
    let prompt_memories: Vec<Memory> = pre
        .memory
        .relevant
        .iter()
        .take(memory_cap)
        .cloned()
        .collect();

    let worldview_snippet = if thinking.mode == TurnThinkingMode::Fast {
        String::new()
    } else {
        worldview_snippet_from_chunks(knowledge_chunks.as_slice())
    };
    let scene_label = state.storage.scene_display_name_for_role(role, scene_id);
    let scene_detail_buf = if thinking.mode == TurnThinkingMode::Fast {
        String::new()
    } else {
        state
            .storage
            .scene_prompt_enrichment_for_role(role, scene_id)
    };
    let top_topic = if thinking.mode == TurnThinkingMode::Fast {
        None
    } else {
        SlotRunner::top_topic_hint(pl, role, scene_id)
    };
    let topic_line = top_topic
        .map(|t| format!("在「{}」下，你们可能会多聊「{}」相关的事。", scene_label, t))
        .unwrap_or_default();
    let life_context_line: String = if immersive && thinking.mode == TurnThinkingMode::Deep {
        role.life_schedule
            .as_ref()
            .and_then(|s| pick_life_state(virtual_time_ms, s))
            .map(|st| format_life_prompt_line(&st, false))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let host_overlay = if thinking.use_concise_prompt(&state.host_profile) {
        DISTRO_CONCISE_PROMPT_OVERLAY
    } else {
        ""
    };
    let host_state_hint = if thinking.mode == TurnThinkingMode::Fast {
        ""
    } else {
        state
            .host_profile
            .state_expression_hint(pre.relation.favorability_before)
    };
    // Prior-turn hint (pre_llm load) is independent of this turn's CE resolve (NARRATIVE_HINT_CONTRACT §2).
    let complex_hint = pre.hints.prev_stored_narrative_hint.as_str();
    let tier = resolve_model_tier(pre.memory.ollama_model.as_str());
    let persona_source = resolve_persona_source(tier, role, &state.host_profile);
    let persona_override = persona_override_for_source(role, persona_source);
    let (_, previous_assistant_reply) = latest_recent_turn_pair(&pre.memory.recent_turns);
    tracing::debug!(
        target: "oclive_turn",
        ?tier,
        ?persona_source,
        "persona_source resolved"
    );

    let continuity_prompt = match crate::domain::narrative_continuity::prompt_for_turn(
        state,
        role,
        ctx.srid,
        scene_id,
        virtual_time_ms,
        &ctx.runtime_snapshot,
        !ctx.is_staged(),
    )
    .await
    {
        Ok(value) => value,
        Err(error) => {
            tracing::warn!(
                target: "oclive_continuity",
                role_id = %ctx.srid,
                scene_id,
                error = %error,
                "narrative continuity prompt unavailable; continuing without it"
            );
            String::new()
        }
    };
    let adult_prompt =
        crate::domain::adult_interaction::prompt_section(role, scene_id, ctx.req.adult.as_ref())
            .unwrap_or_default();
    let staged_adult_continuity = if ctx.is_staged() {
        let prior: Vec<&str> = pre
            .memory
            .recent_turns
            .iter()
            .filter(|(user, _)| user == crate::domain::adult_stage::ADULT_CONTINUATION_INPUT)
            .map(|(_, assistant)| assistant.as_str())
            .rev()
            .take(8)
            .collect();
        prior
            .into_iter()
            .rev()
            .enumerate()
            .map(|(index, transcript)| format!("前一拍 {}：{}", index + 1, transcript.trim()))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        String::new()
    };
    let mut extra_sections: Vec<PromptExtraSection<'_>> = role
        .pack_prompt_extra_sections
        .iter()
        .map(|s| PromptExtraSection {
            title: s.title.as_str(),
            body: s.body.as_str(),
        })
        .collect();
    let reply_mode_instruction = effective_reply_mode(role)
        .map(|cfg| reply_output_format_instruction(cfg.segments, cfg.separator.as_str()));
    if let Some(body) = reply_mode_instruction.as_deref() {
        extra_sections.push(PromptExtraSection {
            title: "输出格式要求",
            body,
        });
    }
    if !continuity_prompt.is_empty() {
        extra_sections.push(PromptExtraSection {
            title: "行动连续性状态",
            body: continuity_prompt.as_str(),
        });
    }
    if !adult_prompt.is_empty() {
        extra_sections.push(PromptExtraSection {
            title: crate::domain::adult_interaction::prompt_title(),
            body: adult_prompt.as_str(),
        });
    }
    if !staged_adult_continuity.is_empty() {
        extra_sections.push(PromptExtraSection {
            title: "成人互动前拍连续性（只作为上下文，不得代写用户）",
            body: staged_adult_continuity.as_str(),
        });
    }

    let prompt_input = PromptInput {
        role,
        personality: &personality,
        memories: &prompt_memories,
        user_input: user_message,
        user_emotion: pre.hints.user_emotion_prompt.as_str(),
        user_relation_id: pre.relation.user_relation_key.as_str(),
        relation_hint: pre.relation.relation_hint.as_str(),
        user_identity_template: pre.relation.user_identity_template.as_str(),
        user_identity_id: pre.relation.user_identity_id.as_str(),
        relation_before: pre.relation.relation_before.as_str(),
        favorability_before: pre.relation.favorability_before,
        relation_preview: relation_after.as_str(),
        favorability_preview: (pre.relation.favorability_before + favor_delta).clamp(0.0, 100.0),
        event_type: &ai_event_type,
        impact_factor: ai_impact_factor_final,
        scene_label: &scene_label,
        scene_detail: scene_detail_buf.as_str(),
        topic_hint_line: &topic_line,
        life_context_line: life_context_line.as_str(),
        worldview_snippet: worldview_snippet.as_str(),
        mutable_personality: pre.memory.mutable_for_prompt.as_str(),
        ephemeral_personality: pre.memory.ephemeral_for_prompt.as_str(),
        reply_quality_anchor: effective_reply_quality_anchor(role),
        previous_complex_emotion_narrative_hint: complex_hint,
        host_prompt_overlay: host_overlay,
        host_state_expression_hint: host_state_hint,
        relation_transition_hint: pre.relation.relation_transition_hint.as_str(),
        extra_sections: &extra_sections,
        persona_override,
        previous_assistant_reply: previous_assistant_reply.as_str(),
    };

    let llm_supports_prefix_cache = SlotRunner::primary_llm(pl).supports_prefix_cache();
    let use_prefix_segments = should_use_stable_prompt_segments(
        prompt_prefix_cache_effective(&state.host_profile),
        llm_supports_prefix_cache,
        ctx.effective_backends.prompt,
    );

    let (
        prompt,
        prompt_stable_hash,
        prompt_stable_len,
        prefix_cache_expected_hit,
        use_ollama_prefix_opts,
    ) = if use_prefix_segments {
        let segments = STAGES
            .stage(ChatStage::BuildPrompt, async {
                SlotRunner::build_prompt_segments(pl, &prompt_input)
            })
            .await?;
        let stable_hash = hash_stable_prefix(&segments.stable_prefix);
        let stable_len = segments.stable_len();
        let mode_key = match thinking.mode {
            TurnThinkingMode::Fast => "fast",
            TurnThinkingMode::Deep => "deep",
        };
        let persona_key = match persona_source {
            PersonaSource::PersonaCapsule => "persona_capsule",
            PersonaSource::FullCore => "full_core",
        };
        let cache_key = SessionCache::prefix_cache_key(
            ctx.srid,
            pre.memory.ollama_model.as_str(),
            format!("{mode_key}:{persona_key}").as_str(),
            scene_id,
            pre.relation.user_identity_id.as_str(),
        );
        let expected_hit =
            state
                .session_cache
                .observe_prefix_cache(cache_key, stable_hash, stable_len);
        tracing::debug!(
            target: "oclive_turn",
            prefix_hash = stable_hash,
            stable_len,
            cache_expected_hit = expected_hit,
            mode = mode_key,
            "prompt prefix cache"
        );
        (
            segments.full(),
            Some(stable_hash),
            Some(stable_len),
            Some(expected_hit),
            true,
        )
    } else {
        let prompt = STAGES
            .stage(ChatStage::BuildPrompt, async {
                SlotRunner::build_prompt(pl, &prompt_input)
            })
            .await?;
        (prompt, None, None, None, false)
    };
    let prompt = apply_adult_output_boundary(prompt, adult_prompt.as_str());

    Ok(MiddleOutput {
        turn_thinking: thinking,
        complex_emotion_out,
        knowledge_chunk_count,
        ai_event_type,
        ai_impact_factor_final,
        ai_event_confidence,
        personality,
        prompt,
        favor_delta,
        relation_after,
        prompt_stable_hash,
        prompt_stable_len,
        prefix_cache_expected_hit,
        use_ollama_prefix_opts,
    })
}
