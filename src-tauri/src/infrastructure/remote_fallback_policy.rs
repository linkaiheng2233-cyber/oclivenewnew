//! Whether to silently fall back to the builtin when a remote HTTP plugin fails (aligned with `app_settings.remote_fallback_to_builtin` and `OCLIVE_REMOTE_FALLBACK_TO_BUILTIN`).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const ENV_REMOTE_FALLBACK: &str = "OCLIVE_REMOTE_FALLBACK_TO_BUILTIN";

#[must_use]
pub fn new_remote_fallback_switch(initial: bool) -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(initial))
}

/// Raw value read from the database → whether fallback is allowed (missing / unparsable → `true`).
#[must_use]
pub fn remote_fallback_from_db_value(raw: Option<String>) -> bool {
    !matches!(
        raw.as_deref().map(|s| s.trim().to_ascii_lowercase()),
        Some(s) if matches!(s.as_str(), "0" | "false" | "no" | "off")
    )
}

/// Environment variable override (if set): `0`/`false`/`no`/`off` → fallback not allowed; `1`/`true`/`yes`/`on` → allowed.
#[must_use]
pub fn remote_fallback_env_override() -> Option<bool> {
    std::env::var(ENV_REMOTE_FALLBACK).ok().and_then(|v| {
        let t = v.trim().to_ascii_lowercase();
        match t.as_str() {
            "0" | "false" | "no" | "off" => Some(false),
            "1" | "true" | "yes" | "on" => Some(true),
            _ => None,
        }
    })
}

#[inline]
pub fn remote_fallback_load(flag: &Arc<AtomicBool>) -> bool {
    flag.load(Ordering::Relaxed)
}
