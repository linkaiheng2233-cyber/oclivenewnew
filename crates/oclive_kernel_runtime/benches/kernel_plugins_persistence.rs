//! Criterion：目录插件 bootstrap / MCP 权限拒绝路径、记忆 I/O、角色包导入导出。
//!
//! 依赖 `full` 特性（默认开启）。目录 RPC 基准需要 `CARGO_BIN_EXE_oclive_test_dir_plugin`（由 `cargo bench` 注入）。
//!
//! 运行：`cargo bench -p oclive_kernel_runtime --bench kernel_plugins_persistence`

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oclive_kernel_runtime::domain::chat_engine::conversation_state_role_id;
use oclive_kernel_runtime::domain::role_lifecycle::load_role;
use oclive_kernel_runtime::infrastructure::directory_plugins::{
    directory_plugin_bootstrap_dto, DEFAULT_DIRECTORY_PLUGIN_ASSET_BASE_URL,
};
use oclive_kernel_runtime::infrastructure::llm::MockLlmClient;
use oclive_kernel_runtime::infrastructure::remote_plugin::{invoke_directory_plugin_rpc, RemoteRpcChannel};
use oclive_kernel_runtime::infrastructure::role_pack_archive::{export_role_pack, import_role_pack};
use oclive_kernel_runtime::infrastructure::RoleStorage;
use oclive_kernel_runtime::state::KernelAppState;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::runtime::Runtime;

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
        "需要 roles/shimeng"
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
    std::env::var_os("CARGO_BIN_EXE_oclive_test_dir_plugin")
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_oclive_test_dir_plugin.exe"))
        .map(PathBuf::from)
        .expect("请使用 `cargo bench` 运行以注入测试目录插件二进制路径")
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
        reply: "bench_plugin".into(),
    })
}

fn bench_directory_plugin_bootstrap_dto(c: &mut Criterion) {
    let plugin_id = format!(
        "bench_dp_{}",
        uuid::Uuid::new_v4()
            .to_string()
            .chars()
            .filter(|c| *c != '-')
            .take(8)
            .collect::<String>()
    );
    let tmp = roles_dir_with_clone("bench_role_dp");
    let roles_root = tmp.path().to_path_buf();
    write_stub_plugin(&roles_root, &plugin_id);

    let rt = Runtime::new().expect("runtime");
    let state = rt
        .block_on(KernelAppState::new_in_memory_with_llm(mock_llm(), roles_root.clone()))
        .expect("state");
    rt.block_on(load_role(&state, "bench_role_dp", false))
        .expect("load_role");
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());

    c.bench_function("directory_rescan_and_bootstrap_dto", |b| {
        b.iter(|| {
            state
                .directory_plugins
                .rescan_plugin_roots(state.storage.roles_dir());
            let dto = directory_plugin_bootstrap_dto(
                state.directory_plugins.as_ref(),
                Some("bench_role_dp".into()),
                DEFAULT_DIRECTORY_PLUGIN_ASSET_BASE_URL,
            );
            black_box(dto);
        });
    });
}

fn bench_directory_plugin_rpc_ping(c: &mut Criterion) {
    let plugin_id = format!(
        "bench_rpc_{}",
        uuid::Uuid::new_v4()
            .to_string()
            .chars()
            .filter(|c| *c != '-')
            .take(8)
            .collect::<String>()
    );
    let tmp = roles_dir_with_clone("bench_role_rpc");
    let roles_root = tmp.path().to_path_buf();
    write_stub_plugin(&roles_root, &plugin_id);

    let rt = Runtime::new().expect("runtime");
    let state = rt
        .block_on(KernelAppState::new_in_memory_with_llm(mock_llm(), roles_root))
        .expect("state");
    rt.block_on(load_role(&state, "bench_role_rpc", false))
        .expect("load_role");
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    let url = state
        .directory_plugins
        .ensure_rpc_url(&plugin_id)
        .expect("ensure_rpc_url");

    c.bench_function("directory_plugin_rpc_ping", |b| {
        b.to_async(&rt).iter(|| async {
            let out = invoke_directory_plugin_rpc(
                url.as_str(),
                "ping",
                json!({ "x": 1 }),
                RemoteRpcChannel::Plugin,
            )
            .await
            .expect("rpc");
            black_box(out);
        });
    });
}

