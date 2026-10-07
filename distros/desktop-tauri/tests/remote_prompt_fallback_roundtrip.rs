//! Characterize existing Remote Prompt and Memory fallback boundaries against an isolated sidecar.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::{extract::Json, http::StatusCode, response::IntoResponse, routing::post, Router};
use oclive_kernel_contracts::{MemoryRetrieval, PromptAssembler};
use oclive_kernel_host::domain::{BuiltinMemoryRetrieval, BuiltinPromptAssembler};
use oclive_kernel_host::infrastructure::high_risk_grants::HighRiskGrantStore;
use oclive_kernel_host::infrastructure::remote_plugin::{
    RemoteMemoryRetrievalHttp, RemotePluginHttpConfig, RemotePromptAssemblerHttp,
};
use oclive_kernel_types::{
    AppError, EventType, Memory, MemoryConfig, MemoryRetrievalInput, PersonalityVector,
    PromptInput, Role,
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
        let (config, grants) = self.connection(directory, grant);
        RemotePromptAssemblerHttp::new(
            Arc::new(reqwest::Client::new()),
            config,
            allowed,
            grants,
            Some(NETWORK_GRANT_REMOTE_PLUGIN.into()),
        )
    }

    fn memory_client(
        &self,
        directory: &TempDir,
        allowed: Arc<AtomicBool>,
        grant: bool,
    ) -> RemoteMemoryRetrievalHttp {
        let (config, grants) = self.connection(directory, grant);
        RemoteMemoryRetrievalHttp::new(
            Arc::new(reqwest::Client::new()),
            config,
            allowed,
            grants,
            Some(NETWORK_GRANT_REMOTE_PLUGIN.into()),
        )
    }

    fn connection(
        &self,
        directory: &TempDir,
        grant: bool,
    ) -> (RemotePluginHttpConfig, Arc<HighRiskGrantStore>) {
        assert!(std::env::var("OCLIVE_SKIP_HIGH_RISK_GRANTS").is_err());
        let grants = HighRiskGrantStore::load(directory.path().to_path_buf(), true);
        if grant {
            grants.grant_network(NETWORK_GRANT_REMOTE_PLUGIN).unwrap();
        }
        (
            RemotePluginHttpConfig {
                endpoint: self.url.clone(),
                timeout: Duration::from_secs(2),
                bearer_token: None,
            },
            grants,
        )
    }

    fn assert_memory_requests(&self, expected: usize, memories: &[Memory], limit: usize) {
        let requests = self.requests.lock().unwrap();
        assert_eq!(requests.len(), expected, "one request per call; no retry");
        for request in requests.iter() {
            assert_eq!(request["method"], "memory.rank");
            assert_eq!(
                request["params"],
                json!({
                    "memories": memories,
                    "user_query": "synthetic query",
                    "scene_id": "scene",
                    "limit": limit,
                })
            );
        }
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

fn memories_for_ranking() -> Vec<Memory> {
    [("a", 1.0), ("b", 10.0), ("c", 2.0)]
        .into_iter()
        .map(|(id, importance)| Memory {
            id: id.into(),
            role_id: "synthetic-role".into(),
            content: format!("memory-{id}"),
            importance,
            weight: 1.0,
            created_at: chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
            scene_id: None,
            mention_count: 1,
            accessed_at: None,
        })
        .collect()
}

fn memory_input(memories: &[Memory], limit: usize) -> MemoryRetrievalInput<'_> {
    MemoryRetrievalInput {
        memories,
        user_query: "synthetic query",
        scene_id: Some("scene"),
        limit,
    }
}

fn assert_memory_unavailable(error: AppError, endpoint: &str) {
    match error {
        AppError::RemoteServiceUnavailable(message) => assert_eq!(
            message,
            format!("memory.rank remote failed or empty endpoint={endpoint}")
        ),
        other => panic!("unexpected error: {other}"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_success_preserves_order_tail_and_limit_even_without_fallback() {
    let memories = memories_for_ranking();
    for (result, expected) in [
        (json!({"ordered_ids": ["unknown", "c", "a"]}), ["c", "a"]),
        (json!({"ordered_ids": []}), ["a", "b"]),
    ] {
        let sidecar = PromptSidecar::start(StatusCode::OK, result).await;
        let directory = tempfile::tempdir().unwrap();
        let client = sidecar.memory_client(&directory, Arc::new(AtomicBool::new(false)), true);
        let ranked = client.rank_memories(memory_input(&memories, 2)).unwrap();
        assert_eq!(
            ranked.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
            expected
        );
        let expected_memories: Vec<_> = expected
            .iter()
            .map(|id| memories.iter().find(|m| m.id == *id).unwrap().clone())
            .collect();
        assert_eq!(
            serde_json::to_value(&ranked).unwrap(),
            serde_json::to_value(&expected_memories).unwrap()
        );
        sidecar.assert_memory_requests(1, &memories, 2);
        sidecar.close().await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_soft_failure_and_bad_shape_obey_live_gate_without_retry() {
    let memories = memories_for_ranking();
    for (status, result) in [
        (StatusCode::SERVICE_UNAVAILABLE, json!({})),
        (StatusCode::OK, json!({"ordered_ids": "not-an-array"})),
    ] {
        let sidecar = PromptSidecar::start(status, result).await;
        let directory = tempfile::tempdir().unwrap();
        let allowed = Arc::new(AtomicBool::new(true));
        let client = sidecar.memory_client(&directory, allowed.clone(), true);
        assert_eq!(
            serde_json::to_value(client.rank_memories(memory_input(&memories, 2)).unwrap())
                .unwrap(),
            serde_json::to_value(
                BuiltinMemoryRetrieval
                    .rank_memories(memory_input(&memories, 2))
                    .unwrap()
            )
            .unwrap()
        );
        allowed.store(false, Ordering::Relaxed);
        assert_memory_unavailable(
            client
                .rank_memories(memory_input(&memories, 2))
                .unwrap_err(),
            &sidecar.url,
        );
        sidecar.assert_memory_requests(2, &memories, 2);
        sidecar.close().await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_empty_input_keeps_its_method_specific_fallback_rule() {
    let sidecar = PromptSidecar::start(StatusCode::OK, json!({"ordered_ids": []})).await;
    let directory = tempfile::tempdir().unwrap();
    let allowed = Arc::new(AtomicBool::new(true));
    let client = sidecar.memory_client(&directory, allowed.clone(), true);
    assert!(client
        .rank_memories(memory_input(&[], 2))
        .unwrap()
        .is_empty());
    allowed.store(false, Ordering::Relaxed);
    assert_memory_unavailable(
        client.rank_memories(memory_input(&[], 2)).unwrap_err(),
        &sidecar.url,
    );
    sidecar.assert_memory_requests(2, &[], 2);
    sidecar.close().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_permission_denial_never_sends_or_falls_back() {
    let sidecar = PromptSidecar::start(StatusCode::OK, json!({"ordered_ids": ["a"]})).await;
    let directory = tempfile::tempdir().unwrap();
    let allowed = Arc::new(AtomicBool::new(true));
    let client = sidecar.memory_client(&directory, allowed.clone(), false);
    let memories = memories_for_ranking();
    for fallback in [true, false] {
        allowed.store(fallback, Ordering::Relaxed);
        assert!(matches!(
            client.rank_memories(memory_input(&memories, 2)),
            Err(AppError::HighRiskCapabilityNotGranted { .. })
        ));
    }
    sidecar.assert_memory_requests(0, &memories, 2);
    sidecar.close().await;
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
