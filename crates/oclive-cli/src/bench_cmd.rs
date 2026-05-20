//! `oclive bench`：对标准与 Monolith 两个二进制各跑多轮子进程采样，输出 JSON 报告。

use crate::bench_metrics::{binary_file_size, run_bench_child_with_peak};
use crate::build_cmd::{
    regenerate_monolith_from_disk, regenerate_monolith_from_disk_quiet, run_timed_dual_build,
    BuildArgs,
};
use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug, Clone)]
pub struct BenchArgs {
    /// 项目根目录（含 Cargo.toml、monolith.toml）
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,

    /// 外层重复次数（每轮一次子进程，用于分位数）
    #[arg(long, default_value_t = 20)]
    pub runs: u32,

    /// 每次子进程内由 `OCLIVE_KERNEL_BENCH_ITERS` 驱动的热循环次数
    #[arg(long, default_value_t = 400)]
    pub inner_iters: u32,

    /// 先执行 `cargo build --release`（两次构建）
    #[arg(long)]
    pub release: bool,

    /// 仅将 JSON 报告写到 stdout（便于管道与 Schema 校验）；进度信息走 stderr
    #[arg(long)]
    pub json: bool,

    /// 写入 JSON 报告路径；`-` 表示 stdout（与 `--json` 二选一效果接近时优先本参数写文件）
    #[arg(long, default_value = "-")]
    pub output: String,

    /// 将本次报告追加到项目根目录 `bench_history.json`（本地文件，勿提交）
    #[arg(long)]
    pub save: bool,

    /// 对比 `bench_history.json` 中最近两次记录（不运行采样）
    #[arg(long)]
    pub compare: bool,

    /// 打印 `bench_history.json` 全部记录的趋势表（不运行采样）
    #[arg(long)]
    pub history: bool,

    /// 监听 `src/**/*.rs` 与 `Cargo.toml`，变更后自动 release 构建 + bench（3 轮）并 --save
    #[arg(long)]
    pub watch: bool,

    /// 终端实时仪表盘（每秒 bench --runs 1，按 q 退出）
    #[arg(long)]
    pub dashboard: bool,

    /// Monolith 档位 × preset 矩阵（各 3 轮）
    #[arg(long)]
    pub matrix: bool,

    /// 与 bench_history 最近一条对比，超阈值则退出码 1
    #[arg(long)]
    pub regression: bool,

    /// 回归阈值（%），未指定时使用各指标默认（p50 5 / p95 10 / 内存与体积 5–10）
    #[arg(long)]
    pub regression_threshold: Option<f64>,

    /// 与指定 Git 引用对比性能（各 5 轮）
    #[arg(long = "compare-versions")]
    pub compare_versions: Option<String>,

