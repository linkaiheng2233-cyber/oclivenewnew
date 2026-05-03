//! P2：目录插件整链烟测 — 扫描发现 → bootstrap DTO → 子进程握手 → JSON-RPC → 清理卸载。

use oclive_kernel_runtime::domain::role_lifecycle::load_role;
use oclive_kernel_runtime::infrastructure::directory_plugins::{
    directory_plugin_bootstrap_dto, DEFAULT_DIRECTORY_PLUGIN_ASSET_BASE_URL,
};
use oclive_kernel_runtime::infrastructure::remote_plugin::{invoke_directory_plugin_rpc, RemoteRpcChannel};
use oclive_kernel_runtime::infrastructure::llm::MockLlmClient;
use oclive_kernel_runtime::state::KernelAppState;
use serde_json::json;
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

fn roles_dir_with_clone(role_dir_name: &str) -> tempfile::TempDir {
    let src = workspace_shimeng_dir();
    assert!(
        src.join("manifest.json").is_file(),
        "need roles/shimeng in repo (got {:?})",
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

fn stub_plugin_exe() -> PathBuf {
    std::env::var("CARGO_BIN_EXE_oclive_test_dir_plugin")
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::var("CARGO_BIN_EXE_oclive_test_dir_plugin.exe").map(PathBuf::from)
        })
        .expect("cargo test should set CARGO_BIN_EXE_oclive_test_dir_plugin")
}

fn write_stub_plugin(roles_root: &Path, plugin_id: &str) {
    let app_data = roles_root.join(".oclive_directory_plugin_data");
    let root = app_data.join("plugins").join(plugin_id);
    fs::create_dir_all(&root).expect("plugin dir");
    let exe = stub_plugin_exe();
    let exe_s = exe.to_string_lossy().replace('\\', "\\\\");
    let manifest = format!(
        r#"{{
  "schema_version": 1,
  "id": "{id}",
  "version": "1.0.0",
  "process": {{
    "command": "{cmd}",
    "args": []
  }}
}}"#,
        id = plugin_id,
        cmd = exe_s
    );
    fs::write(root.join("manifest.json"), manifest).expect("write plugin manifest");
}

fn mock_llm() -> Arc<dyn oclive_kernel_runtime::infrastructure::llm::LlmClient> {
    Arc::new(MockLlmClient {
        reply: "p2_dir_ok".into(),
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn directory_plugin_discover_bootstrap_rpc_and_teardown() {
    let plugin_id = format!(
        "p2ds_{}",
        uuid::Uuid::new_v4()
            .to_string()
            .chars()
            .filter(|c| *c != '-')
            .take(8)
            .collect::<String>()
    );
    let tmp = roles_dir_with_clone("p2_role_dp");
    let roles_root = tmp.path().to_path_buf();
    write_stub_plugin(&roles_root, &plugin_id);

    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles_root.clone())
        .await
        .expect("state");

    load_role(&state, "p2_role_dp", false)
        .await
        .expect("load_role");

    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    assert!(
        state
            .directory_plugins
            .plugin_roots
            .read()
            .contains_key(&plugin_id),
        "plugin should be discovered under app_data/plugins"
    );

    let dto = directory_plugin_bootstrap_dto(
        state.directory_plugins.as_ref(),
        Some("p2_role_dp".into()),
        DEFAULT_DIRECTORY_PLUGIN_ASSET_BASE_URL,
    );
    assert!(
        dto.plugin_ids.contains(&plugin_id),
        "bootstrap plugin_ids should list stub plugin: {:?}",
        dto.plugin_ids
    );

    let url = state
        .directory_plugins
        .ensure_rpc_url(&plugin_id)
        .expect("ensure_rpc_url");
    assert!(url.starts_with("http://"), "rpc url: {}", url);

    let out = invoke_directory_plugin_rpc(
        url.as_str(),
        "ping",
        json!({ "x": 1 }),
        RemoteRpcChannel::Plugin,
    )
    .await
    .expect("invoke_directory_plugin_rpc");
    assert_eq!(out["p2_stub"], true);

    state.directory_plugins.clear_plugin_process(&plugin_id);

    let plugin_fs = roles_root
        .join(".oclive_directory_plugin_data")
        .join("plugins")
        .join(&plugin_id);
    if plugin_fs.exists() {
        fs::remove_dir_all(&plugin_fs).expect("remove plugin tree");
    }
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    assert!(
        !state
            .directory_plugins
            .plugin_roots
            .read()
            .contains_key(&plugin_id),
        "after removal + rescan, plugin id should be gone"
    );
}
