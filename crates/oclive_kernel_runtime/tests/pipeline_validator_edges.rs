//! 蓝图校验与入口降级：`DEGRADE`、非法原子、缺失字段、深层 `BRANCH` 执行等。

use oclive_kernel_runtime::domain::chat_engine::process_message;
use oclive_kernel_runtime::domain::chat_engine::pipeline_loader::PIPELINE_BLUEPRINT_FILENAME;
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

fn blueprint_three_nested_branches_then_agent() -> String {
    r#"{
  "schemaVersion": "1.0",
  "name": "nest3_exec",
  "onFailure": "HALT",
  "steps": [
    { "action": "init_turn" },
    { "action": "ensure_role_runtime" },
    {
      "branch": {
        "predicate": { "type": "sceneIdEquals", "sceneId": "default" },
        "onTrue": [
          {
            "branch": {
              "predicate": { "type": "sceneIdEquals", "sceneId": "default" },
              "onTrue": [
                {
                  "branch": {
                    "predicate": { "type": "sceneIdEquals", "sceneId": "default" },
                    "onTrue": [
                      { "action": "load_role" },
                      { "action": "seed_interaction_mode" },
                      { "action": "log_effective_plugin_backends" },
                      { "action": "resolve_plugins" },
                      { "action": "resolve_main_llm_model" },
                      { "action": "run_agent" }
                    ],
                    "onFalse": [
                      { "action": "load_role" },
                      { "action": "seed_interaction_mode" },
                      { "action": "log_effective_plugin_backends" },
                      { "action": "resolve_plugins" },
                      { "action": "resolve_main_llm_model" },
                      { "action": "run_agent" }
                    ]
                  }
                }
              ],
              "onFalse": [
                {
                  "action": "load_role"
                },
                { "action": "seed_interaction_mode" },
                { "action": "log_effective_plugin_backends" },
                { "action": "resolve_plugins" },
                { "action": "resolve_main_llm_model" },
                { "action": "run_agent" }
              ]
            }
          }
        ],
        "onFalse": [
          { "action": "load_role" },
          { "action": "seed_interaction_mode" },
          { "action": "log_effective_plugin_backends" },
          { "action": "resolve_plugins" },
          { "action": "resolve_main_llm_model" },
          { "action": "run_agent" }
        ]
      }
    }
  ]
}"#
        .to_string()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn three_level_nested_branch_process_message_succeeds() {
    let tmp = isolated_roles_shimeng_clone("nest3_exec");
    let rid = "nest3_exec";
    fs::write(
        tmp.path().join(rid).join(PIPELINE_BLUEPRINT_FILENAME),
        blueprint_three_nested_branches_then_agent(),
    )
    .expect("write pipeline");

    let state = KernelAppState::new_in_memory_with_llm(
        Arc::new(MockLlmClient {
            reply: "nest3_ok".to_string(),
        }),
        tmp.path(),
    )
    .await
    .expect("KernelAppState");

    let req = SendMessageRequest {
        role_id: rid.to_string(),
        user_message: "你好".to_string(),
        scene_id: None,
        session_id: Some("nest3_exec_sess".to_string()),
    };

    let res = process_message(&state, &req).await.expect("process_message");
    assert_eq!(res.api_version, API_VERSION);
    assert!(res.reply.contains("nest3_ok"), "reply={:?}", res.reply);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn degrade_skips_failed_step_and_completes() {
    let tmp = isolated_roles_shimeng_clone("bp_degrade");
    let rid = "bp_degrade";
    let j = r#"{
  "schemaVersion": "1.0",
  "name": "deg",
  "onFailure": "DEGRADE",
  "steps": [
    { "action": "init_turn" },
    { "action": "ensure_role_runtime" },
    { "action": "load_role" },
    { "action": "seed_interaction_mode" },
    { "action": "log_effective_plugin_backends" },
    { "action": "analyze_emotion_user" },
    { "action": "resolve_plugins" },
    { "action": "resolve_main_llm_model" },
    { "action": "run_agent" }
  ]
}"#;
    fs::write(tmp.path().join(rid).join(PIPELINE_BLUEPRINT_FILENAME), j).expect("write");

    let state = KernelAppState::new_in_memory_with_llm(
        Arc::new(MockLlmClient {
            reply: "degrade_ok".to_string(),
        }),
        tmp.path(),
    )
    .await
    .expect("KernelAppState");

    let req = SendMessageRequest {
        role_id: rid.to_string(),
        user_message: "你好".to_string(),
        scene_id: None,
        session_id: Some("bp_degrade_sess".to_string()),
    };

    let res = process_message(&state, &req).await.expect("process_message");
    assert!(res.reply.contains("degrade_ok"), "reply={:?}", res.reply);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unknown_action_blueprint_skipped_and_default_runs() {
    let tmp = isolated_roles_shimeng_clone("bp_unknown");
    let rid = "bp_unknown";
    fs::write(
        tmp.path().join(rid).join(PIPELINE_BLUEPRINT_FILENAME),
        br#"{"schemaVersion":"1.0","name":"u","steps":[{"action":"totally_unknown_atom"}]}"#,
    )
    .expect("write");

    let state = KernelAppState::new_in_memory_with_llm(
        Arc::new(MockLlmClient {
            reply: "unknown_fallback".to_string(),
        }),
        tmp.path(),
    )
    .await
    .expect("KernelAppState");

    let req = SendMessageRequest {
        role_id: rid.to_string(),
        user_message: "你好".to_string(),
        scene_id: None,
        session_id: Some("bp_unknown_sess".to_string()),
    };

    let res = process_message(&state, &req).await.expect("process_message");
    assert!(
        res.reply.contains("unknown_fallback"),
        "reply={:?}",
        res.reply
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn missing_steps_field_fails_parse_and_uses_default() {
    let tmp = isolated_roles_shimeng_clone("bp_no_steps");
    let rid = "bp_no_steps";
    fs::write(
        tmp.path().join(rid).join(PIPELINE_BLUEPRINT_FILENAME),
        br#"{"schemaVersion":"1.0","name":"x"}"#,
    )
    .expect("write");

    let state = KernelAppState::new_in_memory_with_llm(
        Arc::new(MockLlmClient {
            reply: "no_steps_fallback".to_string(),
        }),
        tmp.path(),
    )
    .await
    .expect("KernelAppState");

    let req = SendMessageRequest {
        role_id: rid.to_string(),
        user_message: "你好".to_string(),
        scene_id: None,
        session_id: Some("bp_no_steps_sess".to_string()),
    };

    let res = process_message(&state, &req).await.expect("process_message");
    assert!(
        res.reply.contains("no_steps_fallback"),
        "reply={:?}",
        res.reply
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn memory_heavy_parallel_blueprint_runs() {
    let tmp = isolated_roles_shimeng_clone("bp_mem_par");
    let rid = "bp_mem_par";
    let example = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/blueprints/memory_heavy.ocblueprint");
    let bytes = fs::read(&example).expect("read");
    fs::write(tmp.path().join(rid).join(PIPELINE_BLUEPRINT_FILENAME), bytes).expect("write");

    let state = KernelAppState::new_in_memory_with_llm(
        Arc::new(MockLlmClient {
            reply: "mem_par_ok".to_string(),
        }),
        tmp.path(),
    )
    .await
    .expect("KernelAppState");

    let req = SendMessageRequest {
        role_id: rid.to_string(),
        user_message: "你好".to_string(),
        scene_id: None,
        session_id: Some("bp_mem_par_sess".to_string()),
    };

    let res = process_message(&state, &req).await.expect("process_message");
    assert!(res.reply.contains("mem_par_ok"), "reply={:?}", res.reply);
}
