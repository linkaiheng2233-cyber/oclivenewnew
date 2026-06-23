//! Remote LLM via full `process_message` orchestration (`plugin_backends.llm = remote`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::body::{to_bytes, Body};
use axum::extract::Json;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use axum::Router;
use oclive_kernel_host::domain::slot_runner::SlotRunner;
use oclive_kernel_host::domain::user_llm_env::apply_user_llm_env;
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::state::AppState;
use oclive_kernel_types::models::plugin_backends::LlmBackend;
use oclivenewnew_tauri::http_api::api_router;
use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

async fn mock_llm_handler(Json(body): Json<Value>) -> Json<Value> {
    let method = body.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = body.get("id").cloned().unwrap_or(json!(1));
    let text = if method == "llm.generate_tag" {
        "neutral"
    } else {
        "remote-llm-process-ok"
    };
    Json(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": { "text": text }
    }))
}

fn write_remote_llm_role(dir: &TempDir) -> String {
    let role_id = "llm.remote";
    let role = dir.path().join(role_id);
    std::fs::create_dir_all(role.join("scenes").join("default")).unwrap();
    let bp = serde_json::json!({
        "schema_version": 2,
        "meta": {
            "id": role_id,
            "name": "Remote LLM",
            "version": "0.1.0",
            "author": "t",
            "description": "d",
            "personality": [0.5,0.5,0.5,0.5,0.5,0.5,0.5],
            "relations": { "friend": { "initial_favorability": 50, "favor_multiplier": 1.0, "prompt_hint": "朋友" } },
            "default_relation": "friend",
            "scenes": ["default"]
        },
        "slot_registry": {
            "memory": { "type": "memory", "label": "m", "backend": "builtin", "position": 1 },
            "emotion": { "type": "emotion", "label": "e", "backend": "builtin", "position": 2 },
            "event": { "type": "event", "label": "ev", "backend": "builtin", "position": 3 },
            "prompt": { "type": "prompt", "label": "p", "backend": "builtin", "position": 4 },
            "llm": { "type": "llm", "label": "L", "backend": "remote", "position": 5, "model": "test-model" },
            "agent": { "type": "agent", "label": "a", "backend": "builtin", "position": 6 }
        }
    });
    std::fs::write(role.join("pipeline.ocblueprint"), bp.to_string()).unwrap();
    std::fs::write(role.join("config.json"), "{}").unwrap();
    role_id.to_string()
}

async fn response_json(res: axum::response::Response) -> Value {
    let bytes = to_bytes(res.into_body(), usize::MAX).await.expect("body");
    serde_json::from_slice(&bytes).expect("json")
}

#[tokio::test(flavor = "multi_thread")]
async fn process_message_uses_remote_llm_sidecar_when_configured() {
    let sidecar = Router::new().route("/", post(mock_llm_handler));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let sidecar_task = tokio::spawn(async move {
        axum::serve(listener, sidecar).await.unwrap();
    });

    let url = format!("http://{addr}/");
    let dir = TempDir::new().unwrap();
    let role_id = write_remote_llm_role(&dir);
    let role_path = dir.path().join(&role_id);

    let llm = Arc::new(MockLlmClient {
        reply: "builtin-should-not-win".to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, dir.path().to_path_buf())
            .await
            .expect("AppState"),
    );
    state
        .db_manager
        .upsert_app_setting("user_llm_provider", "cloud")
        .await
        .expect("provider");
    state
        .db_manager
        .upsert_app_setting("user_remote_llm_url", &url)
        .await
        .expect("upsert remote url");
    state
        .db_manager
        .upsert_app_setting("user_llm_cloud_api_style", "oclive_jsonrpc")
        .await
        .expect("upsert cloud style");
    state.mark_user_llm_env_dirty();
    apply_user_llm_env(state.as_ref())
        .await
        .expect("apply user llm env");
    assert_eq!(
        std::env::var("OCLIVE_REMOTE_LLM_URL").expect("remote url env"),
        url
    );

    let role = state.load_role_cached_async(&role_id).await.expect("role");
    let eff = state.effective_plugin_backends_for_session(role.as_ref(), role_id.as_str());
    assert_eq!(eff.llm, LlmBackend::Remote);

    let pl = state.resolved_plugins_for_session(role.as_ref(), Some(role_id.as_str()));
    let primary = SlotRunner::primary_llm(&pl);
    let ollama = state.plugins.llm_for(LlmBackend::Ollama);
    assert!(
        !Arc::ptr_eq(&primary, &ollama),
        "primary LLM must not be the default Ollama client when remote is configured"
    );
    let direct = primary
        .generate("test-model", "hello")
        .await
        .expect("remote llm generate");
    assert_eq!(direct, "remote-llm-process-ok");

    let app = api_router(Arc::clone(&state));
    let body = json!({
        "role_path": role_path.to_string_lossy(),
        "message": "hello remote",
        "scene_id": "default",
    });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/chat")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let v = response_json(res).await;
    assert_eq!(v["reply"], "remote-llm-process-ok");

    sidecar_task.abort();
}
