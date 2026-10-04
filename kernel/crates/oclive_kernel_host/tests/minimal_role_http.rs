#![allow(clippy::expect_used, clippy::unwrap_used)]

use async_trait::async_trait;
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use oclive_kernel_contracts::{LlmClient, LlmGenerateOpts, LlmGenerateOutcome};
use oclive_kernel_host::http_api::api_router_with_auth;
use oclive_kernel_host::state::AppState;
use oclive_kernel_types::models::dto::{
    MinimalRoleLocalMessageRequest, MinimalRoleLocalSource, MinimalRoleMessageRequest,
    MinimalRoleMessageResponse, MinimalRoleProductExtensionStatus,
};
use oclive_kernel_types::{AppError, Result};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

struct RecordingLlm {
    prompts: Mutex<Vec<String>>,
    fail: bool,
}

#[async_trait]
impl LlmClient for RecordingLlm {
    async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
        panic!("only the checked text entry may be called")
    }

    async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
        panic!("the minimum transport must not invoke rich classification")
    }

    async fn generate_with_opts(
        &self,
        _model: &str,
        prompt: &str,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        assert!(opts.is_none());
        tokio::task::yield_now().await;
        self.prompts.lock().unwrap().push(prompt.into());
        if self.fail {
            return Err(AppError::OllamaError(
                "synthetic transport model failure".into(),
            ));
        }
        Ok(LlmGenerateOutcome {
            reply: "  authoritative text\r\n".into(),
            prompt_eval_ms: None,
        })
    }
}

struct Fixture {
    root: tempfile::TempDir,
    state: Arc<AppState>,
    llm: Arc<RecordingLlm>,
}

impl Fixture {
    async fn new(fail: bool) -> Self {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("image.bin"), b"nonempty asset snapshot").unwrap();
        std::fs::write(
            root.path().join("chosen-content.json"),
            json!({"persona_prompt":"  authored persona\r\n", "visual_assets":["image.bin"]})
                .to_string(),
        )
        .unwrap();
        let llm = Arc::new(RecordingLlm {
            prompts: Mutex::default(),
            fail,
        });
        let state = Arc::new(
            AppState::new_in_memory_with_llm(llm.clone(), root.path())
                .await
                .unwrap(),
        );
        Self { root, state, llm }
    }

    fn request(&self) -> MinimalRoleLocalMessageRequest {
        MinimalRoleLocalMessageRequest {
            source: MinimalRoleLocalSource {
                role_id: "converter-owned-id".into(),
                asset_root: self.root.path().to_string_lossy().into_owned(),
                definition_reference: "chosen-content.json".into(),
            },
            message: MinimalRoleMessageRequest {
                user_message: "  current material\r\n".into(),
                requirements: String::new(),
            },
        }
    }

    async fn post(&self, request: Value, authenticated: bool) -> (StatusCode, Value) {
        let router = api_router_with_auth(self.state.clone(), Some("fixture-token".into()));
        let mut builder = Request::post("/chat/minimal").header("content-type", "application/json");
        if authenticated {
            builder = builder.header("x-oclive-api-token", "fixture-token");
        }
        let response = router
            .oneshot(builder.body(Body::from(request.to_string())).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
        let body = serde_json::from_slice(&bytes).unwrap_or_else(
            |_| json!({"rejection_text": String::from_utf8(bytes.to_vec()).unwrap()}),
        );
        (status, body)
    }

    async fn assert_no_rich_activation(&self) {
        assert!(self.state.role_cache.read().is_empty());
        assert!(!self
            .state
            .db_manager
            .role_runtime_exists("converter-owned-id")
            .await
            .unwrap());
        assert!(!self.root.path().join("chats").exists());
    }

    async fn close(self) {
        self.state.db_manager.close_pool().await;
        drop(self.state);
        self.root.close().unwrap();
    }
}

