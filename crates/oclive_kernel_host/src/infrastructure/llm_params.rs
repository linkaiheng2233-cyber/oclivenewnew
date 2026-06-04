//! Ollama sampling params for main-chat and tag-style calls; overridable via env vars, with defaults matching the historical implementation.

fn parse_env_f32(key: &str, default: f32) -> f32 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.trim().parse::<f32>().ok())
        .unwrap_or(default)
}

/// Main-chat generation: `OCLIVE_LLM_TEMPERATURE` (default `0.8`), `OCLIVE_LLM_TOP_P` (default `0.9`).
#[must_use]
pub fn main_chat_options() -> (Option<f32>, Option<f32>) {
    (
        Some(parse_env_f32("OCLIVE_LLM_TEMPERATURE", 0.8)),
        Some(parse_env_f32("OCLIVE_LLM_TOP_P", 0.9)),
    )
}

/// Low-temperature short outputs (portrait tags, structured fields, etc.): `OCLIVE_LLM_TAG_TEMPERATURE` (default `0.28`), `OCLIVE_LLM_TAG_TOP_P` (default `0.85`).
#[must_use]
pub fn tag_task_options() -> (Option<f32>, Option<f32>) {
    (
        Some(parse_env_f32("OCLIVE_LLM_TAG_TEMPERATURE", 0.28)),
        Some(parse_env_f32("OCLIVE_LLM_TAG_TOP_P", 0.85)),
    )
}
