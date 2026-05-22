//! `oclive bench`: standard vs Monolith subprocess sampling.

use crate::bench_metrics::{binary_file_size, run_bench_child_with_peak};
use crate::build_cmd::{
    regenerate_monolith_from_disk, regenerate_monolith_from_disk_quiet, run_timed_dual_build, BuildArgs,
};
use super::bench_history::{append_history, compare_history, history_path, print_bench_history};
use super::bench_stress::{
    run_bench_compare_versions, run_bench_live, run_bench_matrix, run_bench_regression, run_bench_watch,
};
use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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

    /// Soak duration in hours (default 72)
    #[arg(long, default_value_t = 72)]
    pub soak_duration: u64,

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

pub const BENCH_REPORT_SCHEMA_VERSION: u32 = 2;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BenchReport {
    pub schema_version: u32,
    pub package_name: String,
    pub runs: u32,
    pub inner_iters: u32,
    pub release: bool,
    pub standard_ms: SampleStats,
    pub monolith_ms: SampleStats,
    pub binary_size: StandardMonolithPair<u64>,
    pub peak_memory: StandardMonolithPair<u64>,
    pub build_time: StandardMonolithPair<f64>,
}

/// 标准版 vs Monolith 成对指标（字节 / MiB / 秒等由字段语义区分）。
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StandardMonolithPair<T> {
    pub standard: T,
    pub monolith: T,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SampleStats {
    pub samples: Vec<f64>,
    pub min: f64,
    pub max: f64,
    pub p50: f64,
    pub p95: f64,
    pub mean: f64,
}

pub(crate) fn resolve_project_root(path: &Path) -> Result<PathBuf> {
    let root = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().context("current_dir")?.join(path)
    };
    root.canonicalize()
        .with_context(|| format!("cannot resolve project path: {}", root.display()))
}

pub(crate) fn read_package_name(manifest_dir: &Path) -> Result<String> {
    let p = manifest_dir.join("Cargo.toml");
    let raw = fs::read_to_string(&p).context("read Cargo.toml")?;
    let v: toml::Value = toml::from_str(&raw).context("parse Cargo.toml")?;
    v.get("package")
        .and_then(|x| x.get("name"))
        .and_then(|n| n.as_str())
        .map(String::from)
        .context("Cargo.toml missing [package].name")
}

pub(crate) fn release_bin_path(dir: &Path, name: &str, release: bool) -> PathBuf {
    let profile = if release { "release" } else { "debug" };
    let p = dir.join("target").join(profile).join(name);
    if cfg!(windows) {
        p.with_extension("exe")
    } else {
        p
    }
}

fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

pub(crate) fn stats(mut samples: Vec<f64>) -> SampleStats {
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let min = samples.first().copied().unwrap_or(0.0);
    let max = samples.last().copied().unwrap_or(0.0);
    let p50 = percentile_sorted(&samples, 0.50);
    let p95 = percentile_sorted(&samples, 0.95);
    let mean = if samples.is_empty() {
        0.0
    } else {
        samples.iter().sum::<f64>() / samples.len() as f64
    };
    SampleStats {
        samples,
        min,
        max,
        p50,
        p95,
        mean,
    }
}