fn bench_mcp_tool_call_denied_fast_path(c: &mut Criterion) {
    let tmp = roles_dir_with_clone("bench_mcp_role");
    let roles_root = tmp.path().to_path_buf();
    let rt = Runtime::new().expect("runtime");
    let state = rt
        .block_on(KernelAppState::new_in_memory_with_llm(mock_llm(), roles_root))
        .expect("state");
    let app_data = state.storage.roles_dir().join(".oclive_directory_plugin_data");
    let mcp_root = app_data.join("mcp-servers");
    fs::create_dir_all(&mcp_root).expect("mcp dir");
    let mf = mcp_root.join("bench_stdio.json");
    fs::write(
        &mf,
        r#"{"id":"bench_stdio_srv","name":"t","transport":"stdio","command":"echo","args":[],"tools":[{"name":"ping"}]}"#,
    )
    .expect("write mcp manifest");

    c.bench_function("mcp_call_tool_denied_stdio_no_grant", |b| {
        b.to_async(&rt).iter(|| async {
            let r = state
                .plugins
                .call_mcp_tool("bench_stdio_srv", "ping", json!({}))
                .await;
            let _ = black_box(r);
        });
    });
}

fn bench_memory_save_and_load(c: &mut Criterion) {
    let roles = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roles");
    assert!(roles.join("shimeng/manifest.json").is_file(), "需要 roles/shimeng");
    let rt = Runtime::new().expect("runtime");
    let state = rt
        .block_on(KernelAppState::new_in_memory_with_llm(mock_llm(), roles))
        .expect("state");
    let rid = "shimeng";
    let srid = conversation_state_role_id(rid, None);
    rt.block_on(load_role(&state, rid, false))
        .expect("load_role");
    c.bench_function("memory_save_and_load_32", |b| {
        b.to_async(&rt).iter(|| async {
            let _id = state
                .memory_repo
                .save_memory(srid.as_str(), "bench_memory_line", 0.5)
                .await
                .expect("save");
            let rows = state
                .memory_repo
                .load_memories(srid.as_str(), 32)
                .await
                .expect("load");
            black_box(rows);
        });
    });
}

fn bench_role_pack_export_import(c: &mut Criterion) {
    let roles_src = tempfile::tempdir().expect("tempdir");
    let roles_dst = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(roles_src.path().join("mumu").join("scenes").join("default")).unwrap();
    fs::write(
        roles_src.path().join("mumu").join("manifest.json"),
        r#"{"id":"mumu","name":"M","version":"1","author":"t","description":"d","default_personality":[0.5,0.5,0.5,0.5,0.5,0.5,0.5],"evolution":{},"user_relations":{"friend":{"prompt_hint":"x"}},"default_relation":"friend","memory_config":{"scene_weight_multiplier":1.0,"topic_weights":{}}}"#,
    )
    .unwrap();

    let st = RoleStorage::new(roles_src.path());
    let out_tmp = tempfile::tempdir().unwrap();
    let pak = out_tmp.path().join("bench.ocpak");
    let st2 = RoleStorage::new(roles_dst.path());

    let rt = Runtime::new().expect("runtime");
    let mut group = c.benchmark_group("role_pack_zip");
    group.sample_size(20);
    group.bench_function("export_import_roundtrip", |b| {
        b.to_async(&rt).iter(|| async {
            export_role_pack(&st, "mumu", &pak).await.expect("export");
            let id = import_role_pack(&st2, &pak, true, |_| {}).await.expect("import");
            black_box(id);
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_directory_plugin_bootstrap_dto,
    bench_directory_plugin_rpc_ping,
    bench_mcp_tool_call_denied_fast_path,
    bench_memory_save_and_load,
    bench_role_pack_export_import,
);
criterion_main!(benches);
