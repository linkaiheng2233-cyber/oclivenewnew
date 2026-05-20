//! `init` 生成后可选的 Monolith 自动基准测试（`--monolith-bench-preset`）。

use crate::bench_cmd::BenchArgs;
use anyhow::Result;
use std::path::Path;

/// 构建 + bench；失败仅打印警告，不向上传播（不阻塞 init）。
pub fn try_post_init_monolith_bench(project_root: &Path) {
    let root = match project_root.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("⚠ Monolith 自动基准测试跳过（无法解析路径 {}）: {e}", project_root.display());
            return;
        }
    };
    if !root.join("monolith.toml").is_file() {
        eprintln!(
            "⚠ Monolith 自动基准测试跳过：{} 无 monolith.toml",
            root.display()
        );
        return;
    }
    let bench_dir = root.join("bench_results");
    if let Err(e) = run_post_init_bench_inner(&root, &bench_dir) {
        eprintln!("⚠ Monolith 自动基准测试未完成（项目已生成）: {:#}", e);
        eprintln!(
            "  可稍后手动执行: cargo run -p oclive-cli -- bench --release --runs 5 -o {}",
            root.display()
        );
        eprintln!("  填写对比报告: docs/WELD_BENCH_REPORT.md");
    }
}

fn run_post_init_bench_inner(root: &Path, bench_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(bench_dir)?;
    let report_path = bench_dir.join("report.json");
    println!("\n—— 自动 Monolith 基准测试（5 轮 release）——");
    eprintln!("cargo build --release（标准 + Monolith）…");
    let args = BenchArgs {
        path: root.to_path_buf(),
        runs: 5,
        inner_iters: 400,
        release: true,
        json: false,
        output: report_path.to_string_lossy().into_owned(),
        save: false,
        compare: false,
        history: false,
        watch: false,
        dashboard: false,
        matrix: false,
        regression: false,
        regression_threshold: None,
        compare_versions: None,
        cargo_extra: vec![],
    };
    crate::bench_cmd::run(args)?;
    println!("已保存: {}", report_path.display());
    Ok(())
}