    /// 透传给 `cargo build` 的附加参数（放在 `--` 之后）
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

#[derive(Serialize, Deserialize, Default)]
struct BenchHistoryFile {
    schema: u32,
    entries: Vec<BenchHistoryEntry>,
}

#[derive(Serialize, Deserialize)]
struct BenchHistoryEntry {
    ts: u64,
    report: BenchReport,
}

fn history_path(root: &Path) -> PathBuf {
    root.join("bench_history.json")
}

fn append_history(root: &Path, report: &BenchReport) -> Result<()> {
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

fn p99_from_samples(samples: &[f64]) -> f64 {
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

fn trend_arrow(prev: f64, cur: f64, lower_is_better: bool) -> &'static str {
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

fn print_bench_history(root: &Path, json_out: bool) -> Result<()> {
    let path = history_path(root);
    if !path.is_file() {
        bail!("未找到 {}；请先运行 `oclive bench --save ...`", path.display());
    }
    let raw = fs::read_to_string(&path).context("read bench_history")?;
    let file: BenchHistoryFile = serde_json::from_str(&raw).context("parse bench_history")?;
    if file.entries.is_empty() {
        bail!("bench_history 无记录；请先 `oclive bench --save`");
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
                    Some(
                        trend_arrow(p.binary_size.standard as f64, bin as f64, true).to_string(),
                    ),
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

    println!("oclive bench --history（{} 条）", file.entries.len());
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
                trend_arrow(p.binary_size.standard as f64, r.binary_size.standard as f64, true),
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
        println!("趋势（相对上一行）：Standard Monolith PeakMem Binary（↓=改善 ↑=变差 →=持平）");
    }
    Ok(())
}

/// UTC 日历日期（`YYYY-MM-DD`），无额外依赖。
fn unix_ts_to_date(ts: u64) -> String {
    let z = ts / 86_400 + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
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

fn compare_history(root: &Path) -> Result<()> {
    let path = history_path(root);
    if !path.is_file() {
        bail!("未找到 {}；请先运行 `oclive bench --save ...`", path.display());
    }
    let raw = fs::read_to_string(&path).context("read bench_history")?;
    let file: BenchHistoryFile = serde_json::from_str(&raw).context("parse bench_history")?;
    if file.entries.len() < 2 {
        bail!(
            "bench_history 记录不足 2 条（当前 {}）；请至少保存两次 bench 结果",
            file.entries.len()
        );
    }
    let a = &file.entries[file.entries.len() - 2].report;
    let b = &file.entries[file.entries.len() - 1].report;
    println!("oclive bench --compare（最近两次）");
    println!("  ts: {} -> {}", file.entries[file.entries.len() - 2].ts, file.entries[file.entries.len() - 1].ts);
    let lines = [
        ("standard 中位数 ms", a.standard_ms.p50, b.standard_ms.p50),
        ("standard P95 ms", a.standard_ms.p95, b.standard_ms.p95),
        ("standard P99 ms", p99_from_samples(&a.standard_ms.samples), p99_from_samples(&b.standard_ms.samples)),
        ("monolith 中位数 ms", a.monolith_ms.p50, b.monolith_ms.p50),
        ("monolith P95 ms", a.monolith_ms.p95, b.monolith_ms.p95),
        ("monolith P99 ms", p99_from_samples(&a.monolith_ms.samples), p99_from_samples(&b.monolith_ms.samples)),
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

fn resolve_project_root(path: &Path) -> Result<PathBuf> {
    let root = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().context("current_dir")?.join(path)
    };
    root.canonicalize()
        .with_context(|| format!("无法解析项目路径: {}", root.display()))
}

fn read_package_name(manifest_dir: &Path) -> Result<String> {
    let p = manifest_dir.join("Cargo.toml");
    let raw = fs::read_to_string(&p).context("read Cargo.toml")?;
    let v: toml::Value = toml::from_str(&raw).context("parse Cargo.toml")?;
    v.get("package")
        .and_then(|x| x.get("name"))
        .and_then(|n| n.as_str())
        .map(String::from)
        .context("Cargo.toml 缺少 [package].name")
}

fn release_bin_path(dir: &Path, name: &str, release: bool) -> PathBuf {
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

fn stats(mut samples: Vec<f64>) -> SampleStats {
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

fn cargo_build_dual(root: &Path, release: bool, extra: &[String]) -> Result<()> {
    let b = BuildArgs {
        path: root.to_path_buf(),
        no_cargo: false,
        release,
        features: vec![],
        cargo_extra: extra.to_vec(),
    };
    crate::build_cmd::run(b)
}

fn is_bench_watch_path(path: &Path, root: &Path) -> bool {
    if path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml") {
        return path.starts_with(root);
    }
    path.extension().is_some_and(|e| e == "rs") && path.starts_with(root.join("src"))
}

fn run_bench_watch(root: &Path, base: &BenchArgs) -> Result<()> {
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::{Duration, Instant};

    let mt = root.join("monolith.toml");
    if !mt.is_file() {
        bail!("bench --watch 需要 monolith.toml");
    }
    eprintln!(
        "[oclive bench --watch] 监听 {}（2s 防抖）",
        root.join("src").display()
    );
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).context("watcher")?;
    let src_dir = root.join("src");
    watcher
        .watch(&src_dir, RecursiveMode::Recursive)
        .ok();
    let cargo_toml = root.join("Cargo.toml");
    watcher
        .watch(&cargo_toml, RecursiveMode::NonRecursive)
        .ok();

    let mut last = Instant::now()
        .checked_sub(Duration::from_secs(10))
        .unwrap_or_else(Instant::now);
    let mut prev_report: Option<BenchReport> = None;

    loop {
        let mut pending = false;
        while let Ok(Ok(ev)) = rx.try_recv() {
            for p in ev.paths {
                if is_bench_watch_path(&p, root) {
                    pending = true;
                }
            }
        }
        if pending && last.elapsed() >= Duration::from_millis(2000) {
            last = Instant::now();
            eprintln!("\n[oclive bench --watch] 检测到变更，开始构建与采样…");
            let mut run_args = base.clone();
            run_args.watch = false;
            run_args.release = true;
            run_args.runs = 3;
            run_args.save = true;
            if let Err(e) = run(run_args) {
                eprintln!("[oclive bench --watch] 失败: {e}");
            } else if let Ok(file) = fs::read_to_string(history_path(root)) {
                if let Ok(hist) = serde_json::from_str::<BenchHistoryFile>(&file) {
                    if let Some(cur) = hist.entries.last() {
                        if let Some(prev) = &prev_report {
                            print_watch_delta(prev, &cur.report);
                        }
                        prev_report = Some(cur.report.clone());
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn print_watch_delta(prev: &BenchReport, cur: &BenchReport) {
    let d_std = cur.standard_ms.p50 - prev.standard_ms.p50;
    let d_mono = cur.monolith_ms.p50 - prev.monolith_ms.p50;
    println!(
        "  对比上轮: standard p50 {:+.1}ms {} | monolith p50 {:+.1}ms {}",
        d_std,
        arrow(d_std),
        d_mono,
        arrow(d_mono)
    );
}

fn arrow(delta: f64) -> &'static str {
    if delta.abs() < 0.5 {
        "→"
    } else if delta < 0.0 {
        "↓"
    } else {
        "↑"
    }
}

pub fn run(args: BenchArgs) -> Result<()> {
    let root = resolve_project_root(&args.path)?;
    if let Some(ref git_ref) = args.compare_versions {
        return run_bench_compare_versions(&root, git_ref, &args);
    }
    if args.dashboard {
        return run_bench_dashboard(&root, &args);
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
        bail!("未找到 {}", mt.display());
    }
    let file = if args.json {
        regenerate_monolith_from_disk_quiet(&root)?
    } else {
        regenerate_monolith_from_disk(&root)?
    };
    if !file.monolith.enabled {
        bail!("monolith.toml: enabled = false；bench 需要启用 Monolith 以对比双二进制。");
    }

    let build_time = if args.release {
        eprintln!("cargo build --release（标准 + Monolith，计时）…");
        let t = run_timed_dual_build(&root, true, &args.cargo_extra, file.monolith.enabled)
            .context("预热构建（标准 + Monolith）")?;
        StandardMonolithPair {
            standard: t.standard_secs,
            monolith: t.monolith_secs,
        }
    } else {
        cargo_build_dual(&root, false, &args.cargo_extra)
            .context("预热构建（标准 + Monolith）")?;
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
        eprintln!("已写入 {}", args.output);
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
            eprintln!("已追加到 {}", history_path(&root).display());
        }
        if code != 0 {
            std::process::exit(1);
        }
        return Ok(());
    }
    if args.save {
        append_history(&root, &report)?;
        eprintln!("已追加到 {}", history_path(&root).display());
    }
    if !args.json && args.output != "-" {
        print_bench_comparison(&report);
    }
    Ok(())
}

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

fn run_bench_regression(
    root: &Path,
    current: &BenchReport,
    custom_threshold: Option<f64>,
    json_out: bool,
) -> Result<i32> {
    let path = history_path(root);
    if !path.is_file() {
        anyhow::bail!("--regression 需要 bench_history.json；请先 oclive bench --save");
    }
    let raw = fs::read_to_string(&path)?;
    let file: BenchHistoryFile = serde_json::from_str(&raw)?;
    let baseline = file
        .entries
        .last()
        .map(|e| &e.report)
        .context("bench_history 无记录")?;
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
        println!("oclive bench --regression（对比 bench_history 最近一条）\n");
        for (name, base, cur, pct, limit) in &regressions {
            println!(
                "  ⚠️ {name}: {base:.1} -> {cur:.1} (+{pct:.1}% > 阈值 {limit:.0}%)"
            );
        }
        if regressions.is_empty() {
            println!("  ✅ 未检测到超过阈值的性能退化");
        }
    }
    Ok(if regressions.is_empty() { 0 } else { 1 })
}

/// 与指定 Git 引用各跑 N 轮 bench 并输出对比矩阵（`git stash` / `checkout` / 恢复）。
fn run_bench_compare_versions(root: &Path, git_ref: &str, base: &BenchArgs) -> Result<()> {
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

    eprintln!("→ 检出 {git_ref} 并采样…");
    git_checkout(&root, git_ref)?;
    let other = collect_bench_report(&root, &args_other)?;

    eprintln!("→ 恢复当前工作区并采样…");
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

fn collect_bench_report(root: &Path, args: &BenchArgs) -> Result<BenchReport> {
    let mt = root.join("monolith.toml");
    if !mt.is_file() {
        anyhow::bail!("需要 monolith.toml");
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

fn print_version_matrix(git_ref: &str, other: &BenchReport, current: &BenchReport) {
    let rows = [
        (
            "Median Lat",
            other.monolith_ms.p50,
            current.monolith_ms.p50,
        ),
        (
            "P95 Lat",
            other.monolith_ms.p95,
            current.monolith_ms.p95,
        ),
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
        eprintln!("⚠ git stash pop 有冲突，请手动解决");
    }
    Ok(())
}

fn git_checkout(root: &Path, refname: &str) -> Result<()> {
    let st = std::process::Command::new("git")
        .args(["checkout", refname])
        .current_dir(root)
        .status()?;
    if !st.success() {
        anyhow::bail!("git checkout {refname} 失败");
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

/// 终端输出标准版 vs Monolith 焊接版延迟对比（p50 / P95）。
pub fn print_bench_comparison(report: &BenchReport) {
    let std_p50 = report.standard_ms.p50;
    let mono_p50 = report.monolith_ms.p50;
    let std_p95 = report.standard_ms.p95;
    let mono_p95 = report.monolith_ms.p95;
    let improve_p50 = pct_improvement(std_p50, mono_p50);
    let improve_p95 = pct_improvement(std_p95, mono_p95);
    println!("\n—— 标准版 vs Monolith 焊接版（{} 轮, release={}）——", report.runs, report.release);
    println!("  指标        标准版(ms)   焊接版(ms)   变化");
    println!(
        "  p50         {:>10.3}   {:>10.3}   {:>+6.1}%",
        std_p50, mono_p50, improve_p50
    );
    println!(
        "  P95         {:>10.3}   {:>10.3}   {:>+6.1}%",
        std_p95, mono_p95, improve_p95
    );
    if mono_p50 < std_p50 {
        println!("  → 焊接版中位数更低（约 {:.1}%）", improve_p50);
    } else if mono_p50 > std_p50 {
        println!("  → 焊接版中位数更高；可缩小 monolith.toml 的 weld_modules");
    } else {
        println!("  → 中位数接近；可增加 runs 或检查构建 profile");
    }
    print_pair_u64("二进制 (bytes)", report.binary_size.standard, report.binary_size.monolith);
    print_pair_u64(
        "峰值内存 (MiB)",
        report.peak_memory.standard,
        report.peak_memory.monolith,
    );
    if report.release {
        print_pair_f64(
            "编译时间 (s)",
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

fn sparkline(values: &[f64]) -> String {
    const BARS: &str = "▁▂▃▄▅▆▇█";
    if values.is_empty() {
        return String::new();
    }
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = (max - min).max(1.0);
    values
        .iter()
        .map(|v| {
            let idx = (((v - min) / span) * 7.0).round() as usize;
            BARS.chars().nth(idx.min(7)).unwrap_or('▁')
        })
        .collect()
}

fn run_bench_dashboard(root: &Path, base: &BenchArgs) -> Result<()> {
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
    use crossterm::ExecutableCommand;
    use ratatui::layout::{Constraint, Direction, Layout};
    use ratatui::widgets::{Block, Borders, Paragraph};
    use std::io::stdout;
    use std::time::{Duration, Instant};

    let mt = root.join("monolith.toml");
    if !mt.is_file() {
        bail!("bench --dashboard 需要 monolith.toml");
    }
    enable_raw_mode().context("raw")?;
    stdout().execute(EnterAlternateScreen).context("alt")?;
    let mut terminal = ratatui::init();
    let mut last_run = Instant::now()
        .checked_sub(Duration::from_secs(2))
        .unwrap_or_else(Instant::now);
    let mut latest: Option<BenchReport> = None;
    let mut hist_std: Vec<f64> = Vec::new();
    let mut hist_mono: Vec<f64> = Vec::new();
    let mut quit = false;

    while !quit {
        if last_run.elapsed() >= Duration::from_secs(1) {
            last_run = Instant::now();
            let mut run_args = base.clone();
            run_args.dashboard = false;
            run_args.matrix = false;
            run_args.watch = false;
            run_args.history = false;
            run_args.compare = false;
            run_args.runs = 1;
            run_args.release = true;
            run_args.json = true;
            run_args.output = "-".into();
            run_args.save = false;
            let _ = run_args;
            if let Ok(rep) = sample_bench_once(root, base) {
                hist_std.push(rep.standard_ms.p50);
                hist_mono.push(rep.monolith_ms.p50);
                if hist_std.len() > 5 {
                    hist_std.remove(0);
                }
                if hist_mono.len() > 5 {
                    hist_mono.remove(0);
                }
                latest = Some(rep);
            }
        }

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(6),
                    Constraint::Length(3),
                ])
                .split(f.area());
            let title = Paragraph::new("oclive bench --dashboard（q 退出 · 每秒采样）")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);
            let body = if let Some(ref r) = latest {
                format!(
                    "标准版  p50 {:>7.1}ms  P95 {:>7.1}ms  峰值 {:>6}MiB  二进制 {:>6.1}MiB\n焊接版  p50 {:>7.1}ms  P95 {:>7.1}ms  峰值 {:>6}MiB  二进制 {:>6.1}MiB\n\n最近5轮 p50 趋势:\n  标准 {} {}\n  焊接 {} {}",
                    r.standard_ms.p50,
                    r.standard_ms.p95,
                    r.peak_memory.standard,
                    r.binary_size.standard as f64 / (1024.0 * 1024.0),
                    r.monolith_ms.p50,
                    r.monolith_ms.p95,
                    r.peak_memory.monolith,
                    r.binary_size.monolith as f64 / (1024.0 * 1024.0),
                    sparkline(&hist_std),
                    trend_arrow(
                        hist_std.get(hist_std.len().saturating_sub(2)).copied().unwrap_or(0.0),
                        *hist_std.last().unwrap_or(&0.0),
                        true,
                    ),
                    sparkline(&hist_mono),
                    trend_arrow(
                        hist_mono.get(hist_mono.len().saturating_sub(2)).copied().unwrap_or(0.0),
                        *hist_mono.last().unwrap_or(&0.0),
                        true,
                    ),
                )
            } else {
                "正在首次采样…".into()
            };
            f.render_widget(
                Paragraph::new(body).block(Block::default().title(" 指标 ").borders(Borders::ALL)),
                chunks[1],
            );
            f.render_widget(
                Paragraph::new(format!(
                    "sparkline: {} / {}",
                    sparkline(&hist_std),
                    sparkline(&hist_mono)
                ))
                .block(Block::default().title(" 趋势 ").borders(Borders::ALL)),
                chunks[2],
            );
        })?;

        if event::poll(Duration::from_millis(80))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    quit = true;
                }
            }
        }
    }

    ratatui::restore();
    disable_raw_mode().ok();
    let _ = stdout().execute(LeaveAlternateScreen);
    Ok(())
}

fn sample_bench_once(root: &Path, args: &BenchArgs) -> Result<BenchReport> {
    let file = regenerate_monolith_from_disk_quiet(root)?;
    if !file.monolith.enabled {
        bail!("monolith.toml: enabled = false");
    }
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
    let (ms, peak) = run_bench_child_with_peak(&std_bin, args.inner_iters)?;
    let (ms2, peak2) = run_bench_child_with_peak(&mono_bin, args.inner_iters)?;
    Ok(BenchReport {
        schema_version: BENCH_REPORT_SCHEMA_VERSION,
        package_name: pkg,
        runs: 1,
        inner_iters: args.inner_iters,
        release: args.release,
        standard_ms: stats(vec![ms]),
        monolith_ms: stats(vec![ms2]),
        binary_size,
        peak_memory: StandardMonolithPair {
            standard: peak,
            monolith: peak2,
        },
        build_time: StandardMonolithPair {
            standard: 0.0,
            monolith: 0.0,
        },
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MatrixMonolithTier {
    None,
    Latency,
    Memory,
    Embedded,
}

/// Monolith 档位 × settings preset 矩阵（各组合 `--runs` 次，默认 3）。
fn run_bench_matrix(root: &Path, base: &BenchArgs) -> Result<()> {
    let presets = ["minimal", "mixed", "full"];
    let tiers = [
        ("none", MatrixMonolithTier::None),
        ("latency", MatrixMonolithTier::Latency),
        ("memory", MatrixMonolithTier::Memory),
        ("embedded", MatrixMonolithTier::Embedded),
    ];
    let mono_backup = fs::read_to_string(root.join("monolith.toml")).ok();
    let settings_backup = backup_role_settings(root);
    let mut matrix: Vec<Vec<f64>> = vec![vec![0.0; presets.len()]; tiers.len()];

    if base.release {
        eprintln!("矩阵基准：release 预热构建…");
        cargo_build_dual(root, true, &base.cargo_extra)?;
    }

    for (ri, (tier_name, tier)) in tiers.iter().enumerate() {
        for (pi, preset) in presets.iter().enumerate() {
            eprintln!("矩阵 [{tier_name} × {preset}] …");
            apply_matrix_preset(root, preset)?;
            write_matrix_monolith(root, *tier)?;
            regenerate_monolith_from_disk_quiet(root)?;
            let mut cell_args = base.clone();
            cell_args.dashboard = false;
            cell_args.matrix = false;
            cell_args.runs = 3;
            cell_args.json = true;
            if let Ok(rep) = sample_bench_matrix_cell(root, &cell_args, *tier) {
                matrix[ri][pi] = rep;
            }
        }
    }

    restore_role_settings(root, settings_backup);
    if let Some(b) = mono_backup {
        fs::write(root.join("monolith.toml"), b)?;
        let _ = regenerate_monolith_from_disk_quiet(root);
    }

    if base.json {
        #[derive(serde::Serialize)]
        struct Out {
            rows: Vec<&'static str>,
            cols: Vec<&'static str>,
            cells_ms: Vec<Vec<f64>>,
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&Out {
                rows: tiers.iter().map(|(n, _)| *n).collect(),
                cols: presets.to_vec(),
                cells_ms: matrix,
            })?
        );
        return Ok(());
    }

    print_matrix_table(&tiers, &presets, &matrix);
    Ok(())
}

fn sample_bench_matrix_cell(
    root: &Path,
    args: &BenchArgs,
    tier: MatrixMonolithTier,
) -> Result<f64> {
    let pkg = read_package_name(root)?;
    let std_bin = release_bin_path(root, &pkg, args.release);
    let mono_bin = release_bin_path(root, &format!("{pkg}-monolith"), args.release);
    let mut mono_samples = Vec::new();
    let mut std_samples = Vec::new();
    for _ in 0..args.runs {
        let (ms, _) = run_bench_child_with_peak(&std_bin, args.inner_iters)?;
        std_samples.push(ms);
        if tier != MatrixMonolithTier::None {
            let (ms, _) = run_bench_child_with_peak(&mono_bin, args.inner_iters)?;
            mono_samples.push(ms);
        }
    }
    let std_s = stats(std_samples);
    let mono_s = stats(mono_samples);
    Ok(if tier == MatrixMonolithTier::None {
        std_s.p50
    } else {
        mono_s.p50
    })
}

fn print_matrix_table(
    tiers: &[(&str, MatrixMonolithTier)],
    presets: &[&str],
    matrix: &[Vec<f64>],
) {
    let col_w = 10usize;
    print!("{:>10}", "");
    for p in presets {
        print!(" │ {:>width$}", p, width = col_w);
    }
    println!();
    println!("{}", "─".repeat(12 + (col_w + 3) * presets.len()));
    for (i, (name, _)) in tiers.iter().enumerate() {
        print!("{name:>10}");
        for v in &matrix[i] {
            print!(" │ {:>width$.0}ms", v, width = col_w);
        }
        println!();
    }
}

fn write_matrix_monolith(root: &Path, tier: MatrixMonolithTier) -> Result<()> {
    use crate::init::MonolithPresetArg;
    let path = root.join("monolith.toml");
    let toml = match tier {
        MatrixMonolithTier::None => {
            r#"[monolith]
enabled = false
weld_modules = []
exclude = []
"#
            .to_string()
        }
        MatrixMonolithTier::Latency => {
            let w = crate::monolith_codegen::weld_modules_for_preset(MonolithPresetArg::Latency);
            let refs: Vec<&str> = w.to_vec();
            crate::monolith_codegen::render_monolith_toml_with_weld(&refs)
        }
        MatrixMonolithTier::Memory => {
            let w = crate::monolith_codegen::weld_modules_for_preset(MonolithPresetArg::Memory);
            crate::monolith_codegen::render_monolith_toml_with_weld(&w)
        }
        MatrixMonolithTier::Embedded => {
            let w = crate::monolith_codegen::weld_modules_for_preset(MonolithPresetArg::Embedded);
            crate::monolith_codegen::render_monolith_toml_with_weld(&w)
        }
    };
    fs::write(path, toml).context("write monolith.toml for matrix")
}

fn backup_role_settings(root: &Path) -> Vec<(PathBuf, Option<String>)> {
    let mut out = Vec::new();
    let roles = root.join("roles");
    if !roles.is_dir() {
        return out;
    }
    for entry in walkdir_simple(&roles) {
        if entry.ends_with("settings.json") && entry.is_file() {
            let raw = fs::read_to_string(&entry).ok();
            out.push((entry, raw));
        }
    }
    out
}

fn walkdir_simple(dir: &Path) -> Vec<PathBuf> {
    let mut stack = vec![dir.to_path_buf()];
    let mut files = Vec::new();
    while let Some(p) = stack.pop() {
        let Ok(rd) = fs::read_dir(&p) else { continue };
        for e in rd.flatten() {
            let path = e.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files
}

fn restore_role_settings(root: &Path, backup: Vec<(PathBuf, Option<String>)>) {
    let _ = root;
    for (path, raw) in backup {
        if let Some(r) = raw {
            let _ = fs::write(path, r);
        }
    }
}

fn apply_matrix_preset(root: &Path, preset: &str) -> Result<()> {
    let cfg = crate::init::preset_config("matrix", preset);
    let value = crate::generator::build_settings_value(&cfg);
    let roles = root.join("roles");
    if !roles.is_dir() {
        return Ok(());
    }
    for path in walkdir_simple(&roles) {
        if path.file_name().and_then(|n| n.to_str()) == Some("settings.json") {
            let mut existing: serde_json::Value = if path.is_file() {
                let raw = fs::read_to_string(&path)?;
                serde_json::from_str(&raw).unwrap_or(serde_json::json!({}))
            } else {
                serde_json::json!({})
            };
            if let Some(obj) = existing.as_object_mut() {
                if let Some(pb) = value.get("plugin_backends") {
                    obj.insert("plugin_backends".into(), pb.clone());
                }
            }
            fs::write(path, serde_json::to_string_pretty(&existing)?)?;
        }
    }
    Ok(())
}
