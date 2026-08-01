//! Standard vs Monolith subprocess sampling (core bench loop).

use super::bench_core::{
    cargo_build_dual, read_package_name, release_bin_path, stats, BenchReport,
    StandardMonolithPair, BENCH_REPORT_SCHEMA_VERSION,
};
use super::BenchArgs;
use crate::bench_metrics::{binary_file_size, run_bench_child_with_peak};
use crate::build_cmd::{
    regenerate_monolith_from_disk, regenerate_monolith_from_disk_quiet, run_timed_dual_build,
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

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
    emit_json_report(args, report)
}

/// Write any bench-mode report using the shared `--json` / `--output` contract.
///
/// A named output is flushed and atomically persisted in its destination directory so a
/// cancelled soak cannot leave a partially written evidence file. `--json` always keeps the
/// historical stdout-only behaviour, even when `--output` is also present.
///
/// # Errors
///
/// Returns an error when serialization, directory creation, flushing, or persistence fails.
pub fn emit_json_report<T: Serialize>(args: &BenchArgs, report: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    if args.output != "-" && !args.json {
        let path = Path::new(&args.output);
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create report directory {}", parent.display()))?;
        let mut temporary = NamedTempFile::new_in(parent)
            .with_context(|| format!("create temporary report beside {}", path.display()))?;
        temporary
            .write_all(json.as_bytes())
            .and_then(|()| temporary.write_all(b"\n"))
            .and_then(|()| temporary.as_file_mut().sync_all())
            .with_context(|| format!("flush temporary report for {}", path.display()))?;
        temporary
            .persist(path)
            .map_err(|error| error.error)
            .with_context(|| format!("persist report {}", path.display()))?;
        eprintln!("Wrote {}", args.output);
    } else {
        println!("{json}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn named_report_is_persisted_and_replaced_atomically() {
        let directory = tempfile::tempdir().expect("temporary report directory");
        let path = directory.path().join("nested").join("soak.json");
        let mut args = BenchArgs::try_parse_from(["bench"]).expect("default bench args");
        args.output = path.display().to_string();

        emit_json_report(&args, &serde_json::json!({"run": 1})).expect("first report");
        emit_json_report(&args, &serde_json::json!({"run": 2})).expect("replacement report");

        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).expect("read persisted report"))
                .expect("parse persisted report");
        assert_eq!(value, serde_json::json!({"run": 2}));
        assert_eq!(
            std::fs::read_dir(path.parent().expect("report parent"))
                .expect("list report parent")
                .count(),
            1,
            "temporary report must not remain beside the final evidence"
        );
    }
}
