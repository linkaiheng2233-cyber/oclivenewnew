//! 受限 `PARALLEL`：仅 READ_ONLY 子步骤；`process_message` 烟测。

use oclive_kernel_runtime::domain::chat_engine::pipeline_loader::PIPELINE_BLUEPRINT_FILENAME;
use oclive_kernel_runtime::domain::chat_engine::process_message;
use oclive_kernel_runtime::infrastructure::llm::MockLlmClient;
use oclive_kernel_runtime::models::dto::SendMessageRequest;
use oclive_kernel_runtime::state::KernelAppState;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn workspace_roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roles")
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            if let Some(p) = to.parent() {
                fs::create_dir_all(p)?;
            }
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn isolated_roles_shimeng_clone(role_dir_name: &str) -> tempfile::TempDir {
    let src = workspace_roles_dir().join("shimeng");
    assert!(src.join("manifest.json").is_file(), "need roles/shimeng");
    let tmp = tempfile::tempdir().expect("tempdir");
    let dest = tmp.path().join(role_dir_name);
    copy_dir_recursive(&src, &dest).expect("copy");
    let manifest_path = dest.join("manifest.json");
    let raw = fs::read_to_string(&manifest_path).expect("read manifest");
    let mut v: serde_json::Value = serde_json::from_str(&raw).expect("json");
    v["id"] = serde_json::Value::String(role_dir_name.to_string());
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&v).expect("serialize"),
    )
    .expect("write manifest");
    tmp
}

fn blueprint_with_parallel_readonly() -> String {
    r#"{
  "schemaVersion": "1.0",
  "name": "parallel_mem",
  "onFailure": "HALT",
  "steps": [
    {"action": "init_turn"},
    {"action": "ensure_role_runtime"},
    {"action": "load_role"},
    {"action": "seed_interaction_mode"},
    {"action": "log_effective_plugin_backends"},
    {"action": "resolve_plugins"},
    {"action": "resolve_main_llm_model"},
    {
      "id": "p",
      "parallel": [
        [{ "action": "memory_retrieve_short_term" }],
        [{ "action": "memory_retrieve_long_term" }]
      ]
    },
    {"action": "run_agent"}
  ]
}"#
    .to_string()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn process_message_parallel_readonly_then_agent_ok() {
    let tmp = isolated_roles_shimeng_clone("par_ok");
    let rid = "par_ok";
    fs::write(
        tmp.path().join(rid).join(PIPELINE_BLUEPRINT_FILENAME),
        blueprint_with_parallel_readonly(),
    )
    .expect("write pipeline");

    let state = KernelAppState::new_in_memory_with_llm(
        Arc::new(MockLlmClient {
            reply: "parallel_ok".to_string(),
        }),
        tmp.path(),
    )
    .await
    .expect("state");

    let req = SendMessageRequest {
        role_id: rid.to_string(),
        user_message: "你好".to_string(),
        scene_id: None,
        session_id: Some("par_ok_sess".to_string()),
    };

    let res = process_message(&state, &req)
        .await
        .expect("process_message");
    assert!(res.reply.contains("parallel_ok"), "reply={:?}", res.reply);
}
