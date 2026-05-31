use super::super::bench::{collect_bench_report, BenchArgs, BenchReport};
use super::super::bench_history::{history_path, BenchHistoryFile};
use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;
struct RegressionThresholds {
    p50: f64,
    p95: f64,
    peak_mem: f64,
    binary: f64,
}

fn default_thresholds(custom: Option<f64>) -> RegressionThresholds {
    if let Some(t) = custom {
        return RegressionThresholds {
            p50: t,
            p95: t,
            peak_mem: t,
            binary: t,
        };
    }
    RegressionThresholds {
        p50: 5.0,
        p95: 10.0,
        peak_mem: 10.0,
        binary: 5.0,
    }
}

pub(crate) fn run_bench_regression(
    root: &Path,
    current: &BenchReport,
    custom_threshold: Option<f64>,
    json_out: bool,
) -> Result<i32> {
    let path = history_path(root);
    if !path.is_file() {
        anyhow::bail!("--regression requires bench_history.json; run oclive bench --save first");
    }
    let raw = fs::read_to_string(&path)?;
    let file: BenchHistoryFile = serde_json::from_str(&raw)?;
    let baseline = file
        .entries
        .last()
        .map(|e| &e.report)
        .context("bench_history has no entries")?;
    let th = default_thresholds(custom_threshold);
    let mut regressions = Vec::new();

    let checks = [
        (
            "monolith_p50",
            baseline.monolith_ms.p50,
            current.monolith_ms.p50,
            th.p50,
        ),
        (
            "monolith_p95",
            baseline.monolith_ms.p95,
            current.monolith_ms.p95,
            th.p95,
        ),
        (
            "peak_memory",
            baseline.peak_memory.monolith as f64,
            current.peak_memory.monolith as f64,
            th.peak_mem,
        ),
        (
            "binary_size",
            baseline.binary_size.monolith as f64,
            current.binary_size.monolith as f64,
            th.binary,
        ),
    ];
    for (name, base, cur, limit) in checks {
        let pct = if base <= 0.0 {
            0.0
        } else {
            ((cur - base) / base) * 100.0
        };
        if pct > limit {
            regressions.push((name, base, cur, pct, limit));
        }
    }

    if json_out {
        #[derive(Serialize)]
        struct Row {
            metric: String,
            baseline: f64,
            current: f64,
            change_pct: f64,
            threshold_pct: f64,
            regressed: bool,
        }
        let rows: Vec<Row> = checks
            .iter()
            .map(|(name, base, cur, limit)| {
                let pct = if *base <= 0.0 {
                    0.0
                } else {
                    ((cur - base) / base) * 100.0
                };
                Row {
                    metric: (*name).into(),
                    baseline: *base,
                    current: *cur,
                    change_pct: pct,
                    threshold_pct: *limit,
                    regressed: pct > *limit,
                }
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&rows)?);
    } else {
        println!("oclive bench --regression (vs latest bench_history entry)\n");
        for (name, base, cur, pct, limit) in &regressions {
            println!("  ⚠️ {name}: {base:.1} -> {cur:.1} (+{pct:.1}% > threshold {limit:.0}%)");
        }
        if regressions.is_empty() {
            println!("  ✅ No regression above threshold detected");
        }
    }
    Ok(if regressions.is_empty() { 0 } else { 1 })
}

/// Runs N bench rounds against the specified Git ref and outputs a comparison matrix (`git stash` / `checkout` / restore).
pub(crate) fn run_bench_compare_versions(root: &Path, git_ref: &str, base: &BenchArgs) -> Result<()> {
    let root = root.canonicalize()?;
    let mut args_other = base.clone();
    args_other.compare_versions = None;
    args_other.regression = false;
    args_other.runs = 5;
    args_other.release = true;
    args_other.save = false;

    eprintln!("oclive bench --compare-versions {git_ref}");
    let stashed = git_stash_push(&root)?;
    let original_ref = git_current_ref(&root)?;

    eprintln!("→ Checking out {git_ref} and sampling…");
    git_checkout(&root, git_ref)?;
    let other = collect_bench_report(&root, &args_other)?;

    eprintln!("→ Restoring workspace and sampling…");
    git_checkout(&root, &original_ref)?;
    if stashed {
        git_stash_pop(&root)?;
    }
    let current = collect_bench_report(&root, &args_other)?;

    if base.json {
        #[derive(Serialize)]
        struct Cmp {
            git_ref: String,
            other: BenchReport,
            current: BenchReport,
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&Cmp {
                git_ref: git_ref.to_string(),
                other,
                current,
            })?
        );
        return Ok(());
    }

    print_version_matrix(git_ref, &other, &current);
    Ok(())
}

fn print_version_matrix(git_ref: &str, other: &BenchReport, current: &BenchReport) {
    let rows = [
        ("Median Lat", other.monolith_ms.p50, current.monolith_ms.p50),
        ("P95 Lat", other.monolith_ms.p95, current.monolith_ms.p95),
        (
            "Peak Memory",
            other.peak_memory.monolith as f64,
            current.peak_memory.monolith as f64,
        ),
        (
            "Binary Size",
            other.binary_size.monolith as f64,
            current.binary_size.monolith as f64,
        ),
    ];
    println!("┌──────────────┬──────────┬──────────┬────────┐");
    println!("│ Metric       │ {:>8} │ Current  │ Change │", git_ref);
    println!("├──────────────┼──────────┼──────────┼────────┤");
    for (label, base, cur) in rows {
        let ch = pct_change_label(base, cur);
        let b = format_metric(label, base);
        let c = format_metric(label, cur);
        println!("│ {:12} │ {:>8} │ {:>8} │ {:>6} │", label, b, c, ch);
    }
    println!("└──────────────┴──────────┴──────────┴────────┘");
}

fn format_metric(label: &str, v: f64) -> String {
    if label.contains("Memory") || label.contains("Binary") {
        format!("{:.0}MiB", v / (1024.0 * 1024.0))
    } else {
        format!("{:.0}ms", v)
    }
}

fn pct_change_label(base: f64, cur: f64) -> String {
    if base.abs() <= f64::EPSILON {
        return "—".into();
    }
    let pct = ((cur - base) / base) * 100.0;
    if pct.abs() < 0.5 {
        "→".into()
    } else if pct < 0.0 {
        format!("↓ {:.1}%", pct.abs())
    } else {
        format!("↑ {:.1}%", pct)
    }
}

fn git_stash_push(root: &Path) -> Result<bool> {
    let st = std::process::Command::new("git")
        .args(["stash", "push", "-u", "-m", "oclive-bench-compare"])
        .current_dir(root)
        .output()?;
    Ok(st.status.success() && !String::from_utf8_lossy(&st.stdout).trim().is_empty())
}

fn git_stash_pop(root: &Path) -> Result<()> {
    let st = std::process::Command::new("git")
        .args(["stash", "pop"])
        .current_dir(root)
        .status()?;
    if !st.success() {
        eprintln!("⚠ git stash pop conflict; resolve manually");
    }
    Ok(())
}

fn git_checkout(root: &Path, refname: &str) -> Result<()> {
    let st = std::process::Command::new("git")
        .args(["checkout", refname])
        .current_dir(root)
        .status()?;
    if !st.success() {
        anyhow::bail!("git checkout {refname} failed");
    }
    Ok(())
}

fn git_current_ref(root: &Path) -> Result<String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()?;
    if !out.status.success() {
        return Ok("HEAD".into());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
