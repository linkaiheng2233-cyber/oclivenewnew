//! 复杂情感 `narrative_hint`：SQLite 持久化、重启后回填缓存、TTL 过期。

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use chrono::{Duration, Utc};
use oclive_kernel_host::domain::complex_emotion_store::{
    load_stored_narrative_hint, persist_stored_narrative_hint, COMPLEX_EMOTION_HINT_TTL_HOURS,
};
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::state::AppState;
use std::sync::Arc;

async fn in_memory_state() -> AppState {
    let roles_dir = common::roles_dir();
    AppState::new_in_memory_with_llm(Arc::new(MockLlmClient { reply: "ok".into() }), roles_dir)
        .await
        .expect("state")
}

#[tokio::test]
async fn persist_survives_session_cache_clear() {
    let state = in_memory_state().await;
    let srid = "mumu";
    state
        .db_manager
        .ensure_role_runtime(srid)
        .await
        .expect("runtime");

    persist_stored_narrative_hint(&state, srid, "用户可能缺乏兴致".to_string()).await;

    state
        .session_cache
        .clear_complex_emotion_narrative_hint_cache(srid);

    let hint = load_stored_narrative_hint(&state, srid)
        .await
        .expect("load");
    assert!(
        hint.contains("用户可能缺乏兴致"),
        "expected DB-backed hint after cache clear, got: {hint:?}"
    );
}

#[tokio::test]
async fn expired_hint_cleared_on_load() {
    let state = in_memory_state().await;
    let srid = "mumu";
    state
        .db_manager
        .ensure_role_runtime(srid)
        .await
        .expect("runtime");

    let old = (Utc::now() - Duration::hours(COMPLEX_EMOTION_HINT_TTL_HOURS + 2)).to_rfc3339();
    state
        .db_manager
        .set_complex_emotion_hint(srid, "stale narrative", &old)
        .await
        .expect("set");

    let hint = load_stored_narrative_hint(&state, srid)
        .await
        .expect("load");
    assert!(hint.is_empty(), "expired hint should not be returned");
    assert!(
        state
            .db_manager
            .get_complex_emotion_hint(srid)
            .await
            .expect("get")
            .is_none(),
        "expired row should be deleted"
    );
}
