//! `complex_emotion` backend contract: `none` gates hint state while plugin
//! fallback labels continue through the six-slot emotion consumers.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use async_trait::async_trait;
use axum::{extract::Json, routing::post, Router};
use oclive_kernel_host::domain::chat_engine::{conversation_state_role_id, process_message};
use oclive_kernel_host::domain::complex_emotion_store::{
    load_stored_narrative_hint, persist_stored_narrative_hint,
};
use oclive_kernel_host::domain::host_profile::{
    HostProfile, TurnThinkingDefault, TurnThinkingProfile,
};
use oclive_kernel_host::infrastructure::llm::LlmClient;
use oclive_kernel_host::state::{AppState, AppStateBuilder};
use oclive_kernel_types::models::dto::SendMessageRequest;
use oclive_validation::NETWORK_GRANT_REMOTE_PLUGIN;
use oclivenewnew_tauri::error::Result;
use parking_lot::Mutex;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

struct CaptureLlm {
    reply: String,
    prompts: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl LlmClient for CaptureLlm {
    async fn generate(&self, _model: &str, prompt: &str) -> Result<String> {
        self.prompts.lock().push(prompt.to_string());
        Ok(self.reply.clone())
    }

    async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok("neutral".to_string())
    }

    async fn startup_probe(&self) -> Result<()> {
        Ok(())
    }
}

fn write_role(root: &Path, role_id: &str, complex_backend: &str, url: Option<&str>) {
    let role = root.join(role_id);
    std::fs::create_dir_all(role.join("scenes/default")).unwrap();
    let mut complex = json!({
        "type": "complex_emotion",
        "label": "Complex emotion",
        "backend": complex_backend,
        "position": 7
    });
    if let Some(url) = url {
        complex["url"] = json!(url);
    }
    let blueprint = json!({
        "schema_version": 2,
        "meta": {
            "id": role_id,
            "name": "Complex Emotion Contract",
            "version": "0.1.0",
            "author": "test",
            "description": "contract fixture",
            "personality": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            "relations": {
                "friend": {
                    "initial_favorability": 50,
                    "favor_multiplier": 1.0,
                    "prompt_hint": "friend"
                }
            },
            "default_relation": "friend",
            "scenes": ["default"]
        },
        "slot_registry": {
            "memory": { "type": "memory", "label": "Memory", "backend": "builtin", "position": 1 },
            "emotion": { "type": "emotion", "label": "Emotion", "backend": "builtin", "position": 2 },
            "event": { "type": "event", "label": "Event", "backend": "builtin", "position": 3 },
            "prompt": { "type": "prompt", "label": "Prompt", "backend": "builtin", "position": 4 },
            "llm": { "type": "llm", "label": "LLM", "backend": "ollama", "position": 5 },
            "agent": { "type": "agent", "label": "Agent", "backend": "builtin", "position": 6 },
            "complex_emotion": complex
        }
    });
    std::fs::write(role.join("pipeline.ocblueprint"), blueprint.to_string()).unwrap();
    std::fs::write(role.join("config.json"), "{}").unwrap();
    std::fs::write(role.join("core_personality.txt"), "contract fixture").unwrap();
    std::fs::write(
        role.join("scenes/default/scene.json"),
        r#"{"id":"default","label":"Default"}"#,
    )
    .unwrap();
}

async fn state_with_reply(roles: &Path, reply: &str) -> (AppState, Arc<Mutex<Vec<String>>>) {
    let prompts = Arc::new(Mutex::new(Vec::new()));
    let llm: Arc<dyn LlmClient> = Arc::new(CaptureLlm {
        reply: reply.to_string(),
        prompts: Arc::clone(&prompts),
    });
    let host = HostProfile {
        turn_thinking: TurnThinkingProfile {
            default: TurnThinkingDefault::Deep,
            fast_skip_complex_emotion: false,
            ..TurnThinkingProfile::default()
        },
        ..HostProfile::default()
    };
    let state = AppStateBuilder::in_memory_test(llm, roles.to_path_buf(), None)
        .with_host_profile(host)
        .build()
        .await
        .expect("state");
    (state, prompts)
}