#[tokio::test]
async fn authenticated_minimum_transport_uses_real_loading_and_one_text_call() {
    let fixture = Fixture::new(false).await;
    let (status, wire) = fixture
        .post(serde_json::to_value(fixture.request()).unwrap(), true)
        .await;
    assert_eq!(status, StatusCode::OK, "{wire}");
    let response: MinimalRoleMessageResponse = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(response.role_id, "converter-owned-id");
    assert_eq!(response.reply, "  authoritative text\r\n");
    assert_eq!(
        response.product_extensions,
        MinimalRoleProductExtensionStatus::Unavailable
    );
    assert_eq!(wire.as_object().unwrap().len(), 3);
    let prompts = fixture.llm.prompts.lock().unwrap().clone();
    assert_eq!(prompts.len(), 1);
    assert!(prompts[0].contains("  authored persona\r\n"));
    assert!(prompts[0].contains("  current material\r\n"));
    fixture.assert_no_rich_activation().await;
    fixture.close().await;
}

#[tokio::test]
async fn minimum_transport_keeps_authentication_and_refuses_bad_inputs_before_generation() {
    let fixture = Fixture::new(false).await;
    let valid = serde_json::to_value(fixture.request()).unwrap();
    let (status, wire) = fixture.post(valid.clone(), false).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(wire["error"]["code"], "KERNEL_AUTH_REQUIRED");

    let mut cases = Vec::new();
    for (path, value) in [
        ("/source/role_id", json!("   ")),
        ("/source/asset_root", json!("relative-root")),
        ("/source/definition_reference", json!("../outside.json")),
        ("/source/definition_reference", json!("missing.json")),
        ("/message/user_message", json!(" \r\n\t")),
        ("/message/requirements", json!("retain a topic")),
    ] {
        let mut body = valid.clone();
        *body.pointer_mut(path).unwrap() = value;
        cases.push(body);
    }
    for body in cases {
        let (status, wire) = fixture.post(body, true).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{wire}");
        assert!(["EMPTY_MESSAGE", "INVALID_PARAMETER"]
            .contains(&wire["error"]["code"].as_str().unwrap()));
    }
    for body in [
        json!({"source":valid["source"],"message":valid["message"],"adult":true}),
        json!({"source":valid["source"],"message":{"user_message":"hello","scene_id":"fake"}}),
    ] {
        assert_eq!(
            fixture.post(body, true).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    assert!(fixture.llm.prompts.lock().unwrap().is_empty());
    fixture.assert_no_rich_activation().await;
    fixture.close().await;
}

#[tokio::test]
async fn minimum_transport_enforces_local_budgets_and_missing_assets_without_a_rich_fallback() {
    let fixture = Fixture::new(false).await;
    for definition in [
        json!({"persona_prompt":"persona","visual_assets":["missing.bin"]}),
        json!({"persona_prompt":"persona","visual_assets":["../outside.bin"]}),
        json!({"persona_prompt":"x".repeat(64 * 1024),"visual_assets":["image.bin"]}),
    ] {
        std::fs::write(
            fixture.root.path().join("chosen-content.json"),
            definition.to_string(),
        )
        .unwrap();
        let (status, body) = fixture
            .post(serde_json::to_value(fixture.request()).unwrap(), true)
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(body["error"]["code"], "INVALID_PARAMETER");
    }
    for (bytes, assets) in [
        (4 * 1024 * 1024 + 1, vec!["image.bin"]),
        (4 * 1024 * 1024, vec!["image.bin"; 5]),
    ] {
        std::fs::write(fixture.root.path().join("image.bin"), vec![b'x'; bytes]).unwrap();
        std::fs::write(
            fixture.root.path().join("chosen-content.json"),
            json!({"persona_prompt":"persona","visual_assets":assets}).to_string(),
        )
        .unwrap();
        let (status, body) = fixture
            .post(serde_json::to_value(fixture.request()).unwrap(), true)
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(body["error"]["code"], "INVALID_PARAMETER");
    }
    assert!(fixture.llm.prompts.lock().unwrap().is_empty());
    fixture.assert_no_rich_activation().await;
    fixture.close().await;
}

#[tokio::test]
async fn minimum_transport_keeps_model_errors_and_never_retries_or_activates_a_rich_role() {
    let fixture = Fixture::new(true).await;
    let (status, wire) = fixture
        .post(serde_json::to_value(fixture.request()).unwrap(), true)
        .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(wire["error"]["code"], "LLM_ERROR");
    assert!(wire["error"]["message"]
        .as_str()
        .unwrap()
        .contains("synthetic transport model failure"));
    assert_eq!(fixture.llm.prompts.lock().unwrap().len(), 1);
    fixture.assert_no_rich_activation().await;
    fixture.close().await;
}

#[test]
fn minimum_local_transport_without_a_runtime_returns_a_host_error_before_generation() {
    use std::future::Future;
    use std::task::{Context, Poll, Waker};

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let fixture = runtime.block_on(Fixture::new(false));
    let mut call = Box::pin(
        oclive_kernel_host::domain::chat_engine::process_minimal_local_message(
            fixture.state.clone(),
            fixture.request(),
        ),
    );
    assert!(matches!(
        call.as_mut().poll(&mut Context::from_waker(Waker::noop())),
        Poll::Ready(Err(AppError::Unknown(message))) if message.contains("needs a Tokio runtime")
    ));
    drop(call);
    assert!(fixture.llm.prompts.lock().unwrap().is_empty());
    runtime.block_on(fixture.close());
}

#[tokio::test]
async fn minimum_transport_selects_quoted_current_conversation_without_rich_memory() {
    let fixture = Fixture::new(false).await;
    let mut request = serde_json::to_value(fixture.request()).unwrap();
    request["message"]["user_message"] = json!("coffee");
    request["conversation"] = json!([
        {"user_message":"She said she does not like coffee.\r\n", "reply":"Her statement was quoted, not yours."},
        {"user_message":"He prefers tea.", "reply":"Only tea was discussed."}
    ]);
    let (status, wire) = fixture.post(request.clone(), true).await;
    assert_eq!(status, StatusCode::OK, "{wire}");
    {
        let prompts = fixture.llm.prompts.lock().unwrap();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].matches("  authored persona\r\n").count(), 1);
        assert!(prompts[0].contains("She said she does not like coffee.\r\n"));
        assert!(prompts[0].contains("Her statement was quoted, not yours."));
        assert!(!prompts[0].contains("He prefers tea."));
        assert!(prompts[0].contains("User: coffee"));
    }
    request["message"]["user_message"] = json!("unmatched-subject");
    assert_eq!(fixture.post(request, true).await.0, StatusCode::OK);
    {
        let prompts = fixture.llm.prompts.lock().unwrap();
        assert_eq!(prompts.len(), 2);
        assert_eq!(
            prompts[1],
            "【角色设定】\n  authored persona\r\n\n\n【输入材料】\nUser: unmatched-subject"
        );
    }
    fixture.assert_no_rich_activation().await;
    fixture.close().await;
}

