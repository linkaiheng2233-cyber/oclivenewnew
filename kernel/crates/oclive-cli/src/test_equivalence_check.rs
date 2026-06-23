//! `test --equivalence-check` — delegate to `bench --equivalence` when Monolith is enabled.

use anyhow::Result;
use std::path::Path;

use crate::bench_cmd::BenchArgs;

pub fn run(root: &Path) -> Result<()> {
    if !root.join("monolith.toml").is_file() {
        println!("oclive test --equivalence-check: skip (no monolith.toml)");
        return Ok(());
    }
    let args = BenchArgs {
        path: root.to_path_buf(),
        runs: 1,
        inner_iters: 1,
        release: true,
        json: false,
        output: "-".into(),
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
        equivalence: true,
        soak: false,
        soak_duration: 72,
        cold_start: false,
        cold_start_runs: 1,
        cold_start_warm_messages: 5,
        cargo_extra: vec![],
    };
    crate::bench_equivalence::run_equivalence(root, &args)
}