pub(crate) fn cargo_build_dual(root: &Path, release: bool, extra: &[String]) -> Result<()> {
    let b = BuildArgs {
        path: root.to_path_buf(),
        no_cargo: false,
        release,
        features: vec![],
        cargo_extra: extra.to_vec(),
    };
    crate::build_cmd::run(b)
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
    let mt = root.join("monolith.toml");
    if !mt.is_file() {
        bail!("Not found: {}", mt.display());
    }
    let file = if args.json {
        regenerate_monolith_from_disk_quiet(&root)?
    } else {
        regenerate_monolith_from_disk(&root)?
    };
    if !file.monolith.enabled {
        bail!("monolith.toml: enabled = false; bench needs Monolith enabled to compare both binaries.");
    }

    let build_time = if args.release {
        eprintln!("cargo build --release (standard + Monolith, timed)…");
        let t = run_timed_dual_build(&root, true, &args.cargo_extra, file.monolith.enabled)
            .context("warm-up build (standard + Monolith)")?;
        StandardMonolithPair {
            standard: t.standard_secs,
            monolith: t.monolith_secs,
        }
    } else {
        cargo_build_dual(&root, false, &args.cargo_extra)
            .context("warm-up build (standard + Monolith)")?;
        StandardMonolithPair {
            standard: 0.0,
            monolith: 0.0,
        }
    };

    let pkg = read_package_name(&root)?;
    let std_bin = release_bin_path(&root, &pkg, args.release);
    let mono_bin = release_bin_path(&root, &format!("{pkg}-monolith"), args.release);

    let binary_size = StandardMonolithPair {
        standard: binary_file_size(&std_bin)?,
        monolith: binary_file_size(&mono_bin)?,
    };

    let mut std_samples = Vec::with_capacity(args.runs as usize);
    let mut mono_samples = Vec::with_capacity(args.runs as usize);
    let mut std_peak_mib = 0u64;
    let mut mono_peak_mib = 0u64;
    for _ in 0..args.runs {
        let (ms, peak) = run_bench_child_with_peak(&std_bin, args.inner_iters)?;
        std_samples.push(ms);
        std_peak_mib = std_peak_mib.max(peak);
        let (ms, peak) = run_bench_child_with_peak(&mono_bin, args.inner_iters)?;
        mono_samples.push(ms);
        mono_peak_mib = mono_peak_mib.max(peak);
    }

    let report = BenchReport {
        schema_version: BENCH_REPORT_SCHEMA_VERSION,
        package_name: pkg,
        runs: args.runs,
        inner_iters: args.inner_iters,
        release: args.release,
        standard_ms: stats(std_samples),
        monolith_ms: stats(mono_samples),
        binary_size,
        peak_memory: StandardMonolithPair {
            standard: std_peak_mib,
            monolith: mono_peak_mib,
        },
        build_time,
    };
    let json = serde_json::to_string_pretty(&report)?;
    if args.output != "-" && !args.json {
        fs::write(&args.output, &json).with_context(|| format!("write {}", args.output))?;
        eprintln!("Wrote {}", args.output);
    } else {
        println!("{json}");
    }
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

pub(crate) fn collect_bench_report(root: &Path, args: &BenchArgs) -> Result<BenchReport> {
    let mt = root.join("monolith.toml");
    if !mt.is_file() {
        anyhow::bail!("monolith.toml required");
    }
    let file = regenerate_monolith_from_disk_quiet(root)?;
    if args.release {
        let _ = run_timed_dual_build(root, true, &args.cargo_extra, file.monolith.enabled)?;
    } else {
        cargo_build_dual(root, false, &args.cargo_extra)?;
    }
    let pkg = read_package_name(root)?;
    let std_bin = release_bin_path(root, &pkg, args.release);
    let mono_bin = release_bin_path(root, &format!("{pkg}-monolith"), args.release);
    let binary_size = StandardMonolithPair {
        standard: binary_file_size(&std_bin)?,
        monolith: binary_file_size(&mono_bin)?,
    };
    let mut std_samples = Vec::new();
    let mut mono_samples = Vec::new();
    let mut std_peak = 0u64;
    let mut mono_peak = 0u64;
    for _ in 0..args.runs {
        let (ms, peak) = run_bench_child_with_peak(&std_bin, args.inner_iters)?;
        std_samples.push(ms);
        std_peak = std_peak.max(peak);
        let (ms, peak) = run_bench_child_with_peak(&mono_bin, args.inner_iters)?;
        mono_samples.push(ms);
        mono_peak = mono_peak.max(peak);
    }
    Ok(BenchReport {
        schema_version: BENCH_REPORT_SCHEMA_VERSION,
        package_name: pkg,
        runs: args.runs,
        inner_iters: args.inner_iters,
        release: args.release,
        standard_ms: stats(std_samples),
        monolith_ms: stats(mono_samples),
        binary_size,
        peak_memory: StandardMonolithPair {
            standard: std_peak,
            monolith: mono_peak,
        },
        build_time: StandardMonolithPair {
            standard: 0.0,
            monolith: 0.0,
        },
    })
}
pub fn print_bench_comparison(report: &BenchReport) {
    let std_p50 = report.standard_ms.p50;
    let mono_p50 = report.monolith_ms.p50;
    let std_p95 = report.standard_ms.p95;
    let mono_p95 = report.monolith_ms.p95;
    let improve_p50 = pct_improvement(std_p50, mono_p50);
    let improve_p95 = pct_improvement(std_p95, mono_p95);
    println!(
        "\n—— Standard vs Monolith welded ({}, release={}) ——",
        report.runs, report.release
    );
    println!("  metric      standard(ms)  monolith(ms)  change");
    println!(
        "  p50         {:>10.3}   {:>10.3}   {:>+6.1}%",
        std_p50, mono_p50, improve_p50
    );
    println!(
        "  P95         {:>10.3}   {:>10.3}   {:>+6.1}%",
        std_p95, mono_p95, improve_p95
    );
    if mono_p50 < std_p50 {
        println!("  → Monolith median lower (~{:.1}%)", improve_p50);
    } else if mono_p50 > std_p50 {
        println!("  → Monolith median higher; reduce weld_modules in monolith.toml");
    } else {
        println!("  → Medians close; increase runs or check build profile");
    }
    print_pair_u64(
        "binary (bytes)",
        report.binary_size.standard,
        report.binary_size.monolith,
    );
    print_pair_u64(
        "peak memory (MiB)",
        report.peak_memory.standard,
        report.peak_memory.monolith,
    );
    if report.release {
        print_pair_f64(
            "compile time (s)",
            report.build_time.standard,
            report.build_time.monolith,
        );
    }
}

fn print_pair_u64(label: &str, standard: u64, monolith: u64) {
    let pct = pct_improvement_u64(standard, monolith);
    println!(
        "  {label:<16} {:>12}   {:>12}   {:>+6.1}%",
        standard, monolith, pct
    );
}

fn print_pair_f64(label: &str, standard: f64, monolith: f64) {
    let pct = pct_improvement(standard, monolith);
    println!(
        "  {label:<16} {:>12.2}   {:>12.2}   {:>+6.1}%",
        standard, monolith, pct
    );
}

fn pct_improvement_u64(standard: u64, monolith: u64) -> f64 {
    if standard == 0 {
        0.0
    } else {
        ((standard as f64 - monolith as f64) / standard as f64) * 100.0
    }
}

fn pct_improvement(standard: f64, monolith: f64) -> f64 {
    if standard.abs() <= f64::EPSILON {
        0.0
    } else {
        ((standard - monolith) / standard) * 100.0
    }
}