#[tokio::test]
async fn minimum_conversation_budget_is_rejected_before_loading_or_generation() {
    let fixture = Fixture::new(false).await;
    let mut request = serde_json::to_value(fixture.request()).unwrap();
    // Source is deliberately missing: budget rejection must precede file loading.
    request["source"]["definition_reference"] = json!("does-not-exist.json");
    for conversation in [
        json!(vec![json!({"user_message":"a", "reply":"b"}); 9]),
        json!([{ "user_message":"😀".repeat(16 * 1024), "reply":"x" }]),
    ] {
        request["conversation"] = conversation;
        let (status, wire) = fixture.post(request.clone(), true).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{wire}");
        assert_eq!(wire["error"]["code"], "INVALID_PARAMETER");
        assert!(wire["error"]["message"]
            .as_str()
            .unwrap()
            .contains("conversation budget"));
    }
    for conversation in [
        json!(null),
        json!([{ "user_message":"a", "reply":"b", "session_id":"old-rich" }]),
    ] {
        request["conversation"] = conversation;
        assert_eq!(
            fixture.post(request.clone(), true).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    assert!(fixture.llm.prompts.lock().unwrap().is_empty());
    request["source"]["definition_reference"] = json!("chosen-content.json");
    request["conversation"] = json!([
        { "user_message": "a".repeat(64 * 1024), "reply": "" }
    ]);
    assert_eq!(fixture.post(request, true).await.0, StatusCode::OK);
    assert_eq!(fixture.llm.prompts.lock().unwrap().len(), 1);
    fixture.assert_no_rich_activation().await;
    fixture.close().await;
}
