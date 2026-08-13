use super::*;

fn fast_input(message: &str) -> ComplexEmotionInput {
    ComplexEmotionInput {
        role_id: "mumu".into(),
        scene_id: "home".into(),
        user_message: message.into(),
        bot_reply: String::new(),
        recent_dialogue_summary: None,
        previous_narrative_hint: String::new(),
        user_valence: Some(0.0),
        user_dominance: Some(0.0),
        previous_user_message: None,
    }
}

#[test]
fn fast_complex_emotion_keeps_a_mild_baseline() {
    let output = resolve_fast_complex_emotion(&fast_input("你好"));
    assert_eq!(output.source, "turn_thinking_fast_builtin_intensity");
    assert!((output.intensity - 0.25).abs() < f64::EPSILON);
}

#[test]
fn fast_complex_emotion_can_reach_moderate_for_known_patterns() {
    let output = resolve_fast_complex_emotion(&fast_input("随便吧"));
    assert!(output.pattern.is_none());
    assert!((output.intensity - 0.5).abs() < f64::EPSILON);
}
#[test]
fn fast_complex_emotion_full_field_shape_matches_noop_baseline() {
    // M2 slice 2: the fast path must fill every ComplexEmotionOutput field
    // deterministically - narrative/label fields stay empty (Noop defaults
    // in backend_registry are the field baseline), only intensity carries
    // the turn signal, and the source marks the fast producer.
    let output = resolve_fast_complex_emotion(&fast_input("你好"));
    assert!(output.narrative_hint.is_empty());
    assert!(output.labels.is_empty());
    assert_eq!(output.pattern, None);
    assert_eq!(output.confidence, 0.0);
    assert!((output.intensity - 0.25).abs() < f64::EPSILON);
    assert_eq!(output.dissonance_score, 0.0);
    assert!(!output.degraded_to_builtin);
    assert_eq!(output.extension, None);
}

#[test]
fn stable_prompt_segments_require_builtin_prompt_and_cacheable_llm() {
    assert!(should_use_stable_prompt_segments(
        true,
        true,
        PromptBackend::Builtin
    ));
    assert!(!should_use_stable_prompt_segments(
        false,
        true,
        PromptBackend::Builtin
    ));
    assert!(!should_use_stable_prompt_segments(
        true,
        false,
        PromptBackend::Builtin
    ));
    assert!(!should_use_stable_prompt_segments(
        true,
        true,
        PromptBackend::Directory
    ));
    assert!(!should_use_stable_prompt_segments(
        true,
        true,
        PromptBackend::Remote
    ));
}

#[test]
fn adult_output_contract_is_the_last_prompt_instruction() {
    let prompt = apply_adult_output_boundary(
        "【输出边界】只输出当前角色本人的这一轮台词。".to_string(),
        "adult enabled",
    );
    assert!(prompt.ends_with(crate::domain::adult_interaction::output_boundary()));
    assert!(prompt.find("只输出当前角色") < prompt.find("本轮最终输出契约"));
}
