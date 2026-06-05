//! A1.2：高流量 `invoke` 对应 **`_impl`** 链式烟测（单进程、Mock LLM，不经 Tauri IPC）。
//!
//! 对照表见仓库根 `handoff/INVOKE_HOTPATH_MATRIX.md`。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use oclivenewnew_tauri::api::directory_plugin::{
    get_directory_plugin_catalog_impl, get_plugin_state_impl,
};
use oclivenewnew_tauri::api::high_risk::{
    grant_high_risk_capability_impl, list_high_risk_grants_impl, revoke_high_risk_capability_impl,
    MutateHighRiskGrantRequest,
};
use oclivenewnew_tauri::api::hotkeys::get_hotkey_bindings_impl;
use oclivenewnew_tauri::api::memory::query_memories_impl;
use oclivenewnew_tauri::api::role::{
    get_role_info_impl, list_roles_impl, load_role_impl, set_session_slot_override_impl,
};
use oclivenewnew_tauri::api::scene::switch_scene_impl;
use oclivenewnew_tauri::api::time::get_time_state_impl;
use oclivenewnew_tauri::domain::chat_engine::process_message;
use oclivenewnew_tauri::infrastructure::MockLlmClient;
use oclivenewnew_tauri::models::dto::SetSessionSlotOverrideRequest;
use oclivenewnew_tauri::models::dto::{
    QueryMemoriesRequest, SendMessageRequest, SwitchSceneRequest,
};
use oclivenewnew_tauri::state::AppState;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;

fn roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles")
}

async fn hotpath_state() -> AppState {
    let llm = Arc::new(MockLlmClient {
        reply: "hotpath".to_string(),
    });
    AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state")
}

#[tokio::test]
async fn invoke_hotpath_smoke_list_load_info_time_chat_memories_catalog_plugin_hotkeys() {
    let state = hotpath_state().await;

    let roles = list_roles_impl(&state).await.expect("list_roles");
    assert!(
        roles.iter().any(|r| r.id == "mumu"),
        "roles/mumu should be listable: {:?}",
        roles.iter().map(|r| &r.id).collect::<Vec<_>>()
    );

    let loaded = load_role_impl(&state, "mumu", true)
        .await
        .expect("load_role");
    assert_eq!(loaded.role_id, "mumu");

    let info = get_role_info_impl(&state, "mumu", None)
        .await
        .expect("get_role_info");
    assert_eq!(info.role_id, "mumu");
    assert!(
        info.slot_registry_pack
            .as_ref()
            .is_some_and(|m| m.contains_key("llm")),
        "mumu v2 blueprint should expose slot_registry_pack"
    );

    let info2 = set_session_slot_override_impl(
        &state,
        &SetSessionSlotOverrideRequest {
            role_id: "mumu".to_string(),
            slot_key: "llm".to_string(),
            backend: Some("remote".to_string()),
            plugin: None,
            plugins: None,
            model: None,
            local_memory_provider_id: None,
            session_id: None,
        },
    )
    .await
    .expect("set_session_slot_override");
    assert!(
        info2
            .slot_session_overridden_keys
            .iter()
            .any(|k| k == "llm"),
        "session override should mark llm"
    );

    let ts = get_time_state_impl(&state, "mumu")
        .await
        .expect("get_time_state");
    assert!(ts.virtual_time_ms > 0 || !ts.iso_datetime.is_empty());

    let chat = process_message(
        &state,
        &SendMessageRequest {
            role_id: "mumu".to_string(),
            user_message: "invoke hotpath".to_string(),
            scene_id: None,
            session_id: None,
        },
    )
    .await
    .expect("send_message / process_message");
    assert_eq!(chat.reply, "hotpath");

    let mems = query_memories_impl(
        &state,
        &QueryMemoriesRequest {
            role_id: "mumu".to_string(),
            limit: 10,
            offset: 0,
        },
    )
    .await
    .expect("query_memories");
    assert!(mems.len() <= 10);

    let catalog = get_directory_plugin_catalog_impl(&state).expect("get_directory_plugin_catalog");
    for e in &catalog {
        assert!(!e.id.is_empty(), "catalog entry id");
        assert!(!e.version.is_empty(), "catalog entry version");
    }
    let catalog_2 = get_directory_plugin_catalog_impl(&state).expect("catalog second call");
    assert_eq!(catalog.len(), catalog_2.len(), "catalog length stable");
    for (a, b) in catalog.iter().zip(catalog_2.iter()) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.version, b.version);
    }

    let plugin = get_plugin_state_impl("mumu", &state).expect("get_plugin_state");
    let _ = (&plugin.role, &plugin.global_defaults);

    let hotkeys = get_hotkey_bindings_impl(&state).expect("get_hotkey_bindings");
    let hotkeys_2 = get_hotkey_bindings_impl(&state).expect("get_hotkey_bindings again");
    assert_eq!(hotkeys, hotkeys_2);
}

