#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

use super::*;
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::state::AppStateBuilder;
use std::sync::Arc;
use tempfile::TempDir;

fn write_role(root: &std::path::Path, id: &str, binding: &str) {
    let role = root.join(id);
    std::fs::create_dir_all(role.join("scenes/default")).unwrap();
    std::fs::create_dir_all(role.join("user_identities")).unwrap();
    let mut slots = serde_json::Map::new();
    for (index, key) in ["memory", "emotion", "event", "prompt", "llm", "agent"]
        .iter()
        .enumerate()
    {
        slots.insert((*key).into(), json!({"type":key,"label":key,"backend":if *key=="llm" {"ollama"} else {"builtin"},"position":index+1,"model":"test-model"}));
    }
    std::fs::write(role.join("pipeline.ocblueprint"), json!({
        "schema_version":2,"meta":{"id":id,"name":id,"version":"0.1.0","author":"test","description":"isolated identity transport fixture",
        "personality":[0.5,0.5,0.5,0.5,0.5,0.5,0.5],"relations":{"classmate":{"initial_favorability":50,"favor_multiplier":1.0,"prompt_hint":"classmate"},"friend":{"initial_favorability":60,"favor_multiplier":1.0,"prompt_hint":"friend"}},"default_relation":"classmate","identity_binding":binding,"scenes":["default"]},
        "slot_registry":slots
    }).to_string()).unwrap();
    std::fs::write(role.join("config.json"), "{}").unwrap();
    std::fs::write(role.join("user_identities/index.json"), json!({
        "schema_version":1,"default_identity_id":"classmate","identities":{
            "classmate":{"display_name":"Classmate","template_file":"classmate.md","maps_to_relation_id":"classmate"},
            "friend":{"display_name":"Friend","template_file":"friend.md","maps_to_relation_id":"friend"}
        }
    }).to_string()).unwrap();
    for id in ["classmate", "friend"] {
        std::fs::write(role.join(format!("user_identities/{id}.md")), id).unwrap();
    }
}

#[tokio::test]
async fn identity_and_relation_use_authenticated_kernel_state_across_clients() {
    let directory = TempDir::new().unwrap();
    let roles = directory.path().join("fixture-role-library");
    write_role(&roles, "global-role", "global");
    write_role(&roles, "scene-role", "per_scene");
    let state = Arc::new(
        AppStateBuilder::in_memory_test(
            Arc::new(MockLlmClient {
                reply: "unused".into(),
            }),
            &roles,
            None,
        )
        .with_app_data_dir(directory.path().join("data"))
        .build()
        .await
        .unwrap(),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = oclive_kernel_host::http_api::api_router_with_auth(
        Arc::clone(&state),
        Some("identity-test-token".into()),
    );
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = stopped.await;
            })
            .await
            .unwrap();
    });
    let conn = KernelConnection::new(format!("http://{address}"), address.port());
    conn.set_api_token("identity-test-token");
    let before = KernelHttpClient::get_user_identity_state_via_http(
        &conn,
        &GetUserIdentityStateRequest {
            role_id: "global-role".into(),
            scene_id: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(before.current_identity_id, "classmate");
    let selected = KernelHttpClient::set_user_identity_via_http(
        &conn,
        &SetUserIdentityRequest {
            role_id: "global-role".into(),
            identity_id: "friend".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(selected.current_identity_id, "friend");
    assert!(!selected.use_manifest_default);
    let second = KernelConnection::new(format!("http://{address}"), address.port());
    second.set_api_token("identity-test-token");
    let restored = KernelHttpClient::get_user_identity_state_via_http(
        &second,
        &GetUserIdentityStateRequest {
            role_id: "global-role".into(),
            scene_id: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(restored.current_identity_id, "friend");
    let relation = KernelHttpClient::set_user_relation_via_http(
        &conn,
        &SetUserRelationRequest {
            role_id: "global-role".into(),
            relation: "classmate".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(relation.current_user_relation, "classmate");
    let scene = KernelHttpClient::set_scene_user_identity_via_http(
        &conn,
        &SetSceneUserIdentityRequest {
            role_id: "scene-role".into(),
            scene_id: "default".into(),
            identity_id: "friend".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(scene.current_identity_id, "friend");
    let scene_relation = KernelHttpClient::set_scene_user_relation_via_http(
        &conn,
        &SetSceneUserRelationRequest {
            role_id: "scene-role".into(),
            scene_id: "default".into(),
            relation: "classmate".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(scene_relation.role_id, "scene-role");
    KernelHttpClient::clear_scene_user_relation_via_http(
        &conn,
        &ClearSceneUserRelationRequest {
            role_id: "scene-role".into(),
            scene_id: "default".into(),
        },
    )
    .await
    .unwrap();
    let cleared = KernelHttpClient::get_user_identity_state_via_http(
        &second,
        &GetUserIdentityStateRequest {
            role_id: "scene-role".into(),
            scene_id: Some("default".into()),
        },
    )
    .await
    .unwrap();
    assert!(cleared.use_manifest_default);
    let rejected = KernelHttpClient::set_user_identity_via_http(
        &conn,
        &SetUserIdentityRequest {
            role_id: "global-role".into(),
            identity_id: "missing".into(),
        },
    )
    .await
    .unwrap_err();
    assert_eq!(rejected.code(), "INVALID_PARAMETER");
    let unauthorized = KernelConnection::new(format!("http://{address}"), address.port());
    unauthorized.set_api_token("wrong-test-token");
    let error = KernelHttpClient::get_user_identity_state_via_http(
        &unauthorized,
        &GetUserIdentityStateRequest {
            role_id: "global-role".into(),
            scene_id: None,
        },
    )
    .await
    .unwrap_err();
    assert_eq!(error.code(), "KERNEL_AUTH_REQUIRED");
    stop.send(()).unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(3), server)
        .await
        .unwrap()
        .unwrap();
    state.directory_plugins.shutdown_all();
    state.db_manager.close_pool().await;
    drop(state);
    directory.close().unwrap();
}
