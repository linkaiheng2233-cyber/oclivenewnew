//! P0-2：session / 场景切换、专家图编译、多会话 expert 隔离（关键路径）。

use oclive_kernel_runtime::domain::expert_models::compile_graph_to_llama_local_config;
use oclive_kernel_runtime::domain::expert_models_admin::{
    expert_models_get_effective, expert_models_set_role_default, expert_models_set_session_override,
};
use oclive_kernel_runtime::domain::role_lifecycle::load_role;
use oclive_kernel_runtime::domain::scene_commands::switch_scene;
use oclive_kernel_runtime::infrastructure::llm::MockLlmClient;
use oclive_kernel_runtime::models::dto::{
    ExpertModelsGetEffectiveRequest, ExpertModelsSetRoleDefaultRequest,
    ExpertModelsSetSessionOverrideRequest, SwitchSceneRequest,
};
use oclive_kernel_runtime::models::expert_models::{ExpertConfigSource, ExpertGraph, ExpertNode};
use oclive_kernel_runtime::state::KernelAppState;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn workspace_shimeng_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roles/shimeng")
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

fn roles_dir_with_patched_shimeng_clone(role_dir_name: &str) -> tempfile::TempDir {
    let src = workspace_shimeng_dir();
    assert!(
        src.join("manifest.json").is_file(),
        "expected roles/shimeng in repo (got {:?})",
        src
    );
    let tmp = tempfile::tempdir().expect("tempdir");
    let dest = tmp.path().join(role_dir_name);
    copy_dir_recursive(&src, &dest).expect("copy shimeng tree");
    let manifest_path = dest.join("manifest.json");
    let raw = fs::read_to_string(&manifest_path).expect("read manifest");
    let mut v: serde_json::Value = serde_json::from_str(&raw).expect("manifest json");
    v["id"] = serde_json::Value::String(role_dir_name.to_string());
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&v).expect("manifest serialize"),
    )
    .expect("write manifest");
    tmp
}

fn mock_llm() -> Arc<dyn oclive_kernel_runtime::infrastructure::llm::LlmClient> {
    Arc::new(MockLlmClient {
        reply: "p0_ok".to_string(),
    })
}

#[test]
fn expert_compile_graph_minimal_base_only() {
    let tmp = tempfile::tempdir().expect("tmp");
    let gguf = tmp.path().join("models").join("gguf");
    let loras = tmp.path().join("models").join("loras");
    fs::create_dir_all(&gguf).expect("mkdir gguf");
    fs::create_dir_all(&loras).expect("mkdir loras");
    let model_file = gguf.join("stub.gguf");
    fs::write(&model_file, b"x").expect("touch gguf");

    let graph = ExpertGraph {
        version: 1,
        nodes: vec![ExpertNode::BaseModel {
            id: "base".into(),
            gguf_path: model_file.to_string_lossy().to_string(),
            ui: None,
        }],
        edges: vec![],
    };

    let cfg = compile_graph_to_llama_local_config(&graph, &gguf, &loras).expect("compile");
    assert!(cfg.model_path.is_some());
    assert!(cfg.llama_args.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn expert_graph_session_b_falls_through_to_role_default_when_only_a_overridden() {
    let tmp = roles_dir_with_patched_shimeng_clone("p0_exp_iso");
    let roles_root = tmp.path().to_path_buf();
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles_root)
        .await
        .expect("state");

    load_role(&state, "p0_exp_iso", false)
        .await
        .expect("load role");

    let role_graph = ExpertGraph {
        version: 11,
        ..Default::default()
    };
    expert_models_set_role_default(
        &state,
        &ExpertModelsSetRoleDefaultRequest {
            role_id: "p0_exp_iso".into(),
            graph: role_graph.clone(),
            prompt_style: None,
        },
    )
    .await
    .expect("set role default");

    let sess_graph = ExpertGraph {
        version: 77,
        ..Default::default()
    };
    expert_models_set_session_override(
        &state,
        &ExpertModelsSetSessionOverrideRequest {
            role_id: "p0_exp_iso".into(),
            session_id: Some("sess_a".into()),
            graph: sess_graph,
            prompt_style: None,
        },
    )
    .await
    .expect("set session a override");

    let eff_a = expert_models_get_effective(
        &state,
        &ExpertModelsGetEffectiveRequest {
            role_id: "p0_exp_iso".into(),
            session_id: Some("sess_a".into()),
        },
    )
    .await
    .expect("get effective a");
    assert_eq!(eff_a.graph.version, 77);
    assert_eq!(eff_a.graph_source, ExpertConfigSource::SessionOverride);

    let eff_b = expert_models_get_effective(
        &state,
        &ExpertModelsGetEffectiveRequest {
            role_id: "p0_exp_iso".into(),
            session_id: Some("sess_b".into()),
        },
    )
    .await
    .expect("get effective b");
    assert_eq!(eff_b.graph.version, 11);
    assert_eq!(eff_b.graph_source, ExpertConfigSource::RoleDefault);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn switch_scene_together_updates_character_scene_presence_only_does_not() {
    let tmp = roles_dir_with_patched_shimeng_clone("p0_scene_sw");
    let roles_root = tmp.path().to_path_buf();
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles_root)
        .await
        .expect("state");

    load_role(&state, "p0_scene_sw", false)
        .await
        .expect("load");

    switch_scene(
        &state,
        &SwitchSceneRequest {
            role_id: "p0_scene_sw".into(),
            scene_id: "school".into(),
            together: true,
        },
    )
    .await
    .expect("switch together to school");

    let cur = state
        .db_manager
        .get_current_scene("p0_scene_sw")
        .await
        .expect("get current");
    let pres = state
        .db_manager
        .get_user_presence_scene("p0_scene_sw")
        .await
        .expect("get presence");
    assert_eq!(cur.as_deref(), Some("school"));
    assert_eq!(pres.as_deref(), Some("school"));

    switch_scene(
        &state,
        &SwitchSceneRequest {
            role_id: "p0_scene_sw".into(),
            scene_id: "default".into(),
            together: false,
        },
    )
    .await
    .expect("switch narrative only to default");

    let cur2 = state
        .db_manager
        .get_current_scene("p0_scene_sw")
        .await
        .expect("get current 2");
    let pres2 = state
        .db_manager
        .get_user_presence_scene("p0_scene_sw")
        .await
        .expect("get presence 2");
    assert_eq!(
        cur2.as_deref(),
        Some("school"),
        "character should stay at school when together=false"
    );
    assert_eq!(pres2.as_deref(), Some("default"));
}
