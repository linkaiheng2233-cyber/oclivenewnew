//! Optional post-`init` Monolith auto-benchmark (`--monolith-bench-preset`).

use crate::bench_cmd::BenchArgs;
use anyhow::Result;
use std::path::Path;

/// Build + bench; failures only print warnings (do not block init).
pub fn try_post_init_monolith_bench(project_root: &Path) {
    let root = match project_root.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "⚠ Monolith auto-benchmark skipped (cannot resolve path {}): {e}",
                project_root.display()
            );
            return;
        }
    };
    if !root.join("monolith.toml").is_file() {
        eprintln!(
            "⚠ Monolith auto-benchmark skipped: no monolith.toml at {}",
            root.display()
        );
        return;
    }
    let bench_dir = root.join("bench_results");
    if let Err(e) = run_post_init_bench_inner(&root, &bench_dir) {
        eprintln!(
            "⚠ Monolith auto-benchmark incomplete (project was generated): {:#}",
            e
        );
        eprintln!(
            "  Run manually later: cargo run -p oclive-cli -- --experimental bench --release --runs 5 -o {}",
            root.display()
        );
        eprintln!("  Fill comparison report: docs/WELD_BENCH_REPORT.md");
    }
}

fn run_post_init_bench_inner(root: &Path, bench_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(bench_dir)?;
    let report_path = bench_dir.join("report.json");
    println!("\n—— Auto Monolith benchmark (5 release runs) ——");
    eprintln!("cargo build --release (standard + Monolith)…");
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
        live: false,
        dashboard: false,
        matrix: false,
        regression: false,
        regression_threshold: None,
        compare_versions: None,
        stress: false,
        stress_concurrency: 10,
        stress_duration: 30,
        equivalence: false,
        soak: false,
        soak_duration: 72.0,
        soak_real_time: false,
        soak_sample_interval: 60,
        cold_start: false,
        cold_start_runs: 1,
        cold_start_warm_messages: 5,
        cargo_extra: vec![],
    };
    crate::bench_cmd::run(args)?;
    println!("Saved: {}", report_path.display());
    Ok(())
}
