//! Criterion：入口蓝图 vs 默认线性序列（需 `roles/shimeng` 与临时 `pipeline.ocblueprint`）。
//!
//! 运行：`cargo bench -p oclive_kernel_runtime --bench kernel_pipeline_blueprint`

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oclive_kernel_runtime::domain::chat_engine::process_message;
use oclive_kernel_runtime::domain::role_lifecycle::load_role;
use oclive_kernel_runtime::infrastructure::llm::MockLlmClient;
use oclive_kernel_runtime::models::dto::SendMessageRequest;
use oclive_kernel_runtime::state::KernelAppState;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn workspace_roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roles")
}

fn workspace_shimeng_dir() -> PathBuf {
    workspace_roles_dir().join("shimeng")
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

fn temp_shimeng_clone_with_blueprint(
    role_id: &str,
    blueprint: Option<&[u8]>,
) -> (tempfile::TempDir, PathBuf) {
    let src = workspace_shimeng_dir();
    assert!(
        src.join("manifest.json").is_file(),
        "需要仓库 roles/shimeng"
    );
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().to_path_buf();
    let dest = root.join(role_id);
    copy_dir_recursive(&src, &dest).expect("copy");
    let manifest_path = dest.join("manifest.json");
    let raw = fs::read_to_string(&manifest_path).expect("read manifest");
    let mut v: serde_json::Value = serde_json::from_str(&raw).expect("json");
    v["id"] = serde_json::Value::String(role_id.to_string());
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&v).expect("serialize"),
    )
    .expect("write manifest");
    if let Some(bytes) = blueprint {
        fs::write(dest.join("pipeline.ocblueprint"), bytes).expect("write blueprint");
    }
    (tmp, root)
}

fn mock_llm() -> Arc<dyn oclive_kernel_runtime::infrastructure::llm::LlmClient> {
    Arc::new(MockLlmClient {
        reply: "bench_pipeline".into(),
    })
}

fn bench_process_message_default_no_blueprint(c: &mut Criterion) {
    let (_tmp, roles_root) = temp_shimeng_clone_with_blueprint("bench_pl_def", None);
    let rt = Runtime::new().expect("runtime");
    let state = rt
        .block_on(KernelAppState::new_in_memory_with_llm(
            mock_llm(),
            roles_root,
        ))
        .expect("state");
    rt.block_on(load_role(&state, "bench_pl_def", false))
        .expect("load_role");
    let req = SendMessageRequest {
        role_id: "bench_pl_def".into(),
        user_message: "基准蓝图对照（无文件）".into(),
        scene_id: None,
        session_id: Some("bench_pl_def_sess".into()),
    };
    c.bench_function("process_message_default_no_blueprint", |b| {
        b.to_async(&rt).iter(|| async {
            let out = process_message(black_box(&state), black_box(&req))
                .await
                .expect("process_message");
            black_box(out);
        });
    });
}

fn bench_process_message_simple_blueprint(c: &mut Criterion) {
    let example = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/blueprints/simple_companion.ocblueprint");
    let bp = fs::read(&example).expect("read simple_companion");
    let (_tmp, roles_root) = temp_shimeng_clone_with_blueprint("bench_pl_simple", Some(&bp));
    let rt = Runtime::new().expect("runtime");
    let state = rt
        .block_on(KernelAppState::new_in_memory_with_llm(
            mock_llm(),
            roles_root,
        ))
        .expect("state");
    rt.block_on(load_role(&state, "bench_pl_simple", false))
        .expect("load_role");
    let req = SendMessageRequest {
        role_id: "bench_pl_simple".into(),
        user_message: "基准蓝图 simple_companion".into(),
        scene_id: None,
        session_id: Some("bench_pl_simple_sess".into()),
    };
    c.bench_function("process_message_blueprint_simple_companion", |b| {
        b.to_async(&rt).iter(|| async {
            let out = process_message(black_box(&state), black_box(&req))
                .await
                .expect("process_message");
            black_box(out);
        });
    });
}

fn bench_process_message_memory_heavy_parallel(c: &mut Criterion) {
    let example = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/blueprints/memory_heavy.ocblueprint");
    let bp = fs::read(&example).expect("read memory_heavy");
    let (_tmp, roles_root) = temp_shimeng_clone_with_blueprint("bench_pl_mem", Some(&bp));
    let rt = Runtime::new().expect("runtime");
    let state = rt
        .block_on(KernelAppState::new_in_memory_with_llm(
            mock_llm(),
            roles_root,
        ))
        .expect("state");
    rt.block_on(load_role(&state, "bench_pl_mem", false))
        .expect("load_role");
    let req = SendMessageRequest {
        role_id: "bench_pl_mem".into(),
        user_message: "基准蓝图 memory_heavy".into(),
        scene_id: None,
        session_id: Some("bench_pl_mem_sess".into()),
    };
    c.bench_function("process_message_blueprint_memory_heavy_parallel", |b| {
        b.to_async(&rt).iter(|| async {
            let out = process_message(black_box(&state), black_box(&req))
                .await
                .expect("process_message");
            black_box(out);
        });
    });
}

criterion_group!(
    benches,
    bench_process_message_default_no_blueprint,
    bench_process_message_simple_blueprint,
    bench_process_message_memory_heavy_parallel,
);
criterion_main!(benches);
