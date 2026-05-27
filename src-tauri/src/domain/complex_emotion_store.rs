//! 复杂情感 `narrative_hint` 持久化与会话缓存（滞后一轮注入主 Prompt）。

use crate::error::Result;
use crate::infrastructure::db::DbManager;
use crate::state::{AppState, SessionCache};
use chrono::{DateTime, Duration, Utc};

/// `narrative_hint` 在读取时若超过该小时数未更新则清除。后续可迁至角色包 / settings。
pub const COMPLEX_EMOTION_HINT_TTL_HOURS: i64 = 24;

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

/// 会话缓存命中则直接返回；否则从 DB 加载（含 TTL 删除），命中后回填缓存。
///
/// # Errors
///
/// DB 读失败时返回 [`crate::error::AppError`]（调用方在 `pre_llm` 可降级为空）。
pub async fn load_stored_narrative_hint(state: &AppState, srid: &str) -> Result<String> {
    load_stored_narrative_hint_from_parts(
        &state.session_cache,
        state.db_manager.as_ref(),
        srid,
    )
    .await
}

pub(crate) async fn load_stored_narrative_hint_from_parts(
    session_cache: &SessionCache,
    db: &DbManager,
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

/// 写入会话缓存并尽力持久化到 SQLite；DB 失败仅打日志。
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
    db: &DbManager,
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
    use crate::infrastructure::sqlite_pool;
    use crate::state::SessionCache;

    async fn mem_db() -> DbManager {
        let pool = sqlite_pool::connect_memory().await.expect("pool");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        DbManager::new(pool)
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
        let hint =
            load_stored_narrative_hint_from_parts(&cache, &db, srid)
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
        let hint =
            load_stored_narrative_hint_from_parts(&cache, &db, srid)
                .await
                .expect("load");
        assert!(hint.is_empty());
        assert!(
            db.get_complex_emotion_hint(srid)
                .await
                .expect("get")
                .is_none()
        );
    }
}
