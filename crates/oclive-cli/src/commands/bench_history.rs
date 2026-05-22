//! Bench history (`--history`, `--compare`, `--save`).

use super::bench::BenchReport;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
#[derive(Serialize, Deserialize, Default)]
pub(crate) struct BenchHistoryFile {
    schema: u32,
    pub(crate) entries: Vec<BenchHistoryEntry>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct BenchHistoryEntry {
    ts: u64,
    pub(crate) report: BenchReport,
}

pub(crate) fn history_path(root: &Path) -> PathBuf {
    root.join("bench_history.json")
}

pub(crate) fn append_history(root: &Path, report: &BenchReport) -> Result<()> {
    let path = history_path(root);
    let mut file: BenchHistoryFile = if path.is_file() {
        let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        serde_json::from_str(&raw).context("parse bench_history.json")?
    } else {
        BenchHistoryFile {
            schema: 1,
            entries: vec![],
        }
    };
    file.entries.push(BenchHistoryEntry {
        ts: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        report: report.clone(),
    });
    if file.entries.len() > 64 {
        file.entries.drain(0..file.entries.len() - 64);
    }
    let out = serde_json::to_string_pretty(&file)?;
    fs::write(&path, out).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn percentile_of_sorted(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

pub(crate) fn p99_from_samples(samples: &[f64]) -> f64 {
    let mut s = samples.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    percentile_of_sorted(&s, 0.99)
}

fn format_ms(v: f64) -> String {
    format!("{:.0}ms", v)
}

fn format_mib(bytes: u64) -> String {
    if bytes == 0 {
        "—".into()
    } else {
        format!("{:.0}MiB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn format_binary(bytes: u64) -> String {
    if bytes == 0 {
        "—".into()
    } else {
        format!("{:.1}MiB", bytes as f64 / (1024.0 * 1024.0))
    }
}

pub(crate) fn trend_arrow(prev: f64, cur: f64, lower_is_better: bool) -> &'static str {
    let eps = 0.5;
    if (cur - prev).abs() < eps {
        return "→";
    }
    if lower_is_better {
        if cur < prev {
            "↓"
        } else {
            "↑"
        }
    } else if cur > prev {
        "↑"
    } else {
        "↓"
    }
}

pub(crate) fn print_bench_history(root: &Path, json_out: bool) -> Result<()> {
    let path = history_path(root);
    if !path.is_file() {
        bail!(
            "Not found: {}; run `oclive bench --save ...` first",
            path.display()
        );
    }
    let raw = fs::read_to_string(&path).context("read bench_history")?;
    let file: BenchHistoryFile = serde_json::from_str(&raw).context("parse bench_history")?;
    if file.entries.is_empty() {
        bail!("bench_history has no entries; run `oclive bench --save` first");
    }
    if json_out {
        #[derive(Serialize)]
        struct Row {
            date: String,
            standard_ms: f64,
            monolith_ms: f64,
            peak_mem_mib: u64,
            binary_mib: f64,
            trend_standard: Option<String>,
            trend_monolith: Option<String>,
            trend_peak_mem: Option<String>,
            trend_binary: Option<String>,
        }
        let mut rows = Vec::new();
        for (i, e) in file.entries.iter().enumerate() {
            let r = &e.report;
            let date = unix_ts_to_date(e.ts);
            let peak = r.peak_memory.standard.max(r.peak_memory.monolith);
            let bin = r.binary_size.standard;
            let (ts, tm, tp, tb) = if i > 0 {
                let p = &file.entries[i - 1].report;
                (
                    Some(trend_arrow(p.standard_ms.p50, r.standard_ms.p50, true).to_string()),
                    Some(trend_arrow(p.monolith_ms.p50, r.monolith_ms.p50, true).to_string()),
                    Some(
                        trend_arrow(
                            p.peak_memory.standard.max(p.peak_memory.monolith) as f64,
                            peak as f64,
                            true,
                        )
                        .to_string(),
                    ),
                    Some(trend_arrow(p.binary_size.standard as f64, bin as f64, true).to_string()),
                )
            } else {
                (None, None, None, None)
            };
            rows.push(Row {
                date,
                standard_ms: r.standard_ms.p50,
                monolith_ms: r.monolith_ms.p50,
                peak_mem_mib: peak,
                binary_mib: bin as f64 / (1024.0 * 1024.0),
                trend_standard: ts,
                trend_monolith: tm,
                trend_peak_mem: tp,
                trend_binary: tb,
            });
        }
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }

    println!("oclive bench --history ({} entries)", file.entries.len());
    println!("┌────────────┬──────────┬────────────┬──────────┬──────────┐");
    println!("│ Date       │ Standard │ Monolith   │ Peak Mem │ Binary   │");
    println!("├────────────┼──────────┼────────────┼──────────┼──────────┤");
    for (i, e) in file.entries.iter().enumerate() {
        let r = &e.report;
        let date = unix_ts_to_date(e.ts);
        let std = format_ms(r.standard_ms.p50);
        let mono = format_ms(r.monolith_ms.p50);
        let peak = format_mib(r.peak_memory.standard.max(r.peak_memory.monolith));
        let bin = format_binary(r.binary_size.standard);
        let trend = if i > 0 && file.entries.len() >= 2 {
            let p = &file.entries[i - 1].report;
            format!(
                " {} {} {} {}",
                trend_arrow(p.standard_ms.p50, r.standard_ms.p50, true),
                trend_arrow(p.monolith_ms.p50, r.monolith_ms.p50, true),
                trend_arrow(
                    p.peak_memory.standard.max(p.peak_memory.monolith) as f64,
                    r.peak_memory.standard.max(r.peak_memory.monolith) as f64,
                    true,
                ),
                trend_arrow(
                    p.binary_size.standard as f64,
                    r.binary_size.standard as f64,
                    true
                ),
            )
        } else {
            String::new()
        };
        println!(
            "│ {:10} │ {:>8} │ {:>10} │ {:>8} │ {:>8} │{trend}",
            date, std, mono, peak, bin
        );
    }
    println!("└────────────┴──────────┴────────────┴──────────┴──────────┘");
    if file.entries.len() >= 2 {
        println!(
            "Trend (vs previous row): Standard Monolith PeakMem Binary (↓=better ↑=worse →=flat)"
        );
    }
    Ok(())
}

/// UTC 日历日期（`YYYY-MM-DD`），无额外依赖。
fn unix_ts_to_date(ts: u64) -> String {
    let z = ts / 86_400 + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp as i32 + if mp < 10 { 3 } else { -9 };
    let y = y + if m <= 2 { 1 } else { 0 };
    let m = m as u32;
    format!("{y:04}-{m:02}-{d:02}")
}

pub(crate) fn compare_history(root: &Path) -> Result<()> {
    let path = history_path(root);
    if !path.is_file() {
        bail!(
            "Not found: {}; run `oclive bench --save ...` first",
            path.display()
        );
    }
    let raw = fs::read_to_string(&path).context("read bench_history")?;
    let file: BenchHistoryFile = serde_json::from_str(&raw).context("parse bench_history")?;
    if file.entries.len() < 2 {
        bail!(
            "bench_history has fewer than 2 entries (current {}); save at least two bench results",
            file.entries.len()
        );
    }
    let a = &file.entries[file.entries.len() - 2].report;
    let b = &file.entries[file.entries.len() - 1].report;
    println!("oclive bench --compare (last two runs)");
    println!(
        "  ts: {} -> {}",
        file.entries[file.entries.len() - 2].ts,
        file.entries[file.entries.len() - 1].ts
    );
    let lines = [
        ("standard median ms", a.standard_ms.p50, b.standard_ms.p50),
        ("standard P95 ms", a.standard_ms.p95, b.standard_ms.p95),
        (
            "standard P99 ms",
            p99_from_samples(&a.standard_ms.samples),
            p99_from_samples(&b.standard_ms.samples),
        ),
        ("monolith median ms", a.monolith_ms.p50, b.monolith_ms.p50),
        ("monolith P95 ms", a.monolith_ms.p95, b.monolith_ms.p95),
        (
            "monolith P99 ms",
            p99_from_samples(&a.monolith_ms.samples),
            p99_from_samples(&b.monolith_ms.samples),
        ),
    ];
    for (label, x, y) in lines {
        let d = y - x;
        let pct = if x.abs() > f64::EPSILON {
            (d / x) * 100.0
        } else {
            0.0
        };
        let warn = if label.contains("monolith") && d > 0.0 && pct > 5.0 {
            " ⚠️"
        } else {
            ""
        };
        println!("  {label}: {x:.3} -> {y:.3} (Δ {d:+.3} ms, {pct:+.1}%){warn}");
    }
    Ok(())
}
