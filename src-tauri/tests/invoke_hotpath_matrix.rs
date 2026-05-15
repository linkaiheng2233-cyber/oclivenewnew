//! A1.2：高流量 `invoke` 对应 **`_impl`** 链式烟测（单进程、Mock LLM，不经 Tauri IPC）。
//!
//! 对照表见仓库根 `handoff/INVOKE_HOTPATH_MATRIX.md`。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use oclivenewnew_tauri::api::directory_plugin::{
    get_directory_plugin_catalog_impl, get_plugin_state_impl,
};
use oclivenewnew_tauri::api::hotkeys::get_hotkey_bindings_impl;
use oclivenewnew_tauri::api::memory::query_memories_impl;
use oclivenewnew_tauri::api::role::{get_role_info_impl, list_roles_impl, load_role_impl};
use oclivenewnew_tauri::api::time::get_time_state_impl;
use oclivenewnew_tauri::domain::chat_engine::process_message;
use oclivenewnew_tauri::infrastructure::MockLlmClient;
use oclivenewnew_tauri::models::dto::{QueryMemoriesRequest, SendMessageRequest};
use oclivenewnew_tauri::state::AppState;
use std::path::PathBuf;
use std::sync::Arc;

fn roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles")
}

#[tokio::test]
async fn invoke_hotpath_smoke_list_load_info_time_chat_memories_catalog_plugin_hotkeys() {
    let llm = Arc::new(MockLlmClient {
        reply: "hotpath".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");

    let roles = list_roles_impl(&state).await.expect("list_roles");
    assert!(
        roles.iter().any(|r| r.id == "mumu"),
        "roles/mumu should be listable: {:?}",
        roles.iter().map(|r| &r.id).collect::<Vec<_>>()
    );

    let loaded = load_role_impl(&state, "mumu", true).await.expect("load_role");
    assert_eq!(loaded.role_id, "mumu");

    let info = get_role_info_impl(&state, "mumu", None)
        .await
        .expect("get_role_info");
    assert_eq!(info.role_id, "mumu");

    let ts = get_time_state_impl(&state, "mumu").await.expect("get_time_state");
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
