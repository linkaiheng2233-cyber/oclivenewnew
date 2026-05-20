//! `oclive template create` — 从现有工程反向生成模板。

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

use crate::publish_cmd::{pack_template_tarball_with_meta, TemplateManifest};
use crate::registry::oclive_home;

#[derive(Parser, Debug)]
pub struct TemplateCli {
    #[command(subcommand)]
    pub command: TemplateCommands,
}

#[derive(Subcommand, Debug)]
pub enum TemplateCommands {
    /// 从当前工程反向生成可复用模板
    Create(TemplateCreateArgs),
}

#[derive(Parser, Debug)]
pub struct TemplateCreateArgs {
    pub name: String,
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long, default_value = "custom")]
    pub category: String,
    #[arg(short = 'O', long)]
    pub output: Option<PathBuf>,
}

pub fn run_create(args: TemplateCreateArgs) -> Result<()> {
    let root = args.path.canonicalize().context("path")?;
    let meta = analyze_project(&root, &args)?;
    let out = args.output.clone().unwrap_or_else(|| {
        root.parent()
            .unwrap_or(&root)
            .join(format!("{}.oclive-template.tar.gz", args.name))
    });
    pack_template_tarball_with_meta(&root, &out, &meta)?;
    let lib_dir = oclive_home().join("templates");
    fs::create_dir_all(&lib_dir)?;
    let dest = lib_dir.join(format!("{}.oclive-template.tar.gz", args.name));
    fs::copy(&out, &dest).with_context(|| format!("copy to {}", dest.display()))?;
    register_local_template(&args.name, &dest, &meta)?;
    println!("已生成模板包: {}", out.display());
    println!("已注册本地模板库: {}", dest.display());
    println!("使用: oclive init --template-url file://{}", dest.display());
    Ok(())
}

fn analyze_project(root: &Path, args: &TemplateCreateArgs) -> Result<TemplateManifest> {
    let cargo_toml = root.join("Cargo.toml");
    if !cargo_toml.is_file() {
        bail!("缺少 Cargo.toml");
    }
    let (pkg_name, version) = read_package_meta(&cargo_toml)?;
    let project_type = if root.join("src/main.rs").is_file() {
        "kernel_server"
    } else {
        "library"
    };
    let monolith = root.join("monolith.toml").is_file();
    let (preset, monolith_preset) = infer_preset_and_monolith(root, monolith);
    Ok(TemplateManifest {
        name: args.name.clone(),
        version,
        description: args
            .description
            .clone()
            .unwrap_or_else(|| format!("Auto-generated from {}", pkg_name)),
        scene: args.category.clone(),
        preset,
        monolith_preset,
        project_type: project_type.into(),
        monolith_enabled: Some(monolith),
    })
}

fn infer_preset_and_monolith(root: &Path, monolith: bool) -> (String, Option<String>) {
    let mut preset = "minimal".to_string();
    let mut mono_preset = None;
    if monolith {
        if let Ok(raw) = fs::read_to_string(root.join("monolith.toml")) {
            if raw.contains("memory") && raw.contains("prompt") && !raw.contains("emotion") {
                mono_preset = Some("memory".into());
            } else if raw.contains("emotion") {
                mono_preset = Some("embedded".into());
            } else {
                mono_preset = Some("latency".into());
            }
        }
    }
    if let Some(settings) = find_first_settings(root) {
        if let Ok(v) = serde_json::from_str::<Value>(&settings) {
            if let Some(llm) = v
                .get("plugin_backends")
                .and_then(|p| p.get("llm"))
                .and_then(|x| x.as_str())
            {
                preset = match llm {
                    "remote" => "full",
                    "ollama" => "mixed",
                    _ => "minimal",
                }
                .into();
            }
        }
    }
    (preset, mono_preset)
}

fn find_first_settings(root: &Path) -> Option<String> {
    let roles = root.join("roles");
    if !roles.is_dir() {
        return None;
    }
    for e in fs::read_dir(&roles).ok()?.flatten() {
        let s = e.path().join("settings.json");
        if s.is_file() {
            return fs::read_to_string(&s).ok();
        }
    }
    None
}

fn read_package_meta(cargo_toml: &Path) -> Result<(String, String)> {
    let raw = fs::read_to_string(cargo_toml)?;
    let v: toml::Value = toml::from_str(&raw)?;
    let name = v
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .context("[package].name")?
        .to_string();
    let version = v
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|n| n.as_str())
        .unwrap_or("0.1.0")
        .to_string();
    Ok((name, version))
}

fn register_local_template(name: &str, path: &Path, meta: &TemplateManifest) -> Result<()> {
    let index_path = oclive_home().join("templates/index.json");
    let mut entries: Vec<Value> = if index_path.is_file() {
        serde_json::from_str(&fs::read_to_string(&index_path)?).unwrap_or_default()
    } else {
        vec![]
    };
    entries.retain(|e| e.get("name").and_then(|n| n.as_str()) != Some(name));
    entries.push(serde_json::json!({
        "name": name,
        "path": path.display().to_string(),
        "description": meta.description,
        "preset": meta.preset,
        "project_type": meta.project_type,
    }));
    fs::write(&index_path, serde_json::to_string_pretty(&entries)?)?;
    Ok(())
}

pub fn run(cli: TemplateCli) -> Result<()> {
    match cli.command {
        TemplateCommands::Create(args) => run_create(args),
    }
}
