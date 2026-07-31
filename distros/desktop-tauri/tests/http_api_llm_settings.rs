//! `--api` HTTP LLM settings route integration tests.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::state::AppState;
use oclivenewnew_tauri::http_api::api_router;
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

async fn response_json(res: axum::response::Response) -> Value {
    let bytes = to_bytes(res.into_body(), usize::MAX).await.expect("body");
    serde_json::from_slice(&bytes).expect("json")
}

#[tokio::test]
async fn http_api_llm_user_settings_get_ok() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, common::roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let res = app
        .oneshot(
            Request::builder()
                .uri("/llm/user_settings?role_id=mumu")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let v = response_json(res).await;
    assert!(v.get("provider").is_some());
    assert!(v.get("effectiveModel").is_some());
    assert!(v.get("ollamaBaseUrl").is_some());
    assert!(v.get("localRuntimeMode").is_some());
    assert!(v.get("localModelPath").is_some());
    assert!(v.get("localLoraAdapters").is_some_and(Value::is_array));
    assert!(v.get("activeLocalLoraAdapterId").is_some());
    assert!(v.get("performanceActiveBackend").is_some());
}

#[tokio::test]
async fn http_api_lora_activation_rejects_unconfigured_runtime() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, common::roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let body = json!({
        "adapterId": null,
        "adultContentAcknowledged": false,
    });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/llm/lora/activate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn http_api_llm_session_model_post_ok() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, common::roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(Arc::clone(&state));
    let body = json!({
        "roleId": "mumu",
        "model": "llama3.2:latest",
    });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/llm/session_model")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let v = response_json(res).await;
    assert_eq!(v["role_id"], "mumu");
}

#[tokio::test]
async fn http_api_llm_user_settings_post_local_ok() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, common::roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let body = json!({
        "roleId": "mumu",
        "provider": "local",
        "ollamaBaseUrl": "http://127.0.0.1:11434",
    });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/llm/user_settings")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let v = response_json(res).await;
    assert_eq!(v["role_id"], "mumu");
}

#[tokio::test]
async fn http_api_llm_user_settings_keeps_gguf_path_separate_from_ollama_model() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, common::roles_dir())
            .await
            .expect("state"),
    );
    let temp = tempfile::tempdir().unwrap();
    let model_path = temp.path().join("qwen-7b.gguf");
    std::fs::write(&model_path, b"test-gguf").unwrap();
    let app = api_router(Arc::clone(&state));
    let body = json!({
        "roleId": "mumu",
        "provider": "local",
        "localModelPath": model_path.to_string_lossy(),
        "ollamaModel": "qwen2.5:7b",
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/llm/user_settings")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);

    let res = app
        .oneshot(
            Request::builder()
                .uri("/llm/user_settings?role_id=mumu")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let settings = response_json(res).await;
    assert_eq!(
        settings["localModelPath"],
        model_path.to_string_lossy().as_ref()
    );
    assert_eq!(settings["sessionOllamaModel"], "qwen2.5:7b");
    std::env::remove_var("OCLIVE_LOCAL_LLM_MODEL_PATH");
}

#[tokio::test]
async fn http_api_llm_ollama_models_get_ok_or_unreachable() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, common::roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let res = app
        .oneshot(
            Request::builder()
                .uri("/llm/ollama_models")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("oneshot");
    // Ollama may be offline in CI; route should still respond (500 if unreachable).
    assert!(
        res.status() == StatusCode::OK || res.status() == StatusCode::INTERNAL_SERVER_ERROR,
        "unexpected status {}",
        res.status()
    );
}
