//! Per-scene User Identity: `scene_set` with sentinel clears scene override.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use async_trait::async_trait;
use oclive_kernel_host::infrastructure::llm::LlmClient;
use oclive_kernel_host::service::role::identity::{
    get_user_identity_state_impl, set_scene_user_identity_impl,
};
use oclive_kernel_host::state::AppState;
use oclive_kernel_types::models::dto::{
    GetUserIdentityStateRequest, SetSceneUserIdentityRequest, OCLIVE_DEFAULT_IDENTITY_SENTINEL,
};
use oclivenewnew_tauri::error::Result;
use std::io::Write;
use std::sync::Arc;
use tempfile::TempDir;

struct StubLlm;

#[async_trait]
impl LlmClient for StubLlm {
    async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok("ok".to_string())
    }

    async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok("neutral".to_string())
    }

    async fn startup_probe(&self) -> Result<()> {
        Ok(())
    }
}

fn write_per_scene_identity_role(dir: &TempDir) -> String {
    let role_id = "test.identity.per_scene";
    let role = dir.path().join(role_id);
    std::fs::create_dir_all(role.join("scenes").join("default")).unwrap();
    std::fs::create_dir_all(role.join("user_identities")).unwrap();

    let bp = serde_json::json!({
        "schema_version": 2,
        "meta": {
            "id": role_id,
            "name": "Per-scene identity",
            "version": "0.1.0",
            "author": "t",
            "description": "d",
            "personality": [0.5,0.5,0.5,0.5,0.5,0.5,0.5],
            "relations": {
                "classmate": { "initial_favorability": 50, "favor_multiplier": 1.0, "prompt_hint": "同学" },
                "friend": { "initial_favorability": 60, "favor_multiplier": 1.0, "prompt_hint": "朋友" }
            },
            "default_relation": "classmate",
            "identity_binding": "per_scene",
            "scenes": ["default"]
        },
        "slot_registry": {
            "memory": { "type": "memory", "label": "m", "backend": "builtin", "position": 1 },
            "emotion": { "type": "emotion", "label": "e", "backend": "builtin", "position": 2 },
            "event": { "type": "event", "label": "ev", "backend": "builtin", "position": 3 },
            "prompt": { "type": "prompt", "label": "p", "backend": "builtin", "position": 4 },
            "llm": { "type": "llm", "label": "L", "backend": "ollama", "position": 5, "model": "test-model" },
            "agent": { "type": "agent", "label": "a", "backend": "builtin", "position": 6 }
        }
    });
    std::fs::write(role.join("pipeline.ocblueprint"), bp.to_string()).unwrap();
    std::fs::write(role.join("config.json"), "{}").unwrap();
    std::fs::write(
        role.join("user_identities/index.json"),
        serde_json::json!({
            "schema_version": 1,
            "default_identity_id": "classmate",
            "identities": {
                "classmate": {
                    "display_name": "同班同学",
                    "template_file": "classmate.md",
                    "maps_to_relation_id": "classmate"
                },
                "friend": {
                    "display_name": "朋友",
                    "template_file": "friend.md",
                    "maps_to_relation_id": "friend"
                }
            }
        })
        .to_string(),
    )
    .unwrap();
    let mut f = std::fs::File::create(role.join("user_identities/classmate.md")).unwrap();
    f.write_all(b"classmate identity template").unwrap();
    let mut f = std::fs::File::create(role.join("user_identities/friend.md")).unwrap();
    f.write_all(b"friend identity template").unwrap();
    role_id.to_string()
}

#[tokio::test]
async fn per_scene_sentinel_clears_scene_override() {
    let dir = TempDir::new().unwrap();
    let role_id = write_per_scene_identity_role(&dir);
    let llm: Arc<dyn LlmClient> = Arc::new(StubLlm);
    let state = AppState::new_in_memory_with_llm(llm, dir.path().to_path_buf())
        .await
        .expect("state");

    let scene_id = "default";
    let after_set = set_scene_user_identity_impl(
        &state,
        &SetSceneUserIdentityRequest {
            role_id: role_id.clone(),
            scene_id: scene_id.to_string(),
            identity_id: "friend".to_string(),
        },
    )
    .await
    .expect("set friend");
    assert_eq!(after_set.current_identity_id, "friend");
    assert!(!after_set.use_manifest_default);

    let after_clear = set_scene_user_identity_impl(
        &state,
        &SetSceneUserIdentityRequest {
            role_id: role_id.clone(),
            scene_id: scene_id.to_string(),
            identity_id: OCLIVE_DEFAULT_IDENTITY_SENTINEL.to_string(),
        },
    )
    .await
    .expect("clear with sentinel");
    assert_eq!(after_clear.default_identity_id, "classmate");
    assert!(after_clear.use_manifest_default);
    assert_eq!(after_clear.current_identity_id, "classmate");

    let db_override = state
        .db_manager
        .get_user_identity_id_for_scene(&role_id, scene_id)
        .await
        .expect("db");
    assert!(db_override.is_none());
}

#[tokio::test]
async fn global_binding_rejects_scene_set() {
    let roles_dir = common::roles_dir();
    let llm: Arc<dyn LlmClient> = Arc::new(StubLlm);
    let state = AppState::new_in_memory_with_llm(llm, roles_dir)
        .await
        .expect("state");

    let err = set_scene_user_identity_impl(
        &state,
        &SetSceneUserIdentityRequest {
            role_id: "mumu".to_string(),
            scene_id: "home".to_string(),
            identity_id: "classmate".to_string(),
        },
    )
    .await
    .expect_err("global binding should reject scene_set");

    let msg = err.to_string();
    assert!(
        msg.contains("global identity_binding"),
        "unexpected error: {msg}"
    );

    let state_ok = get_user_identity_state_impl(
        &state,
        &GetUserIdentityStateRequest {
            role_id: "mumu".to_string(),
            scene_id: Some("home".to_string()),
        },
    )
    .await
    .expect("get state");
    assert!(!state_ok.identities.is_empty());
}
