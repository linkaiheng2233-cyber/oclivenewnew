//! Directory Reply Post-Processor: mock HTTP plugin prefixes display reply.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use async_trait::async_trait;
use oclive_kernel_host::domain::chat_engine::process_message;
use oclive_kernel_host::infrastructure::directory_plugins::DirectoryPluginRuntime;
use oclive_kernel_host::infrastructure::high_risk_grants::HighRiskGrantStore;
use oclive_kernel_host::infrastructure::llm::LlmClient;
use oclive_kernel_host::state::AppState;
use oclive_kernel_types::models::dto::SendMessageRequest;
use oclivenewnew_tauri::error::Result;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

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

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) {
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

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn install_mock_polish_plugin(plugins_dir: &std::path::Path) {
    let src = repo_root().join("examples/directory-plugin-reply-post-process-minimal");
    let dst = plugins_dir.join("reply-post-process-polish");
    copy_dir_all(&src, &dst);
    let manifest_path = dst.join("manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    manifest["schema_version"] = serde_json::json!(1);
    manifest["id"] = serde_json::json!("reply-post-process-polish");
    manifest["permissions"] = serde_json::json!(["process:spawn"]);
    fs::write(
        manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
}

fn write_directory_postproc_role(dir: &TempDir, plugin_id: &str) -> String {
    let role_id = "pp.directory";
    let role = dir.path().join(role_id);
    fs::create_dir_all(role.join("scenes").join("default")).unwrap();
    let bp = serde_json::json!({
        "schema_version": 2,
        "meta": {
            "id": role_id,
            "name": "PP Directory",
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
            "directory": { "plugin_id": plugin_id },
            "builtin": { "profile": "standard", "max_chars": 2000, "strip_leading_quote": false }
        }
    });
    fs::write(role.join("config.json"), cfg.to_string()).unwrap();
    role_id.to_string()
}

#[tokio::test(flavor = "multi_thread")]
async fn directory_post_processor_applies_mock_plugin_prefix() {
    let dir = TempDir::new().unwrap();
    let role_id = write_directory_postproc_role(&dir, "reply-post-process-polish");
    let raw = "hello from llm";
    let llm: Arc<dyn LlmClient> = Arc::new(FixedReplyLlm {
        reply: raw.to_string(),
    });
    let roles_dir = dir.path().to_path_buf();
    let app_data_plugins = roles_dir
        .join(".oclive_directory_plugin_data")
        .join("plugins");
    install_mock_polish_plugin(&app_data_plugins);

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
            role_id,
            user_message: "hi".to_string(),
            scene_id: Some("default".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("chat");

    assert_eq!(
        resp.reply,
        format!("[dir-pp] {}", raw),
        "directory plugin should prefix display reply"
    );
}

#[test]
fn directory_spawn_starts_mock_polish_plugin() {
    let dir = TempDir::new().unwrap();
    let roles_dir = dir.path().join("roles");
    let app_data = dir.path().join("app_data");
    fs::create_dir_all(&roles_dir).unwrap();
    fs::create_dir_all(&app_data).unwrap();

    install_mock_polish_plugin(&app_data.join("plugins"));

    let grants = HighRiskGrantStore::load(app_data.clone(), false);
    grants
        .grant_process_spawn("reply-post-process-polish")
        .unwrap();

    let rt = DirectoryPluginRuntime::bootstrap(&roles_dir, &app_data, grants);
    rt.rescan_plugin_roots(&roles_dir);

    let url = rt
        .ensure_rpc_url_for_debug("reply-post-process-polish", None)
        .expect("spawn plugin");
    assert!(url.starts_with("http://127.0.0.1:"));
}
