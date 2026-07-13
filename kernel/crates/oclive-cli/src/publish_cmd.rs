//! `oclive publish` and remote templates via `init --template-url`.

use anyhow::{bail, Context, Result};
use clap::Parser;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use tar::{Builder, Header};

#[derive(Parser, Debug)]
pub struct PublishArgs {
    /// Publish type (template only for now)
    #[arg(long, value_enum, default_value_t = PublishTypeArg::Template)]
    pub r#type: PublishTypeArg,

    /// Project root (default: current directory)
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,

    /// Output file (default: ./<package>-<version>.oclive-template.tar.gz)
    #[arg(short = 'O', long)]
    pub output: Option<PathBuf>,
}

#[derive(clap::ValueEnum, Clone, Debug, Default)]
pub enum PublishTypeArg {
    #[default]
    Template,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub scene: String,
    pub preset: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monolith_preset: Option<String>,
    pub project_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monolith_enabled: Option<bool>,
}

const EXCLUDE_DIRS: &[&str] = &[
    "target",
    ".git",
    "bench_results",
    "node_modules",
    ".oclive-compose.pids.json",
];

const EXCLUDE_FILES: &[&str] = &["bench_history.json", ".oclive-compose.pids.json"];

/// `oclive template pack`: package the project into `.oclive-template.tar.gz`.
pub fn run_template_pack(path: PathBuf, output: Option<PathBuf>) -> Result<()> {
    publish_template(&PublishArgs {
        r#type: PublishTypeArg::Template,
        path,
        output,
    })
}

/// Package the project root into `.oclive-template.tar.gz` (reused by registry push, etc.).
pub fn pack_template_tarball(root: &Path, out: &Path) -> Result<()> {
    let cargo_toml = root.join("Cargo.toml");
    if !cargo_toml.is_file() {
        anyhow::bail!("{} is not a Cargo project root", root.display());
    }
    let (name, version) = read_package_meta(&cargo_toml)?;
    let template_meta = TemplateManifest {
        name: name.clone(),
        description: format!("{name} oclive kernel template"),
        version: version.clone(),
        scene: "custom".into(),
        preset: "minimal".into(),
        monolith_preset: if root.join("monolith.toml").is_file() {
            Some("latency".into())
        } else {
            None
        },
        project_type: if root.join("src/main.rs").is_file() {
            "kernel_server".into()
        } else {
            "library".into()
        },
        monolith_enabled: Some(root.join("monolith.toml").is_file()),
    };
    pack_template_tarball_with_meta(root, out, &template_meta)
}

pub fn pack_template_tarball_with_meta(
    root: &Path,
    out: &Path,
    template_meta: &TemplateManifest,
) -> Result<()> {
    let tmp = tempfile::tempdir().context("tempdir")?;
    let staging = tmp.path().join("bundle");
    copy_tree_filtered(root, &staging)?;
    fs::write(
        staging.join("template.json"),
        serde_json::to_string_pretty(template_meta)?,
    )?;
    write_tar_gz(&staging, out)?;
    Ok(())
}

pub fn publish_template(args: &PublishArgs) -> Result<()> {
    let root = args
        .path
        .canonicalize()
        .with_context(|| format!("path {}", args.path.display()))?;
    let cargo_toml = root.join("Cargo.toml");
    if !cargo_toml.is_file() {
        bail!("{} is not a Cargo project root", root.display());
    }
    let (name, version) = read_package_meta(&cargo_toml)?;
    let out = args.output.clone().unwrap_or_else(|| {
        root.parent()
            .unwrap_or(&root)
            .join(format!("{name}-{version}.oclive-template.tar.gz"))
    });
    pack_template_tarball(&root, &out)?;
    println!("Template package written: {}", out.display());
    Ok(())
}

pub fn init_from_template_url(url: &str, output: &Path) -> Result<()> {
    if output.exists() {
        bail!("Output directory already exists: {}", output.display());
    }
    fs::create_dir_all(output).context("create output")?;
    let tmp = tempfile::tempdir()?;
    let archive = tmp.path().join("dl.tar.gz");
    eprintln!("Downloading {url} …");
    let resp = crate::http_client::get(url)
        .call()
        .context("HTTP GET template")?;
    let mut reader = resp.into_reader();
    let mut file = File::create(&archive)?;
    std::io::copy(&mut reader, &mut file)?;
    extract_tar_gz(&archive, output)?;
    println!("Initialized from remote template: {}", output.display());
    Ok(())
}

fn read_package_meta(cargo_toml: &Path) -> Result<(String, String)> {
    let raw = fs::read_to_string(cargo_toml)?;
    let v: toml::Value = toml::from_str(&raw)?;
    let name = v
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .context("Cargo.toml [package].name")?
        .to_string();
    let version = v
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|n| n.as_str())
        .unwrap_or("0.1.0")
        .to_string();
    Ok((name, version))
}

fn should_skip(rel: &Path) -> bool {
    for part in rel.components() {
        let s = part.as_os_str().to_string_lossy();
        if EXCLUDE_DIRS.iter().any(|d| d == &s) {
            return true;
        }
        if EXCLUDE_FILES.iter().any(|f| f == &s) {
            return true;
        }
    }
    false
}

fn copy_tree_filtered(src_root: &Path, dst_root: &Path) -> Result<()> {
    let mut stack = vec![src_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let rel = path.strip_prefix(src_root).unwrap_or(&path);
            if should_skip(rel) {
                continue;
            }
            let to = dst_root.join(rel);
            if path.is_dir() {
                fs::create_dir_all(&to)?;
                stack.push(path);
            } else {
                if let Some(parent) = to.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&path, &to)?;
            }
        }
    }
    Ok(())
}

fn write_tar_gz(src_dir: &Path, out_path: &Path) -> Result<()> {
    let file = File::create(out_path)?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut tar = Builder::new(enc);
    for entry in walkdir_flat(src_dir)? {
        let path = entry.0;
        let rel = path.strip_prefix(src_dir).unwrap();
        let mut header = Header::new_gnu();
        if path.is_file() {
            let mut f = File::open(&path)?;
            header.set_size(f.metadata()?.len());
            header.set_mode(0o644);
            header.set_path(rel)?;
            header.set_cksum();
            tar.append(&header, &mut f)?;
        }
    }
    tar.finish()?;
    Ok(())
}

fn walkdir_flat(dir: &Path) -> Result<Vec<(PathBuf, bool)>> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d)? {
            let entry = entry?;
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.push((p, false));
            }
        }
    }
    Ok(out)
}

pub fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<()> {
    let file = File::open(archive)?;
    let dec = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(dec);
    archive.unpack(dest).context("unpack template")?;
    Ok(())
}