async fn run_turn(state: &AppState, role_id: &str, session_id: &str) {
    process_message(
        state,
        &SendMessageRequest {
            role_id: role_id.to_string(),
            user_message: "我们吵架了".to_string(),
            session_id: Some(session_id.to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("turn");
}

#[tokio::test]
async fn none_backend_updates_emotion_but_neither_reads_nor_writes_hint() {
    const ROLE: &str = "complex.none";
    const SESSION: &str = "none-contract";
    const PRIOR_HINT: &str = "dormant prior hint";
    const NEW_HINT: &str = "disabled backend must discard this";

    let roles = TempDir::new().unwrap();
    write_role(roles.path(), ROLE, "none", None);
    let reply = format!(
        "Fine.\n[EMO]{{\"labels\":[\"anger\"],\"intensity\":0.8,\"narrative_hint\":\"{NEW_HINT}\"}}[/EMO]"
    );
    let (state, prompts) = state_with_reply(roles.path(), &reply).await;
    let srid = conversation_state_role_id(ROLE, Some(SESSION));
    persist_stored_narrative_hint(&state, &srid, PRIOR_HINT.to_string()).await;

    run_turn(&state, ROLE, SESSION).await;

    assert_eq!(
        state
            .db_manager
            .get_current_emotion(&srid)
            .await
            .expect("emotion")
            .as_deref(),
        Some("angry"),
        "[EMO] labels still drive emotion when hint persistence is disabled"
    );
    assert!(
        prompts
            .lock()
            .iter()
            .all(|prompt| !prompt.contains(PRIOR_HINT)),
        "a disabled backend must not inject a dormant stored hint"
    );
    assert_eq!(
        load_stored_narrative_hint(&state, &srid)
            .await
            .expect("stored hint"),
        PRIOR_HINT,
        "a disabled backend must discard the new hint without mutating storage"
    );
}

async fn remote_complex_emotion(Json(body): Json<Value>) -> Json<Value> {
    let id = body.get("id").cloned().unwrap_or(json!(1));
    assert_eq!(
        body.get("method").and_then(Value::as_str),
        Some("complex_emotion.resolve_turn")
    );
    Json(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "source": "remote_contract_test",
            "narrative_hint": "remote hint",
            "labels": ["anger", "sadness"],
            "pattern": "resentful_sad",
            "confidence": 0.9,
            "intensity": 0.8,
            "dissonance_score": 0.8,
            "degraded_to_builtin": false
        }
    }))
}

#[tokio::test(flavor = "multi_thread")]
async fn remote_fallback_labels_drive_current_and_event_emotion() {
    let sidecar = Router::new().route("/", post(remote_complex_emotion));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let sidecar_task = tokio::spawn(async move {
        axum::serve(listener, sidecar).await.unwrap();
    });

    const ROLE: &str = "complex.remote";
    const SESSION: &str = "remote-contract";
    let roles = TempDir::new().unwrap();
    let url = format!("http://{addr}/");
    write_role(roles.path(), ROLE, "remote", Some(&url));
    let (state, _prompts) = state_with_reply(roles.path(), "Reply without an EMO marker").await;
    state
        .high_risk_grants
        .grant_network(NETWORK_GRANT_REMOTE_PLUGIN)
        .expect("grant remote plugin network");

    run_turn(&state, ROLE, SESSION).await;

    let srid = conversation_state_role_id(ROLE, Some(SESSION));
    assert_eq!(
        state
            .db_manager
            .get_current_emotion(&srid)
            .await
            .expect("emotion")
            .as_deref(),
        Some("angry")
    );
    let events = state
        .db_manager
        .get_events(&srid, 10)
        .await
        .expect("events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].bot_emotion, "angry");
    assert_eq!(
        load_stored_narrative_hint(&state, &srid)
            .await
            .expect("stored hint"),
        "remote hint"
    );

    sidecar_task.abort();
}
