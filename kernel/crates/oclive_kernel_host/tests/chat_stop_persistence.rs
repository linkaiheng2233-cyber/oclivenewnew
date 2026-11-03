//! Real Host cancellation and SQLite receipt/history, with a blocking provider.
#![allow(clippy::expect_used)]
use async_trait::async_trait;
use oclive_kernel_contracts::{LlmClient, LlmTokenSink};
use oclive_kernel_host::domain::chat_engine::{
    cancel_message, process_message, process_message_stream, recover_message,
};
use oclive_kernel_host::state::AppStateBuilder;
use oclive_kernel_types::models::dto::{CancelChatRequest, SendMessageRequest};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::Duration;

struct BlockingLlm {
    prefix: &'static str,
    started: AtomicBool,
    dropped: Arc<AtomicBool>,
    calls: AtomicUsize,
}
struct DropProbe(Arc<AtomicBool>);
impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

#[async_trait]
impl LlmClient for BlockingLlm {
    async fn generate(&self, _: &str, _: &str) -> oclive_kernel_types::Result<String> {
        Ok("下一轮仍可回复。".into())
    }
    async fn generate_tag(&self, _: &str, _: &str) -> oclive_kernel_types::Result<String> {
        Ok("{}".into())
    }
    async fn generate_stream(
        &self,
        _: &str,
        _: &str,
        sink: LlmTokenSink,
    ) -> oclive_kernel_types::Result<String> {
        if self.calls.fetch_add(1, Ordering::SeqCst) > 0 {
            sink("下一轮仍可回复。");
            return Ok("下一轮仍可回复。".into());
        }
        let _drop = DropProbe(self.dropped.clone());
        for c in self.prefix.chars() {
            sink(&c.to_string());
        }
        self.started.store(true, Ordering::SeqCst);
        std::future::pending().await
    }
}

fn roles() -> std::path::PathBuf {
    oclive_kernel_runtime::chat_pro_roles_dir(&[std::path::PathBuf::from(env!(
        "CARGO_MANIFEST_DIR"
    ))])
    .expect("official read-only roles")
}
fn request() -> SendMessageRequest {
    SendMessageRequest {
        role_id: "mumu".into(),
        user_message: "请继续说下去。".into(),
        client_request_id: Some(uuid::Uuid::new_v4().to_string()),
        chat_thread_id: Some(uuid::Uuid::new_v4().to_string()),
        ..Default::default()
    }
}
fn stop(req: &SendMessageRequest) -> CancelChatRequest {
    CancelChatRequest {
        role_id: req.role_id.clone(),
        session_id: req.session_id.clone(),
        chat_thread_id: req.chat_thread_id.clone(),
        client_request_id: req.client_request_id.clone().expect("turn UUID"),
    }
}

