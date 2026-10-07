//! Characterize the existing Remote Prompt fallback boundaries against an isolated HTTP sidecar.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::{extract::Json, http::StatusCode, response::IntoResponse, routing::post, Router};
use oclive_kernel_contracts::PromptAssembler;
use oclive_kernel_host::domain::BuiltinPromptAssembler;
use oclive_kernel_host::infrastructure::high_risk_grants::HighRiskGrantStore;
use oclive_kernel_host::infrastructure::remote_plugin::{
    RemotePluginHttpConfig, RemotePromptAssemblerHttp,
};
use oclive_kernel_types::{
    AppError, EventType, MemoryConfig, PersonalityVector, PromptInput, Role,
};
use oclive_validation::NETWORK_GRANT_REMOTE_PLUGIN;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tempfile::TempDir;

struct PromptSidecar {
    url: String,
    requests: Arc<Mutex<Vec<Value>>>,
    server: tokio::task::JoinHandle<()>,
}

impl PromptSidecar {
    async fn start(status: StatusCode, result: Value) -> Self {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let observed = requests.clone();
        let app = Router::new().route(
            "/rpc",
            post(move |Json(body): Json<Value>| {
                let observed = observed.clone();
                let result = result.clone();
                async move {
                    observed.lock().unwrap().push(body.clone());
                    (
                        status,
                        Json(json!({"jsonrpc": "2.0", "id": body["id"], "result": result})),
                    )
                        .into_response()
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/rpc", listener.local_addr().unwrap());
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        Self {
            url,
            requests,
            server,
        }
    }

    fn client(
        &self,
        directory: &TempDir,
        allowed: Arc<AtomicBool>,
        grant: bool,
    ) -> RemotePromptAssemblerHttp {
        assert!(std::env::var("OCLIVE_SKIP_HIGH_RISK_GRANTS").is_err());
        let grants = HighRiskGrantStore::load(directory.path().to_path_buf(), true);
        if grant {
            grants.grant_network(NETWORK_GRANT_REMOTE_PLUGIN).unwrap();
        }
        RemotePromptAssemblerHttp::new(
            Arc::new(reqwest::Client::new()),
            RemotePluginHttpConfig {
                endpoint: self.url.clone(),
                timeout: Duration::from_secs(2),
                bearer_token: None,
            },
            allowed,
            grants,
            Some(NETWORK_GRANT_REMOTE_PLUGIN.into()),
        )
    }

    fn assert_requests(&self, expected: usize, method: &str, role: &Role) {
        let requests = self.requests.lock().unwrap();
        assert_eq!(requests.len(), expected, "one request per call; no retry");
        for request in requests.iter() {
            assert_eq!(request["method"], method);
            assert_eq!(
                request["params"]["role"],
                serde_json::to_value(role).unwrap()
            );
            if method == "prompt.top_topic_hint" {
                assert_eq!(request["params"]["scene_id"], "scene");
            }
        }
    }

    async fn close(mut self) {
        self.server.abort();
        let cancelled = tokio::time::timeout(Duration::from_secs(2), &mut self.server)
            .await
            .expect("sidecar exit deadline")
            .expect_err("sidecar aborted");
        assert!(cancelled.is_cancelled());
    }
}

impl Drop for PromptSidecar {
    fn drop(&mut self) {
        self.server.abort();
    }
}

fn role_with_builtin_hint() -> Role {
    let mut memory = MemoryConfig::default();
    memory.topic_weights.insert(
        "scene".into(),
        HashMap::from([("builtin-topic".into(), 1.0)]),
    );
    Role {
        memory_config: Some(memory),
        ..Role::default()
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn topic_hint_success_shape_does_not_trigger_builtin() {
    let role = role_with_builtin_hint();
    for (result, expected) in [
        (json!({"hint": "remote-topic"}), Some("remote-topic")),
        (json!("remote-topic"), Some("remote-topic")),
        (json!(""), Some("")),
        (json!({"hint": 42}), None),
    ] {
        let sidecar = PromptSidecar::start(StatusCode::OK, result).await;
        let directory = tempfile::tempdir().unwrap();
        let client = sidecar.client(&directory, Arc::new(AtomicBool::new(true)), true);
        assert_eq!(
            client.top_topic_hint(&role, "scene"),
            expected.map(String::from)
        );
        sidecar.assert_requests(1, "prompt.top_topic_hint", &role);
        sidecar.close().await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn topic_hint_transport_failure_obeys_live_switch_without_retry() {
    let sidecar = PromptSidecar::start(StatusCode::SERVICE_UNAVAILABLE, json!({})).await;
    let directory = tempfile::tempdir().unwrap();
    let allowed = Arc::new(AtomicBool::new(true));
    let client = sidecar.client(&directory, allowed.clone(), true);
    let role = role_with_builtin_hint();
    assert_eq!(
        client.top_topic_hint(&role, "scene"),
        Some("builtin-topic".into())
    );
    allowed.store(false, Ordering::Relaxed);
    assert_eq!(client.top_topic_hint(&role, "scene"), None);
    sidecar.assert_requests(2, "prompt.top_topic_hint", &role);
    sidecar.close().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn topic_hint_permission_denial_never_sends_or_falls_back() {
    let sidecar = PromptSidecar::start(StatusCode::OK, json!("remote-topic")).await;
    let directory = tempfile::tempdir().unwrap();
    let client = sidecar.client(&directory, Arc::new(AtomicBool::new(true)), false);
    let role = role_with_builtin_hint();
    assert_eq!(
        BuiltinPromptAssembler.top_topic_hint(&role, "scene"),
        Some("builtin-topic".into())
    );
    assert_eq!(client.top_topic_hint(&role, "scene"), None);
    sidecar.assert_requests(0, "prompt.top_topic_hint", &role);
    sidecar.close().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn build_prompt_bad_shape_keeps_its_existing_gate_and_error() {
    let sidecar = PromptSidecar::start(StatusCode::OK, json!({"hint": "not-a-prompt"})).await;
    let directory = tempfile::tempdir().unwrap();
    let allowed = Arc::new(AtomicBool::new(true));
    let client = sidecar.client(&directory, allowed.clone(), true);
    let role = role_with_builtin_hint();
    let personality = PersonalityVector::zero();
    let event = EventType::Praise;
    let input = PromptInput {
        role: &role,
        personality: &personality,
        memories: &[],
        user_input: "hello",
        user_emotion: "neutral",
        user_relation_id: "friend",
        relation_hint: "",
        relation_before: "Friend",
        favorability_before: 50.0,
        relation_preview: "Friend",
        favorability_preview: 50.0,
        event_type: &event,
        impact_factor: 0.0,
        scene_label: "scene",
        scene_detail: "",
        topic_hint_line: "",
        life_context_line: "",
        worldview_snippet: "",
        mutable_personality: "",
        ephemeral_personality: "",
        reply_quality_anchor: "",
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
        persona_override: None,
        previous_assistant_reply: "",
    };
    assert_eq!(
        client.build_prompt(&input).unwrap(),
        BuiltinPromptAssembler.build_prompt(&input).unwrap()
    );
    allowed.store(false, Ordering::Relaxed);
    match client.build_prompt(&input).unwrap_err() {
        AppError::RemoteServiceUnavailable(message) => {
            assert_eq!(
                message,
                format!("prompt.build_prompt: bad shape endpoint={}", sidecar.url)
            );
        }
        other => panic!("unexpected error: {other}"),
    }
    sidecar.assert_requests(2, "prompt.build_prompt", &role);
    sidecar.close().await;
}
