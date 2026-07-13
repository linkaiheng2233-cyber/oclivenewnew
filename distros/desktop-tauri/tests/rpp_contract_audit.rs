//! K-RPP-01: Reply Post-Processor directory RPC contract audit (PLUGIN_V1 · `reply_post_process.process`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use async_trait::async_trait;
use oclive_kernel_host::domain::chat_engine::process_message;
use oclive_kernel_host::infrastructure::llm::LlmClient;
use oclive_kernel_host::state::AppState;
use oclive_kernel_types::models::dto::SendMessageRequest;
use oclivenewnew_tauri::error::Result;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

const RPP_METHOD: &str = "reply_post_process.process";

struct FixedReplyLlm {
    reply: String,
}

#[async_trait]
impl LlmClient for FixedReplyLlm {
    async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok(self.reply.clone())
    }

    async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok("neutral".to_string())
    }

    async fn startup_probe(&self) -> Result<()> {
        Ok(())
    }
}

fn copy_dir_all(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for ent in fs::read_dir(src).unwrap() {
        let ent = ent.unwrap();
        let ty = ent.file_type().unwrap();
        let to = dst.join(ent.file_name());
        if ty.is_dir() {
            copy_dir_all(&ent.path(), &to);
        } else {
            fs::copy(ent.path(), to).unwrap();
        }
    }
}

fn install_mock_rpp_plugin(plugins_dir: &Path) {
    let src = common::monorepo_root().join("examples/directory-plugin-reply-post-process-minimal");
    let dst = plugins_dir.join("reply-post-process-polish");
    copy_dir_all(&src, &dst);
    let manifest_path = dst.join("manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    manifest["schema_version"] = serde_json::json!(1);
    manifest["id"] = serde_json::json!("reply-post-process-polish");
    manifest["provides"] = serde_json::json!(["reply_post_process"]);
    manifest["permissions"] = serde_json::json!(["process:spawn"]);
    fs::write(
        manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
}

fn write_rpp_role(dir: &TempDir) -> String {
    let role_id = "rpp.contract";
    let role = dir.path().join(role_id);
    fs::create_dir_all(role.join("scenes/default")).unwrap();
    let bp = serde_json::json!({
        "schema_version": 2,
        "meta": {
            "id": role_id,
            "name": "RPP Contract",
            "version": "0.1.0",
            "author": "t",
            "description": "d",
            "personality": [0.5,0.5,0.5,0.5,0.5,0.5,0.5],
            "relations": { "friend": { "initial_favorability": 50, "favor_multiplier": 1.0, "prompt_hint": "朋友" } },
            "default_relation": "friend",
            "scenes": ["default"]
        },
        "slot_registry": {
            "llm": { "type": "llm", "label": "L", "backend": "ollama", "position": 1 }
        }
    });
    fs::write(role.join("pipeline.ocblueprint"), bp.to_string()).unwrap();
    let cfg = serde_json::json!({
        "reply_post_processor": {
            "enabled": true,
            "backend": "directory",
            "directory": { "plugin_id": "reply-post-process-polish" },
            "builtin": { "profile": "standard", "max_chars": 2000, "strip_leading_quote": false }
        }
    });
    fs::write(role.join("config.json"), cfg.to_string()).unwrap();
    role_id.to_string()
}

#[test]
fn mock_plugin_declares_reply_post_process_capability() {
    let manifest_path = common::monorepo_root()
        .join("examples/directory-plugin-reply-post-process-minimal/manifest.json");
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(manifest_path).unwrap()).unwrap();
    let provides = manifest["provides"].as_array().expect("provides array");
    assert!(
        provides
            .iter()
            .any(|v| v.as_str() == Some("reply_post_process")),
        "directory mock must declare provides reply_post_process"
    );
}

#[test]
fn mock_rpc_server_exposes_process_method() {
    let rpc = fs::read_to_string(
        common::monorepo_root()
            .join("examples/directory-plugin-reply-post-process-minimal/rpc_server.mjs"),
    )
    .unwrap();
    assert!(
        rpc.contains(RPP_METHOD),
        "rpc_server must implement {RPP_METHOD}"
    );
    assert!(
        rpc.contains("display_reply"),
        "result must include display_reply"
    );
    assert!(rpc.contains("raw_reply"), "params must accept raw_reply");
}

#[tokio::test(flavor = "multi_thread")]
async fn directory_rpp_process_returns_display_reply_contract() {
    let dir = TempDir::new().unwrap();
    let role_id = write_rpp_role(&dir);
    let raw = "contract raw line";
    let llm: Arc<dyn LlmClient> = Arc::new(FixedReplyLlm {
        reply: raw.to_string(),
    });
    let roles_dir = dir.path().to_path_buf();
    let plugins_dir = roles_dir
        .join(".oclive_directory_plugin_data")
        .join("plugins");
    install_mock_rpp_plugin(&plugins_dir);

    let state = AppState::new_in_memory_with_llm(llm, roles_dir)
        .await
        .expect("state");
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    state
        .high_risk_grants
        .grant_process_spawn("reply-post-process-polish")
        .expect("grant spawn");

    let resp = process_message(
        &state,
        &SendMessageRequest {
            role_id: role_id.clone(),
            user_message: "ping".into(),
            scene_id: Some("default".into()),
            session_id: None,
            include_raw_reply: Some(true),
        },
    )
    .await
    .expect("send");

    assert!(
        resp.reply.starts_with("[dir-pp] "),
        "directory RPP must prefix display_reply; got {:?}",
        resp.reply
    );
    assert_eq!(
        resp.raw_reply.as_deref(),
        Some(raw),
        "include_raw_reply must surface pre-RPP text"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn mumu_reply_post_processor_disabled_by_default() {
    let llm: Arc<dyn LlmClient> = Arc::new(FixedReplyLlm { reply: "ok".into() });
    let state = AppState::new_in_memory_with_llm(llm, common::roles_dir())
        .await
        .expect("state");
    let role = state.load_role_cached_async("mumu").await.expect("mumu");
    assert!(
        !role.pack_reply_post_processor_config.enabled,
        "golden mumu pack must keep reply_post_processor.enabled=false"
    );
}
