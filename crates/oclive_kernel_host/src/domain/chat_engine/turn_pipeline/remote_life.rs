//! Remote-life turn path: skipped complex emotion, stub event, remote prompt build.

use crate::domain::personality_engine::PersonalityEngine;
use crate::domain::remote_life_prompt::build_remote_life_prompt;
use crate::models::{EventType, PersonalitySource};

use super::super::turn_context::TurnContext;
use super::super::turn_error::TurnResult;
use super::{
    compute_turn_favor, skipped_complex_emotion, worldview_snippet_from_chunks, MiddleOutput,
    PreLlmOutput,
};
use crate::domain::life_schedule::{format_life_prompt_line, pick_life_state};

pub(crate) async fn run_middle(
    ctx: &TurnContext<'_>,
    pre: &PreLlmOutput,
) -> TurnResult<MiddleOutput> {
    let state = ctx.state;
    let req = ctx.req;
    let role = ctx.role;
    let scene_id = ctx.scene_id;
    let virtual_time_ms = ctx.virtual_time_ms;
    let user_message = req.user_message.as_str();

    let complex_emotion_out = skipped_complex_emotion();

    let character_scene_id = ctx.character_scene_id.as_deref().unwrap_or("default");
    let knowledge_chunks = role
        .knowledge_index
        .as_ref()
        .map(|idx| idx.retrieve(user_message, Some(character_scene_id), 8))
        .unwrap_or_default();
    let knowledge_chunk_count = knowledge_chunks.len() as u32;

    let ai_event_type = EventType::Ignore;
    let ai_impact_factor_final = 0.0_f64;
    let ai_event_confidence = 0.0_f32;

    let mut personality = pre.personality.clone();
    if role.evolution_config.personality_source != PersonalitySource::Profile {
        personality = PersonalityEngine::evolve_by_event(
            personality,
            ai_impact_factor_final * pre.event_runtime,
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

    let char_label = state
        .storage
        .scene_display_name_for_role(role, character_scene_id);
    let user_label = state.storage.scene_display_name_for_role(role, scene_id);
    let away_material =
        state
            .storage
            .away_life_material_for_role(role, character_scene_id, scene_id);
    let vt_label = if virtual_time_ms > 0 {
        chrono::DateTime::from_timestamp_millis(virtual_time_ms)
            .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "未设定".to_string())
    } else {
        "未设定".to_string()
    };
    let life_schedule_line: String = role
        .life_schedule
        .as_ref()
        .and_then(|s| pick_life_state(virtual_time_ms, s))
        .map(|st| format_life_prompt_line(&st, true))
        .unwrap_or_default();
    let remote_mutable = if role.evolution_config.personality_source == PersonalitySource::Profile {
        pre.mutable_for_prompt.as_str()
    } else {
        ""
    };
    let prompt = build_remote_life_prompt(
        role,
        away_material.as_str(),
        char_label.as_str(),
        user_label.as_str(),
        user_message,
        pre.favorability_before,
        pre.relation_before.as_str(),
        vt_label.as_str(),
        life_schedule_line.as_str(),
        worldview_snippet.as_str(),
        remote_mutable,
    );

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
