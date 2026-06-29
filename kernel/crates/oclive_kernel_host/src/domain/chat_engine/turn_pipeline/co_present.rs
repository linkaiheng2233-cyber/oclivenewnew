//! Co-present turn path: complex emotion, event estimate, prompt build.

use crate::domain::complex_emotion::ComplexEmotionOutput;
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
use crate::domain::slot_runner::SlotRunner;
use crate::domain::turn_thinking::{resolve_turn_thinking, TurnThinkingMode};
use crate::models::knowledge::KnowledgeIndex;
use crate::models::Memory;
use crate::models::PersonalitySource;

use super::super::turn_context::TurnContext;
use super::super::turn_error::TurnResult;
use super::{
    build_complex_emotion_turn_input, compute_turn_favor, latest_recent_turn_pair,
    worldview_snippet_from_chunks, MiddleOutput, PreLlmOutput, STAGES,
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
            ComplexEmotionOutput {
                source: "turn_thinking_fast".into(),
                narrative_hint: String::new(),
                labels: vec![],
                pattern: None,
                confidence: 0.0,
                intensity: 0.0,
                dissonance_score: 0.0,
                degraded_to_builtin: false,
                extension: None,
            }
        } else {
            STAGES
                .stage(ChatStage::ComplexEmotionResolveTurn, async {
                    SlotRunner::resolve_complex_emotion(pl, &complex_emotion_input)
                })
                .await?
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
    let (favor_delta, relation_after) = if favor_scale == 0.0 {
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
    let complex_hint = if thinking.skip_complex_emotion(&state.host_profile) {
        ""
    } else {
        pre.hints.prev_stored_narrative_hint.as_str()
    };
    let tier = resolve_model_tier(pre.memory.ollama_model.as_str());
    let persona_source = resolve_persona_source(tier, thinking.mode, role, &state.host_profile);
    let persona_override = persona_override_for_source(role, persona_source);
    let (_, previous_assistant_reply) = latest_recent_turn_pair(&pre.memory.recent_turns);
    tracing::debug!(
        target: "oclive_turn",
        ?tier,
        ?persona_source,
        "persona_source resolved"
    );

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
        extra_sections: &[],
        persona_override,
        previous_assistant_reply: previous_assistant_reply.as_str(),
    };

    let llm_supports_prefix_cache = SlotRunner::primary_llm(pl).supports_prefix_cache();
    let use_prefix_segments = thinking.mode == TurnThinkingMode::Deep
        && prompt_prefix_cache_effective(&state.host_profile)
        && llm_supports_prefix_cache;

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
        let cache_key = SessionCache::prefix_cache_key(
            ctx.srid,
            pre.memory.ollama_model.as_str(),
            match persona_source {
                PersonaSource::DeepCapsule => "deep_capsule",
                PersonaSource::FullCore => "full_core",
            },
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
            "deep prompt prefix cache"
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
