//! `oclive build`：读取 `monolith.toml`，落盘 vendor 与 `process_message_monolith.rs`，可选执行 `cargo build`（标准 + Monolith）。

use crate::monolith_codegen::{copy_monolith_vendor, generate_monolith_source};
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
    /// 项目根目录（含 Cargo.toml 与 monolith.toml）
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,

    /// 仅重新生成源码，不调用 cargo
    #[arg(long)]
    pub no_cargo: bool,

    /// 等价于 `cargo build --release`
    #[arg(long)]
    pub release: bool,

    /// 传给两次 `cargo build` 的额外 feature（逗号分隔）；Monolith 次构建会自动并入 `monolith`
    #[arg(long, value_delimiter = ',')]
    pub features: Vec<String>,

    /// 透传给 `cargo build` 的附加参数（建议放在 `--` 之后）
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
        .with_context(|| format!("无法解析项目路径: {}", root.display()))
}

fn merge_features_for_monolith(user: &[String]) -> String {
    let mut parts: Vec<String> = user.to_vec();
    if !parts.iter().any(|f| f == "monolith") {
        parts.push("monolith".into());
    }
    parts.join(",")
}

fn run_cargo_build(
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
    let st = cmd
        .status()
        .with_context(|| format!("spawn {:?}", cmd.get_program()))?;
    if !st.success() {
        bail!("cargo build 失败（退出码 {:?}）", st.code());
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
            "未找到 {}。`oclive build` 仅适用于已启用 Monolith 且含 monolith.toml 的内核脚手架项目。",
            mt.display()
        );
    }
    let text = fs::read_to_string(&mt).with_context(|| format!("read {}", mt.display()))?;
    let file = parse_monolith_toml(&text)?;
    validate_monolith_section(&file.monolith)?;
    let plan = resolve_weld_plan(&file.monolith);
    copy_monolith_vendor(root)?;
    let out_rs = root.join("src/process_message_monolith.rs");
    fs::write(&out_rs, generate_monolith_source(&plan))
        .with_context(|| format!("write {}", out_rs.display()))?;
    if log_written {
        eprintln!("已生成 {}", out_rs.display());
    }
    Ok(file)
}

pub fn run(args: BuildArgs) -> Result<()> {
    let root = resolve_project_root(&args.path)?;
    let mt = root.join("monolith.toml");
    if !mt.is_file() {
        eprintln!(
            "未找到 {}；对无 Monolith 项目执行 cargo build。",
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

    eprintln!("cargo build（标准）…");
    run_cargo_build(&root, args.release, feat_std, &args.cargo_extra)?;

    if file.monolith.enabled {
        eprintln!("cargo build（features 含 monolith）…");
        run_cargo_build(
            &root,
            args.release,
            Some(feat_mono_owned.as_str()),
            &args.cargo_extra,
        )?;
    } else {
        eprintln!("monolith.toml: enabled = false，跳过 Monolith 次构建。");
    }

    Ok(())
}
