use super::super::bench::{
    cargo_build_dual, read_package_name, release_bin_path, stats, BenchArgs, BenchReport,
    StandardMonolithPair, BENCH_REPORT_SCHEMA_VERSION,
};
use super::super::bench_history::trend_arrow;
use crate::bench_metrics::{binary_file_size, run_bench_child_with_peak};
use crate::build_cmd::{regenerate_monolith_from_disk_quiet, run_timed_dual_build};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
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

pub(crate) fn run_bench_live(root: &Path, base: &BenchArgs) -> Result<()> {
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};
    use crossterm::terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    };
    use crossterm::ExecutableCommand;
    use ratatui::layout::{Constraint, Direction, Layout};
    use ratatui::widgets::{Block, Borders, Paragraph};
    use std::io::stdout;
    use std::time::{Duration, Instant};

    let mt = root.join("monolith.toml");
    if !mt.is_file() {
        bail!("bench --live requires monolith.toml");
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
            run_args.live = false;
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
            let title = Paragraph::new("oclive bench --live (q quit · sample every second)")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);
            let body = if let Some(ref r) = latest {
                format!(
                    "standard  p50 {:>7.1}ms  P95 {:>7.1}ms  peak {:>6}MiB  binary {:>6.1}MiB\nmonolith  p50 {:>7.1}ms  P95 {:>7.1}ms  peak {:>6}MiB  binary {:>6.1}MiB\n\nlast 5 runs p50 trend:\n  std {} {}\n  mono {} {}",
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
                "Running first sample…".into()
            };
            f.render_widget(
                Paragraph::new(body).block(Block::default().title(" metrics ").borders(Borders::ALL)),
                chunks[1],
            );
            f.render_widget(
                Paragraph::new(format!(
                    "sparkline: {} / {}",
                    sparkline(&hist_std),
                    sparkline(&hist_mono)
                ))
                .block(Block::default().title(" trend ").borders(Borders::ALL)),
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

/// Monolith tier x settings preset matrix (each combination runs `--runs` times, default 3).
pub(crate) fn run_bench_matrix(root: &Path, base: &BenchArgs) -> Result<()> {
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
        eprintln!("Matrix baseline: release warm-up build…");
        cargo_build_dual(root, true, &base.cargo_extra)?;
    }

    for (ri, (tier_name, tier)) in tiers.iter().enumerate() {
        for (pi, preset) in presets.iter().enumerate() {
            eprintln!("matrix [{tier_name} × {preset}] …");
            apply_matrix_preset(root, preset)?;
            write_matrix_monolith(root, *tier)?;
            regenerate_monolith_from_disk_quiet(root)?;
            let mut cell_args = base.clone();
            cell_args.live = false;
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

fn print_matrix_table(tiers: &[(&str, MatrixMonolithTier)], presets: &[&str], matrix: &[Vec<f64>]) {
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
        MatrixMonolithTier::None => r#"[monolith]
enabled = false
weld_modules = []
exclude = []
"#
        .to_string(),
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
