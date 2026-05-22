use super::super::bench::{run, BenchArgs, BenchReport};
use super::super::bench_history::{history_path, BenchHistoryFile};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
fn is_bench_watch_path(path: &Path, root: &Path) -> bool {
    if path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml") {
        return path.starts_with(root);
    }
    path.extension().is_some_and(|e| e == "rs") && path.starts_with(root.join("src"))
}

pub(crate) fn run_bench_watch(root: &Path, base: &BenchArgs) -> Result<()> {
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::{Duration, Instant};

    let mt = root.join("monolith.toml");
    if !mt.is_file() {
        bail!("bench --watch requires monolith.toml");
    }
    eprintln!(
        "[oclive bench --watch] watching {} (2s debounce)",
        root.join("src").display()
    );
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).context("watcher")?;
    let src_dir = root.join("src");
    watcher.watch(&src_dir, RecursiveMode::Recursive).ok();
    let cargo_toml = root.join("Cargo.toml");
    watcher.watch(&cargo_toml, RecursiveMode::NonRecursive).ok();

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
            eprintln!("\n[oclive bench --watch] change detected; building and sampling…");
            let mut run_args = base.clone();
            run_args.watch = false;
            run_args.release = true;
            run_args.runs = 3;
            run_args.save = true;
            if let Err(e) = run(run_args) {
                eprintln!("[oclive bench --watch] failed: {e}");
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
        "  vs last run: standard p50 {:+.1}ms {} | monolith p50 {:+.1}ms {}",
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
