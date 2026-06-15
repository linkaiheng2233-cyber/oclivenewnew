//! Co-present turn path: complex emotion, event estimate, prompt build.

use crate::domain::complex_emotion::ComplexEmotionOutput;
use crate::domain::host_profile::{PromptProfile, DISTRO_CONCISE_PROMPT_OVERLAY};
use crate::domain::life_schedule::{format_life_prompt_line, pick_life_state};
use crate::domain::personality_engine::PersonalityEngine;
use crate::domain::prompt_builder::{effective_reply_quality_anchor, PromptInput};
use crate::domain::slot_runner::SlotRunner;
use crate::models::knowledge::KnowledgeIndex;
use crate::models::PersonalitySource;

use super::super::turn_context::TurnContext;
use super::super::turn_error::TurnResult;
use super::{
    build_complex_emotion_turn_input, compute_turn_favor, worldview_snippet_from_chunks,
    MiddleOutput, PreLlmOutput, STAGES,
};
use crate::domain::chat_engine::chat_stage::ChatStage;

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

    let complex_emotion_input = build_complex_emotion_turn_input(
        mrid,
        scene_id,
        user_message,
        &pre.hints.emotion_result,
        pre.hints.prev_stored_narrative_hint.clone(),
        &pre.memory.recent_turns,
    );
    let complex_emotion_out: ComplexEmotionOutput = if state.host_profile.skip_complex_emotion {
        ComplexEmotionOutput {
            source: "host_skipped".into(),
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

    let knowledge_chunks = role
        .knowledge_index
        .as_ref()
        .map(|idx| idx.retrieve(user_message, Some(scene_id), 8))
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
    let estimate = STAGES
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
            ),
        )
        .await?;
    let ai_event_type = estimate.event_type;
    let ai_impact_factor_final = estimate.impact_factor;
    let ai_event_confidence = estimate.confidence;

    let mut personality = pre.memory.personality.clone();
    if role.evolution_config.personality_source != PersonalitySource::Profile {
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

    let worldview_snippet = worldview_snippet_from_chunks(knowledge_chunks.as_slice());
    let scene_label = state.storage.scene_display_name_for_role(role, scene_id);
    let scene_detail_buf = state
        .storage
        .scene_prompt_enrichment_for_role(role, scene_id);
    let top_topic = SlotRunner::top_topic_hint(pl, role, scene_id);
    let topic_line = top_topic
        .map(|t| format!("在「{}」下，你们可能会多聊「{}」相关的事。", scene_label, t))
        .unwrap_or_default();
    let life_context_line: String = if immersive {
        role.life_schedule
            .as_ref()
            .and_then(|s| pick_life_state(virtual_time_ms, s))
            .map(|st| format_life_prompt_line(&st, false))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let host_overlay = if state.host_profile.prompt_profile == PromptProfile::Concise {
        DISTRO_CONCISE_PROMPT_OVERLAY
    } else {
        ""
    };
    let host_state_hint = state
        .host_profile
        .state_expression_hint(pre.relation.favorability_before);
    let prompt = STAGES
        .stage(ChatStage::BuildPrompt, async {
            SlotRunner::build_prompt(
                pl,
                &PromptInput {
                    role,
                    personality: &personality,
                    memories: &pre.memory.relevant,
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
                    reply_quality_anchor: effective_reply_quality_anchor(role),
                    previous_complex_emotion_narrative_hint: pre
                        .hints
                        .prev_stored_narrative_hint
                        .as_str(),
                    host_prompt_overlay: host_overlay,
                    host_state_expression_hint: host_state_hint,
                    relation_transition_hint: pre.relation.relation_transition_hint.as_str(),
                    extra_sections: &[],
                },
            )
        })
        .await?;

    Ok(MiddleOutput {
        complex_emotion_out,
        knowledge_chunk_count,
        ai_event_type,
        ai_impact_factor_final,
        ai_event_confidence,
        personality,
        prompt,
        favor_delta,
        relation_after,
    })
}
