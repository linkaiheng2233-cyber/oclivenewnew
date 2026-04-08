//! `role_cache` 与磁盘知识文件：`load_role` 后应对话使用最新 `knowledge_index`。
//! 覆盖「包内知识更新后行为可观测」类需求（路线图第 5 月：换知识内容 / 版本后同一追问应反映新正文）。

use oclivenewnew_tauri::api::role::{get_role_info_impl, load_role_impl};
use oclivenewnew_tauri::domain::chat_engine::process_message;
use oclivenewnew_tauri::infrastructure::MockLlmClient;
use oclivenewnew_tauri::models::dto::SendMessageRequest;
use oclivenewnew_tauri::state::AppState;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;

const ROLE_ID: &str = "kcache_r1";
const CACHE_OLD: &str = "KCACHE_MARKER_OLD";
const CACHE_NEW: &str = "KCACHE_MARKER_NEW";

fn write_minimal_role(root: &Path) {
    let role_dir = root.join(ROLE_ID);
    fs::create_dir_all(role_dir.join("knowledge")).unwrap();
    let manifest = format!(
        r#"{{
        "id": "{rid}",
        "name": "K",
        "version": "1",
        "author": "t",
        "description": "d",
        "default_personality": [0.5,0.5,0.5,0.5,0.5,0.5,0.5],
        "scenes": [],
        "user_relations": {{
            "friend": {{
                "display_name": "F",
                "prompt_hint": "h",
                "favor_multiplier": 1.0,
                "initial_favorability": 50.0
            }}
        }},
        "default_relation": "friend",
        "knowledge": {{ "enabled": true, "glob": "knowledge/**/*.md" }}
    }}"#,
        rid = ROLE_ID
    );
    fs::write(role_dir.join("manifest.json"), manifest).unwrap();
    fs::write(
        role_dir.join("knowledge/lore.md"),
        format!(
            "---\nid: lore1\ntags: []\n---\n\n{CACHE_OLD} 雾城设定。",
            CACHE_OLD = CACHE_OLD
        ),
    )
    .unwrap();
}

#[tokio::test]
async fn load_role_updates_role_cache_after_knowledge_file_change() {
    let tmp = tempdir().unwrap();
    let roles = tmp.path().to_path_buf();
    write_minimal_role(&roles);

    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, &roles)
        .await
        .expect("state");

    load_role_impl(&state, ROLE_ID, true)
        .await
        .expect("load_role");

    let req = SendMessageRequest {
        role_id: ROLE_ID.to_string(),
        user_message: "讲讲雾城".to_string(),
        scene_id: None,
        session_id: None,
    };
    process_message(&state, &req)
        .await
        .expect("process_message");

    let cached = state
        .role_cache
        .read()
        .get(ROLE_ID)
        .cloned()
        .expect("role_cache populated after send_message");
    let body_old = cached
        .knowledge_index
        .as_ref()
        .and_then(|i| i.chunks.first().map(|c| c.body.as_str()))
        .expect("knowledge chunk");
    assert!(
        body_old.contains(CACHE_OLD),
        "expected initial marker in cached knowledge body"
    );
    assert!(!body_old.contains(CACHE_NEW));

    let info = get_role_info_impl(&state, ROLE_ID)
        .await
        .expect("get_role_info");
    assert!(info.knowledge_enabled);
    assert_eq!(info.knowledge_chunk_count, 1);

    let role_dir = roles.join(ROLE_ID);
    fs::write(
        role_dir.join("knowledge/lore.md"),
        format!(
            "---\nid: lore1\ntags: []\n---\n\n{CACHE_NEW} 雾城新设定。",
            CACHE_NEW = CACHE_NEW
        ),
    )
    .unwrap();

    load_role_impl(&state, ROLE_ID, false)
        .await
        .expect("load_role after disk change");

    let cached2 = state
        .role_cache
        .read()
        .get(ROLE_ID)
        .cloned()
        .expect("role_cache after reload");
    let body_new = cached2
        .knowledge_index
        .as_ref()
        .and_then(|i| i.chunks.first().map(|c| c.body.as_str()))
        .expect("knowledge chunk after reload");
    assert!(
        body_new.contains(CACHE_NEW),
        "send_message path must see knowledge_index refreshed by load_role"
    );
    assert!(!body_new.contains(CACHE_OLD));

    let info2 = get_role_info_impl(&state, ROLE_ID)
        .await
        .expect("get_role_info after reload");
    assert!(info2.knowledge_enabled);
    assert_eq!(info2.knowledge_chunk_count, 1);

    process_message(&state, &req)
        .await
        .expect("process_message after reload");
    let cached3 = state
        .role_cache
        .read()
        .get(ROLE_ID)
        .cloned()
        .expect("cache after second send");
    let body_after_send = cached3
        .knowledge_index
        .as_ref()
        .and_then(|i| i.chunks.first().map(|c| c.body.as_str()))
        .expect("knowledge chunk");
    assert!(
        body_after_send.contains(CACHE_NEW),
        "co_present path must keep using refreshed index after send_message"
    );
}