async fn exercise(prefix: &'static str) {
    let data = tempfile::tempdir().expect("isolated data");
    let db = data.path().join("chat.db");
    let llm = Arc::new(BlockingLlm {
        prefix,
        started: AtomicBool::new(false),
        dropped: Arc::new(AtomicBool::new(false)),
        calls: AtomicUsize::new(0),
    });
    let state = Arc::new(
        AppStateBuilder::production(&db, roles(), data.path())
            .with_llm_client(llm.clone())
            .build()
            .await
            .expect("Host"),
    );
    let req = Arc::new(request());
    let visible = Arc::new(Mutex::new(String::new()));
    let sink = visible.clone();
    let running_state = state.clone();
    let running_req = req.clone();
    let running = tokio::spawn(async move {
        process_message_stream(
            &running_state,
            &running_req,
            Arc::new(move |t| sink.lock().expect("visible").push_str(t)),
        )
        .await
    });
    tokio::time::timeout(Duration::from_secs(10), async {
        while !llm.started.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("provider started");
    assert!(cancel_message(&state, &stop(&req)).expect("stop"));
    assert!(cancel_message(&state, &stop(&req)).expect("repeated stop"));
    let response = tokio::time::timeout(Duration::from_secs(2), running)
        .await
        .expect("stop promptly")
        .expect("turn task")
        .expect("persisted stopped outcome");
    assert!(
        llm.dropped.load(Ordering::SeqCst),
        "the upstream generation future must actually be dropped"
    );
    let displayed = visible.lock().expect("visible text").clone();
    assert_eq!(response.reply, displayed);
    assert_eq!(response.reply, prefix);
    assert!(response.reply_stopped);
    assert!(!response.reply_is_fallback);
    assert_eq!(response.favorability_delta, 0.0);
    let sessions = state
        .conversation_store
        .list_sessions_by_role("mumu")
        .await
        .expect("sessions");
    assert_eq!(sessions.len(), 1);
    let storage_id = sessions[0].session_id.clone();
    let messages = state
        .conversation_store
        .fetch_messages(&storage_id, 20, 0)
        .await
        .expect("history");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[1].content, displayed);
    let metadata: serde_json::Value =
        serde_json::from_str(messages[1].metadata.as_deref().expect("metadata")).expect("JSON");
    assert_eq!(metadata["reply_stopped"], true);
    assert_eq!(metadata["reply_is_fallback"], false);
    let recovered = recover_message(&state, &req)
        .await
        .expect("completed receipt");
    assert!(recovered.reply_stopped);
    assert_eq!(
        recovered.assistant_message_id,
        response.assistant_message_id
    );
    assert_eq!(llm.calls.load(Ordering::SeqCst), 1);
    state.db_manager.close_pool().await;
    drop(state);
    let reopened = AppStateBuilder::production(&db, roles(), data.path())
        .with_llm_client(llm.clone())
        .build()
        .await
        .expect("reopened Host");
    let replay = process_message(&reopened, &req)
        .await
        .expect("retry same identity after restart");
    assert_eq!(replay.reply, displayed);
    assert!(replay.reply_stopped);
    assert_eq!(replay.assistant_message_id, response.assistant_message_id);
    assert_eq!(
        llm.calls.load(Ordering::SeqCst),
        1,
        "receipt replay must not generate again"
    );
    let mut next = request();
    next.chat_thread_id = req.chat_thread_id.clone();
    let next_response = tokio::time::timeout(
        Duration::from_secs(3),
        process_message_stream(&reopened, &next, Arc::new(|_| {})),
    )
    .await
    .expect("next send not blocked")
    .expect("next reply");
    assert!(!next_response.reply_stopped);
    assert!(!next_response.reply_is_fallback);
    assert_eq!(llm.calls.load(Ordering::SeqCst), 2);
    let after = reopened
        .conversation_store
        .fetch_messages(&storage_id, 20, 0)
        .await
        .expect("reopened history");
    assert_eq!(after.len(), 4);
    assert_eq!(after[1].content, displayed);
    assert_eq!(after[1].id, messages[1].id);
    reopened.db_manager.close_pool().await;
}

#[tokio::test]
async fn stopped_visible_prefix_survives_recovery_restart_and_next_send() {
    exercise("这一段已经写出。").await;
}

#[tokio::test]
async fn stop_before_first_token_persists_status_without_a_fake_error() {
    exercise("").await;
}

#[tokio::test]
async fn stop_arriving_before_send_never_starts_the_provider() {
    let data = tempfile::tempdir().expect("isolated data");
    let llm = Arc::new(BlockingLlm {
        prefix: "must not run",
        started: AtomicBool::new(false),
        dropped: Arc::new(AtomicBool::new(false)),
        calls: AtomicUsize::new(0),
    });
    let state = AppStateBuilder::production(data.path().join("chat.db"), roles(), data.path())
        .with_llm_client(llm.clone())
        .build()
        .await
        .expect("Host");
    let req = request();
    assert!(cancel_message(&state, &stop(&req)).expect("early stop"));
    let response = process_message_stream(&state, &req, Arc::new(|_| panic!("no token")))
        .await
        .expect("early stopped receipt");
    assert!(response.reply_stopped);
    assert_eq!(response.reply, "");
    assert_eq!(llm.calls.load(Ordering::SeqCst), 0);
    state.db_manager.close_pool().await;
}

#[tokio::test]
async fn explicit_stop_is_authenticated_and_closes_the_actual_ollama_http_stream() {
    use axum::{
        body::{Body, Bytes},
        http::Request,
        response::Response,
        routing::post,
        Json, Router,
    };
    use oclive_kernel_host::infrastructure::ollama_client::OllamaClient;
    use tower::ServiceExt;

    struct HttpStreamingLlm(OllamaClient);
    #[async_trait]
    impl LlmClient for HttpStreamingLlm {
        async fn generate(&self, _: &str, _: &str) -> oclive_kernel_types::Result<String> {
            Ok("unused complete reply".into())
        }
        async fn generate_tag(&self, _: &str, _: &str) -> oclive_kernel_types::Result<String> {
            Ok("{}".into())
        }
        async fn generate_stream(
            &self,
            model: &str,
            prompt: &str,
            sink: LlmTokenSink,
        ) -> oclive_kernel_types::Result<String> {
            LlmClient::generate_stream(&self.0, model, prompt, sink).await
        }
    }
    struct StreamState {
        sent: bool,
        closed: Arc<AtomicBool>,
    }
    impl Drop for StreamState {
        fn drop(&mut self) {
            self.closed.store(true, Ordering::SeqCst);
        }
    }
    let closed = Arc::new(AtomicBool::new(false));
    let server_closed = closed.clone();
    let upstream = Router::new().route("/api/generate", post(move |Json(_): Json<serde_json::Value>| {
        let closed = server_closed.clone();
        async move {
            let stream = futures_util::stream::unfold(StreamState { sent: false, closed }, |mut state| async move {
                if state.sent { return std::future::pending().await; }
                state.sent = true;
                Some((Ok::<_, std::convert::Infallible>(Bytes::from_static(b"{\"model\":\"test\",\"created_at\":\"2026-11-03T00:00:00Z\",\"response\":\"visible prefix\",\"done\":false}\n")), state))
            });
            Response::builder().header("content-type", "application/x-ndjson").body(Body::from_stream(stream)).expect("stream response")
        }
    }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("loopback only");
    let base = format!(
        "http://{}",
        listener.local_addr().expect("loopback address")
    );
    let (shutdown, shutdown_rx) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        axum::serve(listener, upstream)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            .expect("fake Ollama server");
    });
    let data = tempfile::tempdir().expect("isolated data");
    let state = Arc::new(
        AppStateBuilder::production(data.path().join("chat.db"), roles(), data.path())
            .with_llm_client(Arc::new(HttpStreamingLlm(OllamaClient::new(&base))))
            .build()
            .await
            .expect("HTTP Host"),
    );
    let req = Arc::new(request());
    let router = oclive_kernel_host::http_api::api_router_with_auth(
        state.clone(),
        Some("test-only-token".into()),
    );
    let body = serde_json::to_vec(&stop(&req)).expect("cancel body");
    let forbidden = router
        .clone()
        .oneshot(
            Request::post("/chat/cancel")
                .header("content-type", "application/json")
                .body(Body::from(body.clone()))
                .expect("request"),
        )
        .await
        .expect("unauthenticated response");
    assert_eq!(forbidden.status(), axum::http::StatusCode::UNAUTHORIZED);
    let running_state = state.clone();
    let running_req = req.clone();
    let seen = Arc::new(AtomicBool::new(false));
    let seen_sink = seen.clone();
    let running = tokio::spawn(async move {
        process_message_stream(
            &running_state,
            &running_req,
            Arc::new(move |token| {
                if token.contains("visible") {
                    seen_sink.store(true, Ordering::SeqCst);
                }
            }),
        )
        .await
    });
    tokio::time::timeout(Duration::from_secs(10), async {
        while !seen.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("real HTTP token arrived");
    let stopped = router
        .oneshot(
            Request::post("/chat/cancel")
                .header("content-type", "application/json")
                .header("x-oclive-api-token", "test-only-token")
                .body(Body::from(body))
                .expect("authorized request"),
        )
        .await
        .expect("stop response");
    assert_eq!(stopped.status(), axum::http::StatusCode::OK);
    let response = tokio::time::timeout(Duration::from_secs(2), running)
        .await
        .expect("stopped promptly")
        .expect("turn task")
        .expect("saved partial");
    assert!(response.reply_stopped);
    assert_eq!(response.reply, "visible prefix");
    tokio::time::timeout(Duration::from_secs(3), async {
        while !closed.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("the actual upstream HTTP body was dropped");
    state.db_manager.close_pool().await;
    let _ = shutdown.send(());
    tokio::time::timeout(Duration::from_secs(3), server)
        .await
        .expect("server shutdown")
        .expect("server task");
}
