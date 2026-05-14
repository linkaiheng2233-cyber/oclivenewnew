//! `oclive bench`：对标准与 Monolith 两个二进制各跑多轮子进程采样，输出 JSON 报告。

use crate::build_cmd::{
    regenerate_monolith_from_disk, regenerate_monolith_from_disk_quiet, BuildArgs,
};
use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

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

    /// 透传给 `cargo build` 的附加参数（放在 `--` 之后）
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub cargo_extra: Vec<String>,
}

#[derive(Serialize)]
pub struct BenchReport {
    pub schema_version: u32,
    pub package_name: String,
    pub runs: u32,
    pub inner_iters: u32,
    pub release: bool,
    pub standard_ms: SampleStats,
    pub monolith_ms: SampleStats,
}

#[derive(Serialize)]
pub struct SampleStats {
    pub samples: Vec<f64>,
    pub min: f64,
    pub max: f64,
    pub p50: f64,
    pub p95: f64,
    pub mean: f64,
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

fn run_subprocess_bench(bin: &Path, inner_iters: u32) -> Result<f64> {
    if !bin.is_file() {
        bail!("找不到二进制: {}", bin.display());
    }
    let t0 = Instant::now();
    let st = Command::new(bin)
        .env("OCLIVE_KERNEL_BENCH_ITERS", inner_iters.to_string())
        .status()
        .with_context(|| format!("run {}", bin.display()))?;
    if !st.success() {
        bail!("二进制退出失败: {:?}", st.code());
    }
    Ok(t0.elapsed().as_secs_f64() * 1000.0)
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

pub fn run(args: BenchArgs) -> Result<()> {
    let root = resolve_project_root(&args.path)?;
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

    cargo_build_dual(&root, args.release, &args.cargo_extra)
        .context("预热构建（标准 + Monolith）")?;

    let pkg = read_package_name(&root)?;
    let std_bin = release_bin_path(&root, &pkg, args.release);
    let mono_bin = release_bin_path(&root, &format!("{pkg}-monolith"), args.release);

    let mut std_samples = Vec::with_capacity(args.runs as usize);
    let mut mono_samples = Vec::with_capacity(args.runs as usize);
    for _ in 0..args.runs {
        std_samples.push(run_subprocess_bench(&std_bin, args.inner_iters)?);
        mono_samples.push(run_subprocess_bench(&mono_bin, args.inner_iters)?);
    }

    let report = BenchReport {
        schema_version: 1,
        package_name: pkg,
        runs: args.runs,
        inner_iters: args.inner_iters,
        release: args.release,
        standard_ms: stats(std_samples),
        monolith_ms: stats(mono_samples),
    };
    let json = serde_json::to_string_pretty(&report)?;
    if args.output != "-" && !args.json {
        fs::write(&args.output, &json).with_context(|| format!("write {}", args.output))?;
        eprintln!("已写入 {}", args.output);
    } else {
        println!("{json}");
    }
    Ok(())
}
