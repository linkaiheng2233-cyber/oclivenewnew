//! Saving an Ollama URL must update the shared production fallback, not only tags UI.
#![allow(clippy::expect_used)]
use axum::{
    body::{Body, Bytes},
    response::Response,
    routing::post,
    Json, Router,
};
use oclive_kernel_host::{
    domain::{
        host_profile::HostProfile,
        user_llm_env::{apply_user_llm_env, KEY_OLLAMA_BASE},
    },
    state::AppStateBuilder,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

#[tokio::test]
async fn saved_endpoint_reaches_existing_production_client_without_redirecting_an_inflight_stream()
{
    let release = Arc::new(tokio::sync::Notify::new());
    let old_release = release.clone();
    let old_app = Router::new().route("/api/generate", post(move |Json(_): Json<serde_json::Value>| {
        let release = old_release.clone();
        async move {
            let frames = futures_util::stream::unfold((0, release), |(step, release)| async move {
                if step > 1 { return None; }
                if step == 1 { release.notified().await; }
                let frame = if step == 0 {
                    b"{\"model\":\"fixture\",\"created_at\":\"2026-11-03T00:00:00Z\",\"response\":\"old endpoint\",\"done\":false}\n".as_slice()
                } else {
                    b"{\"model\":\"fixture\",\"created_at\":\"2026-11-03T00:00:00Z\",\"response\":\"\",\"done\":true}\n".as_slice()
                };
                Some((Ok::<_, std::convert::Infallible>(Bytes::copy_from_slice(frame)), (step + 1, release)))
            });
            Response::new(Body::from_stream(frames))
        }
    }));
    let new_app = Router::new().route("/api/generate", post(|| async { Json(serde_json::json!({"model":"fixture", "created_at":"2026-11-03T00:00:00Z", "response":"new endpoint", "done":true})) }));
    let old_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("old fixture");
    let new_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("new fixture");
    let old_url = format!("http://{}", old_listener.local_addr().expect("old address"));
    let new_url = format!("http://{}", new_listener.local_addr().expect("new address"));
    let old_server = tokio::spawn(async move {
        axum::serve(old_listener, old_app)
            .await
            .expect("old server")
    });
    let new_server = tokio::spawn(async move {
        axum::serve(new_listener, new_app)
            .await
            .expect("new server")
    });
    let data = tempfile::tempdir().expect("isolated data");
    let roles = oclive_kernel_runtime::chat_pro_roles_dir(&[std::path::PathBuf::from(env!(
        "CARGO_MANIFEST_DIR"
    ))])
    .expect("read-only roles");
    let state = AppStateBuilder::production(data.path().join("app.db"), roles, data.path())
        .with_host_profile(HostProfile::default())
        .build()
        .await
        .expect("production Host");
    state
        .db_manager
        .upsert_app_setting(KEY_OLLAMA_BASE, &old_url)
        .await
        .expect("save old endpoint");
    state
        .db_manager
        .upsert_app_setting("user_llm_provider", "local")
        .await
        .expect("local provider");
    state.mark_user_llm_env_dirty();
    apply_user_llm_env(&state).await.expect("apply old URL");
    let client = state.ollama.as_ref().expect("shared Ollama client").clone();
    let started = Arc::new(AtomicBool::new(false));
    let old_started = started.clone();
    let old_inflight = tokio::spawn(async move {
        client
            .generate_stream_with_callback(
                "fixture",
                "old request",
                None,
                None,
                Arc::new(move |token| {
                    if token == "old endpoint" {
                        old_started.store(true, Ordering::SeqCst);
                    }
                }),
            )
            .await
    });
    tokio::time::timeout(Duration::from_secs(2), async {
        while !started.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("old request reached saved URL");
    state
        .db_manager
        .upsert_app_setting(KEY_OLLAMA_BASE, &new_url)
        .await
        .expect("save replacement URL");
    state.mark_user_llm_env_dirty();
    apply_user_llm_env(&state)
        .await
        .expect("apply new URL without recreating Host");
    let new_reply = tokio::time::timeout(
        Duration::from_secs(3),
        state.llm.generate_tag("fixture", "next request"),
    )
    .await
    .expect("new endpoint promptly")
    .expect("new reply");
    assert_eq!(new_reply, "new endpoint");
    release.notify_one();
    assert_eq!(
        old_inflight.await.expect("old task").expect("old reply"),
        "old endpoint"
    );
    state.db_manager.close_pool().await;
    old_server.abort();
    new_server.abort();
}
