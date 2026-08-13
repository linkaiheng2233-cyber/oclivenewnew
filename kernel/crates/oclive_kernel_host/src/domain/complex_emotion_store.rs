//! Complex emotion `narrative_hint` persistence and session cache (injected into main Prompt one turn later).

use crate::domain::repository::ComplexEmotionHintStore;
use crate::error::Result;
use crate::state::{AppState, SessionCache};
use chrono::{DateTime, Duration, Utc};
use oclive_validation::{slot_registry_instances_sorted, SlotRegistryEntry};
use std::collections::BTreeMap;

/// Clear `narrative_hint` on read when it has not been updated within this many hours. May move to role pack / settings later.
pub const COMPLEX_EMOTION_HINT_TTL_HOURS: i64 = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RoleComplexEmotionBackend {
    Disabled,
    Builtin,
    Plugin,
}

impl RoleComplexEmotionBackend {
    #[must_use]
    pub(crate) const fn persists_hint(self) -> bool {
        matches!(self, Self::Builtin | Self::Plugin)
    }
}

/// Returns the effective last-wins `complex_emotion` backend for this role.
/// Omitted and explicit `none` entries both disable hint reads and writes.
#[must_use]
pub(crate) fn role_complex_emotion_backend(
    slot_registry: Option<&BTreeMap<String, SlotRegistryEntry>>,
) -> RoleComplexEmotionBackend {
    let Some(registry) = slot_registry else {
        return RoleComplexEmotionBackend::Disabled;
    };
    slot_registry_instances_sorted(registry, "complex_emotion")
        .last()
        .map_or(
            RoleComplexEmotionBackend::Disabled,
            |(_, entry)| match entry.backend.trim() {
                "builtin" => RoleComplexEmotionBackend::Builtin,
                "remote" | "directory" => RoleComplexEmotionBackend::Plugin,
                _ => RoleComplexEmotionBackend::Disabled,
            },
        )
}

fn parse_updated_at(raw: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

#[must_use]
pub fn is_complex_emotion_hint_expired(updated_at: &str, now: DateTime<Utc>) -> bool {
    let Some(ts) = parse_updated_at(updated_at) else {
        return true;
    };
    now.signed_duration_since(ts) > Duration::hours(COMPLEX_EMOTION_HINT_TTL_HOURS)
}

/// Return from session cache on hit; otherwise load from DB (with TTL delete) and backfill cache.
///
/// # Errors
///
/// Returns [`crate::error::AppError`] on DB read failure (caller may degrade to empty in `pre_llm`).
pub async fn load_stored_narrative_hint(state: &AppState, srid: &str) -> Result<String> {
    load_stored_narrative_hint_from_parts(&state.session_cache, state.db_manager.as_ref(), srid)
        .await
}

pub(crate) async fn load_stored_narrative_hint_from_parts(
    session_cache: &SessionCache,
    db: &dyn ComplexEmotionHintStore,
    srid: &str,
) -> Result<String> {
    if session_cache.has_stored_complex_emotion_narrative_hint(srid) {
        return Ok(session_cache.stored_complex_emotion_narrative_hint(srid));
    }

    let Some((hint, updated_at)) = db.get_complex_emotion_hint(srid).await? else {
        return Ok(String::new());
    };

    if is_complex_emotion_hint_expired(&updated_at, Utc::now()) {
        if let Err(e) = db.delete_complex_emotion_hint(srid).await {
            tracing::warn!(
                target: "oclive_complex_emotion",
                role_id = %srid,
                error = %e,
                "delete expired complex_emotion_hint failed"
            );
        }
        return Ok(String::new());
    }

    session_cache.set_stored_complex_emotion_narrative_hint(srid, hint.clone());
    Ok(hint)
}

/// Write session cache and best-effort persist to SQLite; DB failure is logged only.
pub async fn persist_stored_narrative_hint(state: &AppState, srid: &str, hint: String) {
    persist_stored_narrative_hint_to_parts(
        &state.session_cache,
        state.db_manager.as_ref(),
        srid,
        hint,
    )
    .await;
}

pub(crate) async fn persist_stored_narrative_hint_to_parts(
    session_cache: &SessionCache,
    db: &dyn ComplexEmotionHintStore,
    srid: &str,
    hint: String,
) {
    session_cache.set_stored_complex_emotion_narrative_hint(srid, hint.clone());
    let trimmed = hint.trim();
    if trimmed.is_empty() {
        if let Err(e) = db.delete_complex_emotion_hint(srid).await {
            tracing::warn!(
                target: "oclive_complex_emotion",
                role_id = %srid,
                error = %e,
                "delete complex_emotion_hint failed"
            );
        }
        return;
    }
    let updated_at = Utc::now().to_rfc3339();
    if let Err(e) = db
        .set_complex_emotion_hint(srid, trimmed, updated_at.as_str())
        .await
    {
        tracing::warn!(
            target: "oclive_complex_emotion",
            role_id = %srid,
            error = %e,
            "set_complex_emotion_hint failed"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::test_db;
    use crate::state::SessionCache;

    async fn mem_db() -> impl ComplexEmotionHintStore {
        test_db::mem_db_manager().await
    }

    #[test]
    fn expired_when_beyond_ttl() {
        let old = (Utc::now() - Duration::hours(COMPLEX_EMOTION_HINT_TTL_HOURS + 1)).to_rfc3339();
        assert!(is_complex_emotion_hint_expired(&old, Utc::now()));
        let recent = (Utc::now() - Duration::hours(1)).to_rfc3339();
        assert!(!is_complex_emotion_hint_expired(&recent, Utc::now()));
    }

    #[tokio::test]
    async fn load_from_db_after_cache_miss() {
        let db = mem_db().await;
        let cache = SessionCache::new();
        let srid = "role_a";
        let now = Utc::now().to_rfc3339();
        db.set_complex_emotion_hint(srid, "用户可能缺乏兴致", &now)
            .await
            .expect("set");
        let hint = load_stored_narrative_hint_from_parts(&cache, &db, srid)
            .await
            .expect("load");
        assert!(hint.contains("用户可能缺乏兴致"));
        assert!(cache.has_stored_complex_emotion_narrative_hint(srid));
    }

    #[tokio::test]
    async fn load_deletes_expired_row() {
        let db = mem_db().await;
        let cache = SessionCache::new();
        let srid = "role_b";
        let old = (Utc::now() - Duration::hours(COMPLEX_EMOTION_HINT_TTL_HOURS + 2)).to_rfc3339();
        db.set_complex_emotion_hint(srid, "stale hint", &old)
            .await
            .expect("set");
        let hint = load_stored_narrative_hint_from_parts(&cache, &db, srid)
            .await
            .expect("load");
        assert!(hint.is_empty());
        assert!(db
            .get_complex_emotion_hint(srid)
            .await
            .expect("get")
            .is_none());
    }
}
