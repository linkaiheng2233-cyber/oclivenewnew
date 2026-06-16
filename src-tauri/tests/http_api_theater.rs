//! `POST /theater/scene` integration test (`tower::ServiceExt::oneshot`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use oclivenewnew_tauri::http_api::api_router;
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::state::AppState;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceExt;

fn roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles")
}

async fn response_json(res: axum::response::Response) -> Value {
    let bytes = to_bytes(res.into_body(), usize::MAX).await.expect("body");
    serde_json::from_slice(&bytes).expect("json")
}

fn sample_scene_body() -> Value {
    json!({
        "cast_a": { "role_id": "mumu", "name": "木木" },
        "cast_b": { "role_id": "枫侵月", "name": "枫侵月" },
        "scene_id": "home",
        "base_beats": [
            {
                "id": "b1",
                "cast": "b",
                "name": "枫侵月",
                "text": "粥还要不要温一下？"
            }
        ],
        "applied_tweaks": [],
        "fallback_beats": [
            {
                "id": "b1",
                "cast": "b",
                "name": "枫侵月",
                "text": "粥还要不要温一下？"
            }
        ]
    })
}

#[tokio::test]
async fn http_api_theater_scene_ok_with_mock_llm() {
    let llm = Arc::new(MockLlmClient {
        reply: r#"[{"id":"b1","cast":"b","name":"枫侵月","text":"模拟重写台词。"}]"#.to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let body = sample_scene_body();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/theater/scene")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let v = response_json(res).await;
    assert_eq!(v["source"], "local");
    assert_eq!(v["beats"][0]["text"], "模拟重写台词。");
    assert!(v["model"].as_str().is_some());
}

#[tokio::test]
async fn http_api_theater_scene_alternate_cast_b_ok() {
    let llm = Arc::new(MockLlmClient {
        reply: r#"[{"id":"b1","cast":"b","name":"枫侵月","text":"换角后台词。"}]"#.to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let mut body = sample_scene_body();
    body["cast_b"] = json!({ "role_id": "枫侵月", "name": "枫侵月" });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/theater/scene")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let v = response_json(res).await;
    assert_eq!(v["beats"][0]["text"], "换角后台词。");
    assert_eq!(v["source"], "local");
}

#[tokio::test]
async fn http_api_theater_scene_empty_base_beats_400() {
    let llm = Arc::new(MockLlmClient {
        reply: "[]".to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let mut body = sample_scene_body();
    body["base_beats"] = json!([]);
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/theater/scene")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let v = response_json(res).await;
    assert_eq!(v["error"]["code"], "THEATER_SCENE_GEN_FAILED");
}

#[tokio::test]
async fn http_api_theater_scene_ripple_preserves_prefix() {
    let llm = Arc::new(MockLlmClient {
        reply: r#"[{"id":"r1","cast":"a","name":"木木","text":"涟漪改写。"}]"#.to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let body = json!({
        "cast_a": { "role_id": "mumu", "name": "木木" },
        "cast_b": { "role_id": "枫侵月", "name": "枫侵月" },
        "scene_id": "home",
        "base_beats": [
            { "id": "b1", "cast": "b", "name": "枫侵月", "text": "前缀台词。" },
            { "id": "b2", "cast": "a", "name": "木木", "text": "原后缀。" }
        ],
        "applied_tweaks": [{
            "kind": "chip",
            "chip_label": "喝茶",
            "drama_seed": "苦药变笑料",
            "insert_after_beat_id": "b1",
            "lead_cast": "a"
        }],
        "fallback_beats": [
            { "id": "b1", "cast": "b", "name": "枫侵月", "text": "前缀台词。" },
            { "id": "p1", "cast": "a", "name": "木木", "text": "罐头补丁。" },
            { "id": "b2", "cast": "a", "name": "木木", "text": "原后缀。" }
        ]
    });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/theater/scene")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let v = response_json(res).await;
    assert_eq!(v["beats"][0]["text"], "前缀台词。");
    assert_eq!(v["beats"][1]["text"], "罐头补丁。");
    assert_eq!(v["beats"][2]["text"], "涟漪改写。");
    assert_eq!(v["source"], "local");
}

#[tokio::test]
async fn http_api_theater_scene_cast_adapt_ok() {
    let llm = Arc::new(MockLlmClient {
        reply: r#"{"beats":[{"id":"b1","cast":"b","name":"小枫","text":"非默认卡司开场。"}],"forks":[{"chip_id":"tea","patch_lines":[{"id":"tea-1","cast":"b","name":"小枫","text":"适配罐头。"}]}]}"#.to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let body = json!({
        "cast_a": { "role_id": "custom-a", "name": "小木" },
        "cast_b": { "role_id": "custom-b", "name": "小枫" },
        "scene_id": "home",
        "mode": "cast_adapt",
        "base_beats": [
            {
                "id": "b1",
                "cast": "b",
                "name": "小枫",
                "text": "换名 baseline。"
            }
        ],
        "applied_tweaks": [],
        "fallback_beats": [
            {
                "id": "b1",
                "cast": "b",
                "name": "小枫",
                "text": "换名 baseline。"
            }
        ],
        "fork_templates": [{
            "chip_id": "tea",
            "insert_after_beat_id": "b6",
            "patch_lines": [{
                "id": "tea-1",
                "cast": "b",
                "name": "小枫",
                "text": "罐头 baseline。"
            }]
        }]
    });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/theater/scene")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let v = response_json(res).await;
    assert_eq!(v["beats"][0]["id"], "b1");
    assert_eq!(v["beats"][0]["text"], "非默认卡司开场。");
    assert_eq!(v["adapted_forks"][0]["chip_id"], "tea");
    assert_eq!(v["adapted_forks"][0]["insert_after_beat_id"], "b6");
    assert_eq!(v["adapted_forks"][0]["patch_lines"][0]["text"], "适配罐头。");
    assert_eq!(v["source"], "local");
}

#[tokio::test]
async fn http_api_theater_scene_cast_rewrite_ok() {
    let llm = Arc::new(MockLlmClient {
        reply: r#"{"beats":[
          {"id":"b1","cast":"b","name":"诗梦","text":"……烦死了，自己不会热吗。"},
          {"id":"b2","cast":"a","name":"木木","text":"要你管。"},
          {"id":"b3","cast":"b","name":"诗梦","text":"快吃，要迟到了。"},
          {"id":"b4","cast":"a","name":"木木","text":"知道了。"},
          {"id":"b5","cast":"b","name":"诗梦","text":"伞在玄关。"},
          {"id":"b6","cast":"a","name":"木木","text":"……哦。"}
        ],"forks":[
          {"chip_id":"tea","insert_after_beat_id":"b4","patch_lines":[
            {"id":"tea-1","cast":"b","name":"诗梦","text":"把这杯苦药喝了。"},
            {"id":"tea-2","cast":"a","name":"木木","text":"变态！"}
          ]},
          {"chip_id":"late","insert_after_beat_id":"b4","patch_lines":[
            {"id":"late-1","cast":"b","name":"诗梦","text":"糟了要迟到。"}
          ]},
          {"chip_id":"biteTongue","insert_after_beat_id":"b4","patch_lines":[
            {"id":"bt-1","cast":"a","name":"木木","text":"呜！"}
          ]},
          {"chip_id":"nickname","insert_after_beat_id":"b4","patch_lines":[
            {"id":"nick-1","cast":"a","name":"木木","text":"小诗诗？"}
          ]}
        ]}"#.to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let body = json!({
        "cast_a": { "role_id": "mumu", "name": "木木" },
        "cast_b": { "role_id": "shimeng", "name": "诗梦" },
        "scene_id": "home",
        "base_beats": [],
        "applied_tweaks": [],
        "fallback_beats": [
            { "id": "b1", "cast": "b", "name": "枫侵月", "text": "fallback" }
        ],
        "mode": "cast_rewrite",
        "poke_chips": [
            { "chip_id": "tea", "drama_seed": "苦药变笑料" },
            { "chip_id": "late", "drama_seed": "快迟到" },
            { "chip_id": "biteTongue", "drama_seed": "咬舌头" },
            { "chip_id": "nickname", "drama_seed": "新称呼" }
        ]
    });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/theater/scene")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let v = response_json(res).await;
    assert_eq!(v["beats"][0]["text"], "……烦死了，自己不会热吗。");
    assert_eq!(v["adapted_forks"][0]["chip_id"], "tea");
    assert_eq!(v["source"], "local");
}

#[tokio::test]
async fn http_api_theater_scene_fallback_on_bad_llm_json() {
    let llm = Arc::new(MockLlmClient {
        reply: "not json at all".to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let body = sample_scene_body();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/theater/scene")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let v = response_json(res).await;
    assert_eq!(v["source"], "fallback");
    assert_eq!(v["beats"][0]["text"], "粥还要不要温一下？");
}
