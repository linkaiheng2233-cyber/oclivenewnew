//! `pack` 子命令：角色包校验、创建、打包（分发向）。

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use oclive_validation::{
    validate_role_pack_directory, CURRENT_SETTINGS_SCHEMA_VERSION,
};
use serde_json::json;
use zip::write::FileOptions;
use zip::ZipWriter;

#[derive(Parser, Debug)]
pub struct PackArgs {
    #[command(subcommand)]
    pub command: PackCommands,
}

#[derive(Subcommand, Debug)]
pub enum PackCommands {
    /// 校验角色包目录（manifest + settings 合并后与宿主加载前磁盘校验一致）
    Validate(PackValidateArgs),
    /// 生成可校验的最小角色包目录
    Create(PackCreateArgs),
    /// 将角色包目录打成 `.oclivepack`（ZIP 容器，根目录为角色 id）
    Publish(PackPublishArgs),
}

#[derive(Parser, Debug)]
pub struct PackValidateArgs {
    /// 角色包根目录（内含 manifest.json）
    pub path: PathBuf,
    /// 用于比较 `manifest.min_runtime_version` 的宿主 semver（默认：本 CLI 的 `CARGO_PKG_VERSION`）
    #[arg(long)]
    pub host_version: Option<String>,
}

#[derive(Parser, Debug)]
pub struct PackCreateArgs {
    /// 输出目录：默认在其下创建 `roles/<id>/`；若指定 `--flat` 则本目录即为角色根（须已存在或将被创建为包根）
    #[arg(short = 'o', long)]
    pub output: PathBuf,
    /// 角色 id（写入 manifest；`--flat` 时也应与磁盘文件夹名一致）
    #[arg(long)]
    pub id: String,
    #[arg(long, default_value = "My Character")]
    pub name: String,
    #[arg(long, default_value = "0.1.0")]
    pub version: String,
    #[arg(long, default_value = "author")]
    pub author: String,
    #[arg(long, default_value = "")]
    pub description: String,
    /// `output` 直接作为角色包根目录（不创建 `roles/<id>/` 前缀）
    #[arg(long, default_value_t = false)]
    pub flat: bool,
}

#[derive(Parser, Debug)]
pub struct PackPublishArgs {
    /// 角色包根目录（内含 manifest.json；其中的 `id` 决定 ZIP 内顶层目录名）
    pub path: PathBuf,
    /// 输出文件路径（默认：`<cwd>/<id>-<version>.oclivepack`）
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,
}

pub fn run_pack(args: PackArgs) -> Result<()> {
    match args.command {
        PackCommands::Validate(a) => run_validate(a),
        PackCommands::Create(a) => run_create(a),
        PackCommands::Publish(a) => run_publish(a),
    }
}

fn run_validate(args: PackValidateArgs) -> Result<()> {
    let role_dir = args.path.canonicalize().context("resolve role path")?;
    let host = args
        .host_version
        .as_deref()
        .unwrap_or(env!("CARGO_PKG_VERSION"));
    match validate_role_pack_directory(&role_dir, host, CURRENT_SETTINGS_SCHEMA_VERSION) {
        Ok(()) => {
            println!("✓ 角色包验证通过");
            Ok(())
        }
        Err(errs) => {
            eprintln!("✗ 角色包验证失败：");
            for e in errs {
                eprintln!("  - {}", e);
            }
            anyhow::bail!("validation failed");
        }
    }
}

fn run_create(args: PackCreateArgs) -> Result<()> {
    let id = args.id.trim();
    if id.is_empty() {
        anyhow::bail!("--id 不能为空");
    }
    let root: PathBuf = if args.flat {
        args.output.clone()
    } else {
        args.output.join("roles").join(id)
    };
    fs::create_dir_all(root.join("scenes").join("default"))
        .with_context(|| format!("create scenes/default under {}", root.display()))?;

    let manifest = json!({
        "id": id,
        "name": args.name,
        "version": args.version,
        "author": args.author,
        "description": args.description,
        "default_personality": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
        "scenes": ["default"],
        "user_relations": {
            "friend": {
                "_comment_display_name": "可选；与 id 相同时可省略展示名",
                "initial_favorability": 50.0,
                "favor_multiplier": 1.0
            }
        },
        "default_relation": "friend"
    });
    fs::write(
        root.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).context("serialize manifest")?,
    )
    .context("write manifest.json")?;

    let settings = json!({
        "_comment": "标准 JSON 无 // 注释；说明请用 _ 前缀键",
        "schema_version": 1,
        "plugin_backends": {
            "memory": "builtin",
            "emotion": "builtin",
            "event": "builtin",
            "prompt": "builtin",
            "llm": "ollama",
            "agent": "builtin"
        }
    });
    fs::write(
        root.join("settings.json"),
        serde_json::to_string_pretty(&settings).context("serialize settings")?,
    )
    .context("write settings.json")?;

    fs::write(
        root.join("core_personality.txt"),
        "# Core personality (UTF-8 text). Optional alongside manifest default_personality.\n",
    )
    .context("write core_personality.txt")?;

    let scene = json!({
        "name": "Default",
        "time_windows": [],
        "keywords": [],
        "events": []
    });
    fs::write(
        root.join("scenes").join("default").join("scene.json"),
        serde_json::to_string_pretty(&scene).context("scene.json")?,
    )
    .context("write scene.json")?;

    println!("已生成角色包目录：{}", root.display());
    Ok(())
}

fn run_publish(args: PackPublishArgs) -> Result<()> {
    let role_dir = args.path.canonicalize().context("resolve role path")?;
    let manifest_raw = fs::read_to_string(role_dir.join("manifest.json")).context("read manifest")?;
    let v: serde_json::Value = serde_json::from_str(&manifest_raw).context("parse manifest")?;
    let id = v
        .get("id")
        .and_then(|x| x.as_str())
        .context("manifest.json 缺少 id")?;
    let version = v
        .get("version")
        .and_then(|x| x.as_str())
        .unwrap_or("0.0.0");
    let out_path = args.output.clone().unwrap_or_else(|| {
        PathBuf::from(format!("{}-{}.oclivepack", id.replace('/', "_"), version))
    });
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let file = fs::File::create(&out_path).with_context(|| format!("create {}", out_path.display()))?;
    let mut zip = ZipWriter::new(file);
    walk_pack(&mut zip, &role_dir, &role_dir, id)?;
    zip.finish().context("zip finish")?;
    println!("已写入 {}", out_path.display());
    Ok(())
}

fn walk_pack(
    zip: &mut ZipWriter<fs::File>,
    base: &Path,
    current: &Path,
    role_id: &str,
) -> Result<()> {
    for entry in fs::read_dir(current).with_context(|| format!("read_dir {}", current.display()))? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let rel = path.strip_prefix(base).context("strip_prefix")?;
        let zip_path = format!("{}/{}", role_id, rel.to_string_lossy().replace('\\', "/"));
        if path.is_dir() {
            walk_pack(zip, base, &path, role_id)?;
        } else {
            let mut f = fs::File::open(&path)?;
            let file_opts = FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o644);
            zip.start_file(zip_path, file_opts).context("zip start_file")?;
            std::io::copy(&mut f, zip).context("zip write")?;
        }
    }
    Ok(())
}
