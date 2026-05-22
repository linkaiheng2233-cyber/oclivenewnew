//! Standard vs Monolith subprocess sampling (core bench loop).

use super::bench_core::{
    cargo_build_dual, read_package_name, release_bin_path, stats, BenchReport,
    BENCH_REPORT_SCHEMA_VERSION, StandardMonolithPair,
};
use super::BenchArgs;
use crate::bench_metrics::{binary_file_size, run_bench_child_with_peak};
use crate::build_cmd::{regenerate_monolith_from_disk, regenerate_monolith_from_disk_quiet, run_timed_dual_build};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

/// Run standard vs Monolith sampling and write/print the JSON report.
///
/// # Errors
///
/// Returns an error when `monolith.toml` is missing/disabled, build fails, or sampling fails.
pub fn run_standard_monolith_bench(root: &Path, args: &BenchArgs) -> Result<BenchReport> {
    let mt = root.join("monolith.toml");
    if !mt.is_file() {
        bail!("Not found: {}", mt.display());
    }
    let file = if args.json {
        regenerate_monolith_from_disk_quiet(root)?
    } else {
        regenerate_monolith_from_disk(root)?
    };
    if !file.monolith.enabled {
        bail!("monolith.toml: enabled = false; bench needs Monolith enabled to compare both binaries.");
    }

    let build_time = if args.release {
        eprintln!("cargo build --release (standard + Monolith, timed)…");
        let t = run_timed_dual_build(root, true, &args.cargo_extra, file.monolith.enabled)
            .context("warm-up build (standard + Monolith)")?;
        StandardMonolithPair {
            standard: t.standard_secs,
            monolith: t.monolith_secs,
        }
    } else {
        cargo_build_dual(root, false, &args.cargo_extra)
            .context("warm-up build (standard + Monolith)")?;
        StandardMonolithPair {
            standard: 0.0,
            monolith: 0.0,
        }
    };

    let pkg = read_package_name(root)?;
    let std_bin = release_bin_path(root, &pkg, args.release);
    let mono_bin = release_bin_path(root, &format!("{pkg}-monolith"), args.release);

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
            standard: std_peak_mib,
            monolith: mono_peak_mib,
        },
        build_time,
    })
}

/// Write bench JSON to stdout or `--output` path.
///
/// # Errors
///
/// Returns an error when serialization or file write fails.
pub fn emit_bench_report(args: &BenchArgs, report: &BenchReport) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    if args.output != "-" && !args.json {
        fs::write(&args.output, &json).with_context(|| format!("write {}", args.output))?;
        eprintln!("Wrote {}", args.output);
    } else {
        println!("{json}");
    }
    Ok(())
}
