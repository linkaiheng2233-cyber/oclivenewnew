//! Bench sampling helpers, report types, and comparison output.

use super::BenchArgs;
use crate::bench_metrics::{binary_file_size, run_bench_child_with_peak};
use crate::build_cmd::{regenerate_monolith_from_disk_quiet, run_timed_dual_build, BuildArgs};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
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

/// Paired standard vs Monolith metrics (units like bytes / MiB / seconds are distinguished by field semantics).
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

/// Canonicalize a project root from CLI `-o` / cwd-relative path.
///
/// # Errors
///
/// Fails when the path cannot be resolved or canonicalized.
pub fn resolve_project_root(path: &Path) -> Result<PathBuf> {
    let root = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().context("current_dir")?.join(path)
    };
    root.canonicalize()
        .with_context(|| format!("cannot resolve project path: {}", root.display()))
}

/// Read `[package].name` from `Cargo.toml`.
///
/// # Errors
///
/// Fails when the manifest is missing or invalid.
pub fn read_package_name(manifest_dir: &Path) -> Result<String> {
    let p = manifest_dir.join("Cargo.toml");
    let raw = fs::read_to_string(&p).context("read Cargo.toml")?;
    let v: toml::Value = toml::from_str(&raw).context("parse Cargo.toml")?;
    v.get("package")
        .and_then(|x| x.get("name"))
        .and_then(|n| n.as_str())
        .map(String::from)
        .context("Cargo.toml missing [package].name")
}

#[must_use]
pub fn release_bin_path(dir: &Path, name: &str, release: bool) -> PathBuf {
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

#[must_use]
pub fn stats(mut samples: Vec<f64>) -> SampleStats {
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

/// Build standard + monolith binaries via shared `build_cmd`.
///
/// # Errors
///
/// Propagates `cargo` / manifest errors from `build_cmd::run`.
pub fn cargo_build_dual(root: &Path, release: bool, extra: &[String]) -> Result<()> {
    let b = BuildArgs {
        path: root.to_path_buf(),
        no_cargo: false,
        release,
        features: vec![],
        cargo_extra: extra.to_vec(),
    };
    crate::build_cmd::run(b)
}
/// Sample standard vs monolith binaries and aggregate a [`BenchReport`].
///
/// # Errors
///
/// Fails when `monolith.toml` is missing, builds fail, or child bench processes error.
pub fn collect_bench_report(root: &Path, args: &BenchArgs) -> Result<BenchReport> {
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