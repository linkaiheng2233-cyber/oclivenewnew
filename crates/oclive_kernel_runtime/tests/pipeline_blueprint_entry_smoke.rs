//! `pipeline.ocblueprint`：加载、解释与 HALT 失败时回退到默认入口序列。

use oclive_kernel_runtime::domain::chat_engine::pipeline_loader::PIPELINE_BLUEPRINT_FILENAME;
use oclive_kernel_runtime::domain::chat_engine::process_message;
use oclive_kernel_runtime::infrastructure::llm::MockLlmClient;
use oclive_kernel_runtime::models::dto::{SendMessageRequest, API_VERSION};
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn process_message_with_example_blueprint_matches_default() {
    let tmp = isolated_roles_shimeng_clone("bp_ok");
    let rid = "bp_ok";
    let example = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/blueprints/simple_companion.ocblueprint");
    let bp_bytes = fs::read(&example).expect("read example blueprint");
    fs::write(
        tmp.path().join(rid).join(PIPELINE_BLUEPRINT_FILENAME),
        bp_bytes,
    )
    .expect("write pipeline");

    let state = KernelAppState::new_in_memory_with_llm(
        Arc::new(MockLlmClient {
            reply: "blueprint_ok".to_string(),
        }),
        tmp.path(),
    )
    .await
    .expect("KernelAppState");

    let req = SendMessageRequest {
        role_id: rid.to_string(),
        user_message: "你好".to_string(),
        scene_id: None,
        session_id: Some("bp_ok_sess".to_string()),
    };

    let res = process_message(&state, &req)
        .await
        .expect("process_message");
    assert_eq!(res.api_version, API_VERSION);
    assert!(res.reply.contains("blueprint_ok"), "reply={:?}", res.reply);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn invalid_pipeline_file_skips_blueprint_and_still_replies() {
    let tmp = isolated_roles_shimeng_clone("bp_bad_json");
    let rid = "bp_bad_json";
    fs::write(
        tmp.path().join(rid).join(PIPELINE_BLUEPRINT_FILENAME),
        b"not json",
    )
    .expect("write bad pipeline");

    let state = KernelAppState::new_in_memory_with_llm(
        Arc::new(MockLlmClient {
            reply: "fallback_bad_json".to_string(),
        }),
        tmp.path(),
    )
    .await
    .expect("KernelAppState");

    let req = SendMessageRequest {
        role_id: rid.to_string(),
        user_message: "你好".to_string(),
        scene_id: None,
        session_id: Some("bp_bad_json_sess".to_string()),
    };

    let res = process_message(&state, &req)
        .await
        .expect("process_message");
    assert!(
        res.reply.contains("fallback_bad_json"),
        "reply={:?}",
        res.reply
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pipeline_halt_first_step_failure_falls_back_to_default() {
    let tmp = isolated_roles_shimeng_clone("bp_halt_fb");
    let rid = "bp_halt_fb";
    let bad_first = br#"{"schemaVersion":"1.0","name":"halt","onFailure":"HALT","steps":[{"action":"run_agent"}]}"#;
    fs::write(
        tmp.path().join(rid).join(PIPELINE_BLUEPRINT_FILENAME),
        bad_first,
    )
    .expect("write pipeline");

    let state = KernelAppState::new_in_memory_with_llm(
        Arc::new(MockLlmClient {
            reply: "halt_fallback_ok".to_string(),
        }),
        tmp.path(),
    )
    .await
    .expect("KernelAppState");

    let req = SendMessageRequest {
        role_id: rid.to_string(),
        user_message: "你好".to_string(),
        scene_id: None,
        session_id: Some("bp_halt_fb_sess".to_string()),
    };

    let res = process_message(&state, &req)
        .await
        .expect("process_message");
    assert!(
        res.reply.contains("halt_fallback_ok"),
        "reply={:?}",
        res.reply
    );
}
