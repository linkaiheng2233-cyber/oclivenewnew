//! `oclive build`：读取 `monolith.toml`，落盘 vendor 与 `process_message_monolith.rs`，可选执行 `cargo build`（标准 + Monolith）。

use crate::monolith_codegen::{
    copy_monolith_vendor, generate_monolith_source_with_dual_core,
};
use crate::monolith_config::{
    parse_monolith_toml, resolve_weld_plan, validate_monolith_section, MonolithFile,
};
use anyhow::{bail, Context, Result};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug, Clone)]
pub struct BuildArgs {
    /// Project root (contains Cargo.toml and monolith.toml)
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,

    /// Regenerate sources only; do not invoke cargo
    #[arg(long)]
    pub no_cargo: bool,

    /// Equivalent to `cargo build --release`
    #[arg(long)]
    pub release: bool,

    /// Extra features for both cargo builds (comma-separated); Monolith build auto-adds `monolith`
    #[arg(long, value_delimiter = ',')]
    pub features: Vec<String>,

    /// Extra args forwarded to `cargo build` (place after `--`)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub cargo_extra: Vec<String>,
}

fn resolve_project_root(path: &Path) -> Result<PathBuf> {
    let root = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().context("current_dir")?.join(path)
    };
    root.canonicalize()
        .with_context(|| format!("cannot resolve project path: {}", root.display()))
}

fn merge_features_for_monolith(user: &[String]) -> String {
    let mut parts: Vec<String> = user.to_vec();
    if !parts.iter().any(|f| f == "monolith") {
        parts.push("monolith".into());
    }
    parts.join(",")
}

pub fn run_cargo_build(
    root: &Path,
    release: bool,
    features: Option<&str>,
    extra: &[String],
) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build").current_dir(root);
    if release {
        cmd.arg("--release");
    }
    if let Some(f) = features {
        if !f.is_empty() {
            cmd.arg("--features").arg(f);
        }
    }
    for a in extra {
        cmd.arg(a);
    }
    let output = cmd
        .output()
        .with_context(|| format!("spawn {:?}", cmd.get_program()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let hint = crate::cargo_hints::suggest_cargo_build_failure(&stderr);
        eprintln!("\n{hint}\n");
        if !stderr.is_empty() {
            eprintln!("--- cargo stderr ---\n{stderr}");
        }
        bail!("cargo build failed (exit code {:?})", output.status.code());
    }
    Ok(())
}

/// 读取 `monolith.toml` 并写入 vendor + `process_message_monolith.rs`（不调用 cargo）。
pub fn regenerate_monolith_from_disk(root: &Path) -> Result<MonolithFile> {
    regenerate_monolith_from_disk_inner(root, true)
}

pub fn regenerate_monolith_from_disk_quiet(root: &Path) -> Result<MonolithFile> {
    regenerate_monolith_from_disk_inner(root, false)
}

fn regenerate_monolith_from_disk_inner(root: &Path, log_written: bool) -> Result<MonolithFile> {
    let mt = root.join("monolith.toml");
    if !mt.is_file() {
        bail!(
            "Not found: {}. `oclive build` applies only to kernel scaffolds with Monolith enabled and monolith.toml.",
            mt.display()
        );
    }
    let text = fs::read_to_string(&mt).with_context(|| format!("read {}", mt.display()))?;
    let file = parse_monolith_toml(&text)?;
    validate_monolith_section(&file.monolith)?;
    let plan = resolve_weld_plan(&file.monolith);
    copy_monolith_vendor(root)?;
    let out_rs = root.join("src/process_message_monolith.rs");
    fs::write(
        &out_rs,
        generate_monolith_source_with_dual_core(&plan, file.dual_core.enabled),
    )
        .with_context(|| format!("write {}", out_rs.display()))?;
    if log_written {
        eprintln!("Generated {}", out_rs.display());
    }
    Ok(file)
}

/// 标准 / Monolith 两次 `cargo build` 的耗时（秒）。
#[derive(Debug, Clone, Copy)]
pub struct DualBuildTimings {
    pub standard_secs: f64,
    pub monolith_secs: f64,
}

/// 在已有 `monolith.toml` 且 enabled 时分别计时两次 release 构建。
pub fn run_timed_dual_build(
    root: &Path,
    release: bool,
    extra: &[String],
    monolith_enabled: bool,
) -> Result<DualBuildTimings> {
    let feat_std_owned = None::<String>;
    let feat_std = feat_std_owned.as_deref();
    let t0 = std::time::Instant::now();
    run_cargo_build(root, release, feat_std, extra)?;
    let standard_secs = t0.elapsed().as_secs_f64();
    let monolith_secs = if monolith_enabled {
        let feat_mono = merge_features_for_monolith(&[]);
        let t1 = std::time::Instant::now();
        run_cargo_build(root, release, Some(feat_mono.as_str()), extra)?;
        t1.elapsed().as_secs_f64()
    } else {
        0.0
    };
    Ok(DualBuildTimings {
        standard_secs,
        monolith_secs,
    })
}

pub fn run(args: BuildArgs) -> Result<()> {
    let root = resolve_project_root(&args.path)?;
    let mt = root.join("monolith.toml");
    if !mt.is_file() {
        eprintln!(
            "Not found: {}; running cargo build for non-Monolith project.",
            mt.display()
        );
        if args.no_cargo {
            return Ok(());
        }
        let feat = if args.features.is_empty() {
            None
        } else {
            Some(args.features.join(","))
        };
        return run_cargo_build(&root, args.release, feat.as_deref(), &args.cargo_extra);
    }

    let file = regenerate_monolith_from_disk(&root)?;

    if args.no_cargo {
        return Ok(());
    }

    let feat_std_owned = if args.features.is_empty() {
        None
    } else {
        Some(args.features.join(","))
    };
    let feat_std = feat_std_owned.as_deref();
    let feat_mono_owned = merge_features_for_monolith(&args.features);

    eprintln!("cargo build (standard)…");
    run_cargo_build(&root, args.release, feat_std, &args.cargo_extra)?;

    if file.monolith.enabled {
        eprintln!("cargo build (features include monolith)…");
        run_cargo_build(
            &root,
            args.release,
            Some(feat_mono_owned.as_str()),
            &args.cargo_extra,
        )?;
    } else {
        eprintln!("monolith.toml: enabled = false; skipping Monolith build.");
    }

    Ok(())
}