#[tokio::test]
async fn invoke_hotpath_role_info_full_shape() {
    let state = hotpath_state().await;
    load_role_impl(&state, "mumu", true)
        .await
        .expect("load_role");

    let info = get_role_info_impl(&state, "mumu", None)
        .await
        .expect("get_role_info");

    assert_eq!(info.role_id, "mumu");
    assert!(!info.role_name.is_empty());
    assert!(!info.version.is_empty());
    assert_eq!(info.personality_vector.len(), 7);
    assert!(!info.scenes.is_empty());
    assert_eq!(info.scenes.len(), info.scene_labels.len());
    assert!(!info.effective_ollama_model.is_empty());
    assert!(info.slot_registry_pack.is_some());
    assert!(info.slot_registry_effective.is_some());
}

#[tokio::test]
async fn invoke_hotpath_list_roles_returns_catalog() {
    let state = hotpath_state().await;
    let roles = list_roles_impl(&state).await.expect("list_roles");
    assert!(!roles.is_empty());
    for r in &roles {
        assert!(!r.id.is_empty());
        assert!(!r.name.is_empty());
    }
}

#[tokio::test]
async fn invoke_hotpath_switch_scene_updates_presence() {
    let state = hotpath_state().await;
    load_role_impl(&state, "mumu", true)
        .await
        .expect("load_role");

    let target = "school";
    let resp = switch_scene_impl(
        &state,
        &SwitchSceneRequest {
            role_id: "mumu".to_string(),
            scene_id: target.to_string(),
            together: false,
        },
    )
    .await
    .expect("switch_scene");

    assert_eq!(resp.role.user_presence_scene.as_deref(), Some(target));
    assert!(resp.role.scenes.iter().any(|s| s == target));
}

#[tokio::test]
async fn invoke_hotpath_high_risk_grants_grant_and_revoke() {
    let state = hotpath_state().await;
    let plugin_id = "com.test.hotpath.plugin";

    let before = list_high_risk_grants_impl(&state).expect("list before");
    assert_grant_absent(&before, plugin_id);

    grant_high_risk_capability_impl(
        &state,
        &MutateHighRiskGrantRequest {
            kind: "process:spawn".to_string(),
            id: plugin_id.to_string(),
        },
    )
    .expect("grant");

    let after_grant = list_high_risk_grants_impl(&state).expect("list after grant");
    assert_grant_present(&after_grant, plugin_id);

    revoke_high_risk_capability_impl(
        &state,
        &MutateHighRiskGrantRequest {
            kind: "process:spawn".to_string(),
            id: plugin_id.to_string(),
        },
    )
    .expect("revoke");

    let after_revoke = list_high_risk_grants_impl(&state).expect("list after revoke");
    assert_grant_absent(&after_revoke, plugin_id);
}

fn assert_grant_present(snapshot: &Value, plugin_id: &str) {
    let arr = snapshot
        .get("process:spawn")
        .and_then(|v| v.as_array())
        .expect("process:spawn array");
    assert!(
        arr.iter().any(|v| v.as_str() == Some(plugin_id)),
        "expected grant for {plugin_id}: {snapshot}"
    );
}

fn assert_grant_absent(snapshot: &Value, plugin_id: &str) {
    let arr = snapshot
        .get("process:spawn")
        .and_then(|v| v.as_array())
        .map(|a| a.as_slice())
        .unwrap_or(&[]);
    assert!(
        !arr.iter().any(|v| v.as_str() == Some(plugin_id)),
        "unexpected grant for {plugin_id}: {snapshot}"
    );
}
