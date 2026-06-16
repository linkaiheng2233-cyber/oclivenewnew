//! Theater scene director tunables (env overrides; small-model defaults).

const DEFAULT_RIPPLE_MAX_BEATS: u32 = 12;
const DEFAULT_SCENE_LLM_TIMEOUT_SECS: u64 = 25;

fn parse_env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(default)
}

fn parse_env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(default)
}

/// Max beats the LLM may output for the ripple zone (`OCLIVE_THEATER_RIPPLE_MAX_BEATS`, default 12).
#[must_use]
pub fn ripple_max_beats() -> u32 {
    parse_env_u32("OCLIVE_THEATER_RIPPLE_MAX_BEATS", DEFAULT_RIPPLE_MAX_BEATS).clamp(4, 64)
}

/// Per-attempt LLM timeout for scene director (`OCLIVE_THEATER_SCENE_TIMEOUT_SECS`, default 25).
#[must_use]
pub fn scene_llm_timeout_secs() -> u64 {
    parse_env_u64("OCLIVE_THEATER_SCENE_TIMEOUT_SECS", DEFAULT_SCENE_LLM_TIMEOUT_SECS).clamp(5, 120)
}
