//! Co-present turn path: fast emotion helpers and module index.

use crate::domain::complex_emotion::{
    BuiltinKeywordComplexEmotionProvider, ComplexEmotionInput, ComplexEmotionOutput,
    FAST_INTENSITY_SOURCE,
};
use crate::models::PromptBackend;

mod run_middle;
#[cfg(test)]
mod tests;

pub(crate) use run_middle::run_middle;

fn resolve_fast_complex_emotion(input: &ComplexEmotionInput) -> ComplexEmotionOutput {
    let inferred = BuiltinKeywordComplexEmotionProvider.resolve_turn_inner(input);
    ComplexEmotionOutput {
        source: FAST_INTENSITY_SOURCE.into(),
        narrative_hint: String::new(),
        labels: vec![],
        pattern: None,
        confidence: 0.0,
        intensity: inferred.intensity,
        dissonance_score: 0.0,
        degraded_to_builtin: false,
        extension: None,
    }
}

fn should_use_stable_prompt_segments(
    prompt_prefix_cache_enabled: bool,
    llm_supports_prefix_cache: bool,
    prompt_backend: PromptBackend,
) -> bool {
    prompt_prefix_cache_enabled
        && llm_supports_prefix_cache
        && matches!(prompt_backend, PromptBackend::Builtin)
}

fn apply_adult_output_boundary(mut prompt: String, adult_prompt: &str) -> String {
    if !adult_prompt.is_empty() {
        prompt.push_str("\n\n");
        prompt.push_str(crate::domain::adult_interaction::output_boundary());
    }
    prompt
}
