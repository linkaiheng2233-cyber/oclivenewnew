//! Criterion：核心对话与角色加载/切换（需仓库内 `roles/shimeng`）。
//!
//! 运行：`cargo bench -p oclive_kernel_runtime --bench kernel_hot_paths`

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

/// 两个独立角色目录（自 shimeng 克隆），用于 `load_role` 切换基准。
fn temp_roles_two_clones(a: &str, b: &str) -> (tempfile::TempDir, PathBuf) {
    let src = workspace_shimeng_dir();
    assert!(
        src.join("manifest.json").is_file(),
        "需要仓库 roles/shimeng（基准从该目录克隆）"
    );
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().to_path_buf();
    for (name, rid) in [(a, a), (b, b)] {
        let dest = root.join(name);
        copy_dir_recursive(&src, &dest).expect("copy shimeng tree");
        let manifest_path = dest.join("manifest.json");
        let raw = fs::read_to_string(&manifest_path).expect("read manifest");
        let mut v: serde_json::Value = serde_json::from_str(&raw).expect("manifest json");
        v["id"] = serde_json::Value::String(rid.to_string());
        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&v).expect("manifest serialize"),
        )
        .expect("write manifest");
    }
    (tmp, root)
}

fn mock_llm() -> Arc<dyn oclive_kernel_runtime::infrastructure::llm::LlmClient> {
    Arc::new(MockLlmClient {
        reply: "bench_reply".into(),
    })
}

fn bench_process_message_once(c: &mut Criterion) {
    let roles = workspace_roles_dir();
    assert!(
        roles.join("shimeng/manifest.json").is_file(),
        "需要 roles/shimeng"
    );
    let rt = Runtime::new().expect("runtime");
    let state = rt
        .block_on(KernelAppState::new_in_memory_with_llm(mock_llm(), roles))
        .expect("state");
    rt.block_on(load_role(&state, "shimeng", false))
        .expect("load_role");
    let req = SendMessageRequest {
        role_id: "shimeng".into(),
        user_message: "基准单轮".into(),
        scene_id: None,
        session_id: Some("crit_bench_sess".into()),
    };
    c.bench_function("process_message_once_mock_llm", |b| {
        b.to_async(&rt).iter(|| async {
            let out = process_message(black_box(&state), black_box(&req))
                .await
                .expect("process_message");
            black_box(out);
        });
    });
}

fn bench_process_message_10_rounds(c: &mut Criterion) {
    let roles = workspace_roles_dir();
    let rt = Runtime::new().expect("runtime");
    let state = rt
        .block_on(KernelAppState::new_in_memory_with_llm(mock_llm(), roles))
        .expect("state");
    rt.block_on(load_role(&state, "shimeng", false))
        .expect("load_role");
    let req = SendMessageRequest {
        role_id: "shimeng".into(),
        user_message: "基准多轮".into(),
        scene_id: None,
        session_id: Some("crit_bench_10".into()),
    };
    let mut group = c.benchmark_group("process_message_10_rounds");
    group.sample_size(15);
    group.bench_function("sequential_10", |b| {
        b.to_async(&rt).iter(|| async {
            for _ in 0..10 {
                let out = process_message(black_box(&state), black_box(&req))
                    .await
                    .expect("process_message");
                black_box(out);
            }
        });
    });
    group.finish();
}

fn bench_load_role_switch(c: &mut Criterion) {
    let (_tmp, roles_root) = temp_roles_two_clones("bench_lr_a", "bench_lr_b");
    let rt = Runtime::new().expect("runtime");
    let state = rt
        .block_on(KernelAppState::new_in_memory_with_llm(
            mock_llm(),
            roles_root.clone(),
        ))
        .expect("state");
    c.bench_function("load_role_toggle_two_roles", |b| {
        b.to_async(&rt).iter(|| async {
            load_role(black_box(&state), black_box("bench_lr_a"), false)
                .await
                .expect("a");
            load_role(black_box(&state), black_box("bench_lr_b"), false)
                .await
                .expect("b");
        });
    });
}

criterion_group!(
    benches,
    bench_process_message_once,
    bench_process_message_10_rounds,
    bench_load_role_switch,
);
criterion_main!(benches);
