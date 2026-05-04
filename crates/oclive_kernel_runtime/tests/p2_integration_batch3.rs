//! P2 第三批次：目录侧权限相关（MCP）、专家图编译边界、多场景切换。

use oclive_kernel_runtime::domain::chat_engine::process_message;
use oclive_kernel_runtime::domain::expert_models::compile_graph_to_llama_local_config;
use oclive_kernel_runtime::domain::role_lifecycle::load_role;
use oclive_kernel_runtime::domain::scene_commands::switch_scene;
use oclive_kernel_runtime::infrastructure::llm::MockLlmClient;
use oclive_kernel_runtime::models::dto::{SendMessageRequest, SwitchSceneRequest};
use oclive_kernel_runtime::models::expert_models::{ExpertGraph, ExpertNode};
use oclive_kernel_runtime::state::KernelAppState;
use serde_json::json;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

fn workspace_roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roles")
}

fn workspace_shimeng_dir() -> PathBuf {
    workspace_roles_dir().join("shimeng")
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            if let Some(p) = to.parent() {
                std::fs::create_dir_all(p)?;
            }
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// 独立 `roles` 根目录，避免往仓库 `roles/.oclive_directory_plugin_data/` 写入 MCP 清单干扰其它测试。
fn isolated_roles_with_shimeng_clone(role_dir_name: &str) -> tempfile::TempDir {
    let src = workspace_shimeng_dir();
    assert!(
        src.join("manifest.json").is_file(),
        "need roles/shimeng (got {:?})",
        src
    );
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

fn mock_llm() -> Arc<dyn oclive_kernel_runtime::infrastructure::llm::LlmClient> {
    Arc::new(MockLlmClient {
        reply: "batch3_ok".into(),
    })
}

/// `stdio` MCP：未授予 `process:spawn` 时 `call_mcp_tool` 应直接拒绝，不尝试起子进程。
#[cfg(feature = "kernel-agent")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mcp_stdio_tool_call_denied_without_process_spawn_grant() {
    let tmp = isolated_roles_with_shimeng_clone("p2b3_mcp_role");
    let roles_root = tmp.path().to_path_buf();
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles_root)
        .await
        .expect("state");

    // app_data = roles_dir.join(".oclive_directory_plugin_data") — 与 KernelAppState::new_in_memory 一致
    let app_data = state
        .storage
        .roles_dir()
        .join(".oclive_directory_plugin_data");
    let mcp_root = app_data.join("mcp-servers");
    fs::create_dir_all(&mcp_root).expect("mcp dir");
    let mf = mcp_root.join("p2_stdio_test.json");
    fs::write(
        &mf,
        r#"{"id":"p2_stdio_srv","name":"t","transport":"stdio","command":"echo","args":[],"tools":[{"name":"ping"}]}"#,
    )
    .expect("write mcp manifest");

    let err = state
        .plugins
        .call_mcp_tool("p2_stdio_srv", "ping", json!({}))
        .await
        .expect_err("should deny without grant");
    assert!(
        err.contains("process:spawn") || err.contains("permission"),
        "unexpected err: {}",
        err
    );
}

#[test]
fn compile_graph_rejects_missing_base_model_file() {
    let tmp = tempfile::tempdir().unwrap();
    let gguf = tmp.path().join("models").join("gguf");
    fs::create_dir_all(&gguf).unwrap();
    let loras = tmp.path().join("models").join("loras");
    fs::create_dir_all(&loras).unwrap();

    let graph = ExpertGraph {
        version: 1,
        nodes: vec![ExpertNode::BaseModel {
            id: "b1".into(),
            gguf_path: gguf.join("nope.gguf").to_string_lossy().into_owned(),
            ui: None,
        }],
        edges: vec![],
    };
    let e = compile_graph_to_llama_local_config(&graph, gguf.as_path(), loras.as_path())
        .expect_err("missing file");
    let msg = e.to_string();
    assert!(
        msg.contains("not found") || msg.contains("base model"),
        "{}",
        msg
    );
}

#[test]
fn compile_graph_rejects_missing_lora_file() {
    let tmp = tempfile::tempdir().unwrap();
    let gguf = tmp.path().join("models").join("gguf");
    fs::create_dir_all(&gguf).unwrap();
    let base = gguf.join("base.gguf");
    fs::write(&base, b"x").unwrap();
    let loras = tmp.path().join("models").join("loras");
    fs::create_dir_all(&loras).unwrap();

    let graph = ExpertGraph {
        version: 1,
        nodes: vec![
            ExpertNode::BaseModel {
                id: "b1".into(),
                gguf_path: base.to_string_lossy().into_owned(),
                ui: None,
            },
            ExpertNode::LoraAdapter {
                id: "l1".into(),
                gguf_path: loras.join("missing.gguf").to_string_lossy().into_owned(),
                strength: 0.5,
                enabled: true,
                order: 0,
                ui: None,
            },
        ],
        edges: vec![],
    };
    let e = compile_graph_to_llama_local_config(&graph, gguf.as_path(), loras.as_path())
        .expect_err("missing lora");
    assert!(e.to_string().contains("LoRA"), "{}", e);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn switch_scene_then_message_reports_new_scene_id() {
    let roles = workspace_roles_dir();
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles)
        .await
        .expect("state");

    load_role(&state, "shimeng", false)
        .await
        .expect("load_role");

    switch_scene(
        &state,
        &SwitchSceneRequest {
            role_id: "shimeng".into(),
            scene_id: "school".into(),
            together: true,
        },
    )
    .await
    .expect("switch_scene");

    let res = process_message(
        &state,
        &SendMessageRequest {
            role_id: "shimeng".into(),
            user_message: "课间聊聊".into(),
            scene_id: Some("school".into()),
            session_id: None,
        },
    )
    .await
    .expect("process_message");

    assert_eq!(res.scene_id, "school");
}
