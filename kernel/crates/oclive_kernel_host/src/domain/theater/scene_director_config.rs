//! Theater scene director tunables (env overrides; small-model defaults).

const DEFAULT_RIPPLE_MAX_BEATS: u32 = 12;
const DEFAULT_SCENE_LLM_TIMEOUT_SECS: u64 = 25;
/// Cast rewrite runs two LLM attempts; small models need more time per attempt than ripple poke.
const DEFAULT_CAST_REWRITE_LLM_TIMEOUT_SECS: u64 = 45;
const DEFAULT_CAST_REWRITE_MIN_BEATS: u32 = 4;

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
    parse_env_u64(
        "OCLIVE_THEATER_SCENE_TIMEOUT_SECS",
        DEFAULT_SCENE_LLM_TIMEOUT_SECS,
    )
    .clamp(5, 120)
}

/// Per-attempt LLM timeout for cast rewrite (`OCLIVE_THEATER_CAST_REWRITE_TIMEOUT_SECS`, default 45).
#[must_use]
pub fn cast_rewrite_llm_timeout_secs() -> u64 {
    parse_env_u64(
        "OCLIVE_THEATER_CAST_REWRITE_TIMEOUT_SECS",
        DEFAULT_CAST_REWRITE_LLM_TIMEOUT_SECS,
    )
    .clamp(10, 180)
}

const DEFAULT_PATCH_MAX_LINES: u32 = 4;

fn parse_env_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .map(|s| {
            let t = s.trim().to_ascii_lowercase();
            t == "1" || t == "true" || t == "yes"
        })
        .unwrap_or(default)
}

/// Max lines in a poke patch segment (`OCLIVE_THEATER_PATCH_MAX_LINES`, default 4).
#[must_use]
pub fn patch_max_lines() -> u32 {
    parse_env_u32("OCLIVE_THEATER_PATCH_MAX_LINES", DEFAULT_PATCH_MAX_LINES).clamp(2, 8)
}

/// Whether patch mode may append one partner reaction line (`OCLIVE_THEATER_PATCH_PARTNER_REPLY`, default true).
#[must_use]
pub fn patch_partner_reply_enabled() -> bool {
    parse_env_bool("OCLIVE_THEATER_PATCH_PARTNER_REPLY", true)
}

/// Minimum beats accepted from cast-rewrite LLM output (`OCLIVE_THEATER_CAST_REWRITE_MIN_BEATS`, default 4).
#[must_use]
pub fn cast_rewrite_min_beats() -> u32 {
    parse_env_u32(
        "OCLIVE_THEATER_CAST_REWRITE_MIN_BEATS",
        DEFAULT_CAST_REWRITE_MIN_BEATS,
    )
    .clamp(3, 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cast_rewrite_timeout_defaults_and_clamps() {
        assert_eq!(cast_rewrite_llm_timeout_secs(), 45);
        assert_eq!(cast_rewrite_min_beats(), 4);
    }
}
