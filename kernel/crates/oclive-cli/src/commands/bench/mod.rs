//! `oclive bench`: standard vs Monolith subprocess sampling.

mod bench_core;
mod bench_runner;

pub use bench_core::{
    cargo_build_dual, collect_bench_report, print_bench_comparison, read_package_name,
    release_bin_path, resolve_project_root, stats, BenchReport, SampleStats, StandardMonolithPair,
    BENCH_REPORT_SCHEMA_VERSION,
};
pub(crate) use bench_runner::emit_json_report;

use super::bench_history::{append_history, compare_history, history_path, print_bench_history};
use super::bench_stress::{
    run_bench_compare_versions, run_bench_live, run_bench_matrix, run_bench_regression,
    run_bench_watch,
};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
pub struct BenchArgs {
    /// Project root (contains Cargo.toml, monolith.toml)
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,

    /// Outer repeat count (one subprocess per run, for percentiles)
    #[arg(long, default_value_t = 20)]
    pub runs: u32,

    /// Hot-loop iterations per subprocess (`OCLIVE_KERNEL_BENCH_ITERS`)
    #[arg(long, default_value_t = 400)]
    pub inner_iters: u32,

    /// Run `cargo build --release` first (two builds)
    #[arg(long)]
    pub release: bool,

    /// Write JSON report to stdout only (pipes / schema validation); progress on stderr
    #[arg(long)]
    pub json: bool,

    /// JSON report path; `-` means stdout (prefer this over `--json` when writing to a file)
    #[arg(long, default_value = "-")]
    pub output: String,

    /// Append this report to `bench_history.json` at project root (local; do not commit)
    #[arg(long)]
    pub save: bool,

    /// Compare the two most recent entries in `bench_history.json` (no sampling)
    #[arg(long)]
    pub compare: bool,

    /// Print trend table for all `bench_history.json` entries (no sampling)
    #[arg(long)]
    pub history: bool,

    /// Watch `src/**/*.rs` and `Cargo.toml`; on change run release build + bench (3 runs) and --save
    #[arg(long)]
    pub watch: bool,

    /// Terminal live performance dashboard (sparkline; not the web `oclive dashboard`)
    #[arg(long)]
    pub live: bool,

    /// [deprecated] Use `--live` instead
    #[arg(long, hide = true)]
    pub dashboard: bool,

    /// Monolith tier × preset matrix (3 runs each)
    #[arg(long)]
    pub matrix: bool,

    /// Compare to latest bench_history entry; exit 1 if over threshold
    #[arg(long)]
    pub regression: bool,

    /// Regression threshold (%); defaults per metric if omitted (p50 5 / p95 10 / memory & size 5–10)
    #[arg(long)]
    pub regression_threshold: Option<f64>,

    /// Compare performance against a Git ref (5 runs each)
    #[arg(long = "compare-versions")]
    pub compare_versions: Option<String>,

    /// HTTP /chat concurrency stress test (kernel must be running)
    #[arg(long)]
    pub stress: bool,

    /// Stress test concurrent workers (default 10)
    #[arg(long, default_value_t = 10)]
    pub stress_concurrency: u32,

    /// Stress test duration in seconds (default 30)
    #[arg(long, default_value_t = 30)]
    pub stress_duration: u64,

    /// Compare standard vs Monolith /chat replies (exact match; MOCK_LLM)
    #[arg(long)]
    pub equivalence: bool,

    /// Long-duration stability: periodic /chat + resource snapshots
    #[arg(long)]
    pub soak: bool,

    /// Soak duration in hours; fractional hours are accepted (default 72)
    #[arg(long, default_value_t = 72.0)]
    pub soak_duration: f64,

    /// Use actual wall-clock hours instead of the accelerated local smoke clock
    #[arg(long)]
    pub soak_real_time: bool,

    /// Resource sample interval in seconds for --soak-real-time (default 60)
    #[arg(long, default_value_t = 60)]
    pub soak_sample_interval: u64,

    /// Cold-start latency: spawn kernel --api and measure first /chat reply
    #[arg(long)]
    pub cold_start: bool,

    /// Cold-start repetitions (restart kernel each time; default 1)
    #[arg(long, default_value_t = 1)]
    pub cold_start_runs: u32,

    /// Warm messages after cold-start probe per run (default 5)
    #[arg(long, default_value_t = 5)]
    pub cold_start_warm_messages: u32,

    /// Extra args forwarded to `cargo build` (after `--`)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub cargo_extra: Vec<String>,
}

/// Run the bench subcommand (standard vs monolith sampling and advanced modes).
///
/// # Errors
///
/// Returns an error when the project path, build, or subprocess sampling fails.
pub fn run(args: BenchArgs) -> Result<()> {
    let root = resolve_project_root(&args.path)?;
    if args.equivalence {
        return crate::bench_equivalence::run_equivalence(&root, &args);
    }
    if args.soak {
        return crate::bench_soak::run_soak(&root, &args);
    }
    if args.cold_start {
        return crate::bench_cold_start::run_cold_start(&root, &args);
    }
    if args.stress {
        return crate::bench_stress::run_stress(&root, &args);
    }
    if let Some(ref git_ref) = args.compare_versions {
        return run_bench_compare_versions(&root, git_ref, &args);
    }
    if args.dashboard {
        eprintln!(
            "⚠ [deprecated] `oclive bench --dashboard` — use `--live` (web UI: `oclive dashboard`)"
        );
    }
    if args.live || args.dashboard {
        return run_bench_live(&root, &args);
    }
    if args.matrix {
        return run_bench_matrix(&root, &args);
    }
    if args.watch {
        return run_bench_watch(&root, &args);
    }
    if args.history {
        return print_bench_history(&root, args.json);
    }
    if args.compare {
        return compare_history(&root);
    }
    let report = bench_runner::run_standard_monolith_bench(&root, &args)?;
    bench_runner::emit_bench_report(&args, &report)?;
    if args.regression {
        let code = run_bench_regression(&root, &report, args.regression_threshold, args.json)?;
        if !args.json && args.output != "-" {
            print_bench_comparison(&report);
        }
        if args.save {
            append_history(&root, &report)?;
            eprintln!("Appended to {}", history_path(&root).display());
        }
        if code != 0 {
            std::process::exit(1);
        }
        return Ok(());
    }
    if args.save {
        append_history(&root, &report)?;
        eprintln!("Appended to {}", history_path(&root).display());
    }
    if !args.json && args.output != "-" {
        print_bench_comparison(&report);
    }
    Ok(())
}
