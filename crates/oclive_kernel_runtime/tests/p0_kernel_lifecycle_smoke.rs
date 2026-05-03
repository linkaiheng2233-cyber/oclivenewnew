//! P0.T：`delete_role`、`expert_models_*` 会话链、（`role-pack-zip`）插件归档安装；依赖仓库 `roles/shimeng` 作为可删模板。

use oclive_kernel_runtime::domain::expert_models_admin::{
    expert_models_get_effective, expert_models_set_session_override,
};
use oclive_kernel_runtime::domain::role_lifecycle::{delete_role, load_role};
use oclive_kernel_runtime::error::AppError;
use oclive_kernel_runtime::infrastructure::llm::MockLlmClient;
use oclive_kernel_runtime::models::dto::{
    ExpertModelsGetEffectiveRequest, ExpertModelsSetSessionOverrideRequest,
};
use oclive_kernel_runtime::models::expert_models::{ExpertConfigSource, ExpertGraph};
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

/// 返回 **roles 根目录**（其下仅有 `role_dir_name/` 一份从 shimeng 拷贝并改过 manifest.id 的角色）。
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
        reply: "ok".to_string(),
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_role_removes_role_dir_and_prevents_reload() {
    let tmp = roles_dir_with_patched_shimeng_clone("p0_del_role");
    let roles_root = tmp.path().to_path_buf();
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles_root.clone())
        .await
        .expect("state");

    load_role(&state, "p0_del_role", false)
        .await
        .expect("load disposable role");
    assert!(roles_root.join("p0_del_role").is_dir());

    let v = delete_role(&state, "p0_del_role".to_string())
        .await
        .expect("delete_role");
    assert_eq!(v["ok"], true);
    assert_eq!(v["role_id"], "p0_del_role");

    assert!(
        !roles_root.join("p0_del_role").exists(),
        "role directory should be removed"
    );
    let err = state
        .storage
        .load_role("p0_del_role")
        .expect_err("role should be gone from disk");
    assert!(matches!(err, AppError::RoleNotFound(_)));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn expert_models_session_override_roundtrip() {
    let tmp = roles_dir_with_patched_shimeng_clone("p0_exp_role");
    let roles_root = tmp.path().to_path_buf();
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles_root)
        .await
        .expect("state");

    load_role(&state, "p0_exp_role", false)
        .await
        .expect("load role");

    let graph = ExpertGraph {
        version: 42,
        ..Default::default()
    };

    expert_models_set_session_override(
        &state,
        &ExpertModelsSetSessionOverrideRequest {
            role_id: "p0_exp_role".into(),
            session_id: Some("sid_a".into()),
            graph: graph.clone(),
            prompt_style: None,
        },
    )
    .await
    .expect("set session override");

    let eff = expert_models_get_effective(
        &state,
        &ExpertModelsGetEffectiveRequest {
            role_id: "p0_exp_role".into(),
            session_id: Some("sid_a".into()),
        },
    )
    .await
    .expect("get effective");

    assert_eq!(eff.graph.version, 42);
    assert_eq!(eff.graph_source, ExpertConfigSource::SessionOverride);
}

#[cfg(feature = "role-pack-zip")]
#[test]
fn install_plugin_from_packed_stub_directory() {
    use oclive_kernel_runtime::infrastructure::directory_plugins::OclivePluginManifest;
    use oclive_kernel_runtime::infrastructure::plugin_archive::pack_plugin_directory_to_zip_deflated;
    use oclive_kernel_runtime::infrastructure::plugin_install::install_plugin_from_archive_bytes_at;

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("plugin_src");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("manifest.json"),
        r#"{"schema_version":1,"id":"p0_pack_plug","version":"1.0.0","process":{"command":"echo"}}"#,
    )
    .unwrap();
    let zip_path = tmp.path().join("stub.oclive-plugin");
    let digest = pack_plugin_directory_to_zip_deflated(&root, &zip_path).unwrap();
    assert_eq!(digest.len(), 64);

    let bytes = fs::read(&zip_path).unwrap();
    let plugins_root = tmp.path().join("plugins_out");
    let app_data = tmp.path().join("appdata");
    let pid = install_plugin_from_archive_bytes_at(&plugins_root, &app_data, &bytes).unwrap();
    assert_eq!(pid, "p0_pack_plug");
    assert!(
        OclivePluginManifest::load_from_dir(&plugins_root.join("p0_pack_plug")).is_ok(),
        "installed tree should parse as manifest"
    );
}
