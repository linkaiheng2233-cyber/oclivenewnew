//! `pack` 子命令：角色包校验、创建、打包（分发向）。

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use oclive_validation::{
    migrate_role_pack_dir_to_blueprint_v2, validate_role_pack_directory_with_profile,
    RolePackValidationProfile, CURRENT_SETTINGS_SCHEMA_VERSION, PIPELINE_BLUEPRINT_FILENAME,
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
    /// Validate role pack directory (default: pipeline.ocblueprint v2; --profile legacy: manifest+settings)
    Validate(PackValidateArgs),
    /// Generate a minimal valid role pack directory
    Create(PackCreateArgs),
    /// Pack role pack directory into `.oclivepack` (ZIP; top-level folder is role id)
    Publish(PackPublishArgs),
    /// Migrate manifest.json + settings.json → pipeline.ocblueprint v2
    MigrateToBlueprint(MigrateToBlueprintArgs),
}

#[derive(Parser, Debug)]
pub struct PackValidateArgs {
    /// Role pack root (contains manifest.json)
    pub path: PathBuf,
    /// Host semver for `manifest.min_runtime_version` (default: this CLI `CARGO_PKG_VERSION`)
    #[arg(long)]
    pub host_version: Option<String>,
    /// Profile: `default` (blueprint v2/v3) | `legacy` | `creator` | `robot-soul` (see ROLE_PACK_SPEC)
    #[arg(long, default_value = "default")]
    pub profile: String,
}

#[derive(Parser, Debug)]
pub struct PackCreateArgs {
    /// Output dir: creates `roles/<id>/` under it by default; with `--flat`, output is the role root
    #[arg(short = 'o', long)]
    pub output: PathBuf,
    /// Role id (written to manifest; with `--flat` should match folder name)
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
    /// Use `output` as role pack root directly (no `roles/<id>/` prefix)
    #[arg(long, default_value_t = false)]
    pub flat: bool,
    /// Output v2 blueprint pack (`pipeline.ocblueprint` only) instead of manifest/settings
    #[arg(long, default_value_t = false)]
    pub format_blueprint_v2: bool,
}

#[derive(Parser, Debug)]
pub struct MigrateToBlueprintArgs {
    /// Role pack root (contains manifest.json)
    pub path: PathBuf,
    /// Remove manifest.json and settings.json after writing blueprint
    #[arg(long, default_value_t = true)]
    pub remove_legacy: bool,
}

#[derive(Parser, Debug)]
pub struct PackPublishArgs {
    /// Role pack root (manifest.json `id` becomes ZIP top-level folder name)
    pub path: PathBuf,
    /// Output path (default: `<cwd>/<id>-<version>.oclivepack`)
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,
}

pub fn run_pack(args: PackArgs) -> Result<()> {
    match args.command {
        PackCommands::Validate(a) => run_validate(a),
        PackCommands::Create(a) => run_create(a),
        PackCommands::Publish(a) => run_publish(a),
        PackCommands::MigrateToBlueprint(a) => run_migrate_to_blueprint(a),
    }
}

fn run_validate(args: PackValidateArgs) -> Result<()> {
    let role_dir = args.path.canonicalize().context("resolve role path")?;
    let host = args
        .host_version
        .as_deref()
        .unwrap_or(env!("CARGO_PKG_VERSION"));
    let profile: RolePackValidationProfile = args
        .profile
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;
    match validate_role_pack_directory_with_profile(
        &role_dir,
        host,
        CURRENT_SETTINGS_SCHEMA_VERSION,
        profile,
    ) {
        Ok(()) => {
            println!("✓ Role pack validation passed");
            Ok(())
        }
        Err(errs) => {
            eprintln!("✗ Role pack validation failed:");
            for e in errs {
                eprintln!("  - {}", e);
            }
            anyhow::bail!("validation failed");
        }
    }
}

fn run_migrate_to_blueprint(args: MigrateToBlueprintArgs) -> Result<()> {
    let role_dir = args.path.canonicalize().context("resolve role path")?;
    migrate_role_pack_dir_to_blueprint_v2(&role_dir, args.remove_legacy)
        .map_err(|errs| anyhow::anyhow!("migrate failed:\n{}", errs.join("\n")))?;
    println!(
        "Migrated to {} ({})",
        role_dir.join(PIPELINE_BLUEPRINT_FILENAME).display(),
        if args.remove_legacy {
            "legacy files removed"
        } else {
            "legacy files kept"
        }
    );
    Ok(())
}

fn run_create(args: PackCreateArgs) -> Result<()> {
    let id = args.id.trim();
    if id.is_empty() {
        anyhow::bail!("--id cannot be empty");
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
                "_comment_display_name": "Optional; omit display name when same as id",
                "initial_favorability": 50.0,
                "favor_multiplier": 1.0
            }
        },
        "default_relation": "friend"
    });
    if args.format_blueprint_v2 {
        let bp = serde_json::json!({
            "schema_version": 2,
            "meta": {
                "id": id,
                "name": args.name,
                "version": args.version,
                "author": args.author,
                "description": args.description,
                "personality": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
                "relations": {
                    "friend": { "initial_favorability": 50.0, "favor_multiplier": 1.0 }
                },
                "default_relation": "friend",
                "interaction_mode": "immersive"
            },
            "slot_registry": {
                "memory": { "type": "memory", "label": "Memory", "backend": "builtin", "position": 0 },
                "emotion": { "type": "emotion", "label": "Emotion", "backend": "builtin", "position": 0 },
                "complex_emotion": { "type": "complex_emotion", "label": "Complex emotion", "backend": "builtin", "position": 1 },
                "event": { "type": "event", "label": "Event", "backend": "builtin", "position": 0 },
                "prompt": { "type": "prompt", "label": "Prompt", "backend": "builtin", "position": 0 },
                "llm": { "type": "llm", "label": "LLM", "backend": "ollama", "position": 0 },
                "agent": { "type": "agent", "label": "Agent", "backend": "builtin", "position": 0 }
            }
        });
        fs::write(
            root.join(PIPELINE_BLUEPRINT_FILENAME),
            serde_json::to_string_pretty(&bp).context("serialize blueprint")?,
        )
        .context("write pipeline.ocblueprint")?;
        println!("Role pack directory created (v2): {}", root.display());
        return Ok(());
    }

    fs::write(
        root.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).context("serialize manifest")?,
    )
    .context("write manifest.json")?;

    let settings = json!({
        "_comment": "Standard JSON has no // comments; use _-prefixed keys for notes",
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

    println!("Role pack directory created: {}", root.display());
    Ok(())
}

fn run_publish(args: PackPublishArgs) -> Result<()> {
    let role_dir = args.path.canonicalize().context("resolve role path")?;
    let blueprint_path = role_dir.join(PIPELINE_BLUEPRINT_FILENAME);
    let manifest_path = role_dir.join("manifest.json");
    let (id, version) = if blueprint_path.is_file() {
        let raw = fs::read_to_string(&blueprint_path).context("read blueprint")?;
        let v: serde_json::Value = serde_json::from_str(&raw).context("parse blueprint")?;
        let id = v
            .get("meta")
            .and_then(|m| m.get("id"))
            .and_then(|x| x.as_str())
            .context("pipeline.ocblueprint meta.id missing")?;
        let version = v
            .get("meta")
            .and_then(|m| m.get("version"))
            .and_then(|x| x.as_str())
            .unwrap_or("0.0.0");
        (id.to_string(), version.to_string())
    } else {
        let manifest_raw = fs::read_to_string(&manifest_path).context("read manifest")?;
        let v: serde_json::Value = serde_json::from_str(&manifest_raw).context("parse manifest")?;
        let id = v
            .get("id")
            .and_then(|x| x.as_str())
            .context("manifest.json missing id")?
            .to_string();
        let version = v
            .get("version")
            .and_then(|x| x.as_str())
            .unwrap_or("0.0.0")
            .to_string();
        (id, version)
    };
    let out_path = args.output.clone().unwrap_or_else(|| {
        PathBuf::from(format!("{}-{}.oclivepack", id.replace('/', "_"), version))
    });
    let id = id.as_str();
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let file =
        fs::File::create(&out_path).with_context(|| format!("create {}", out_path.display()))?;
    let mut zip = ZipWriter::new(file);
    walk_pack(&mut zip, &role_dir, &role_dir, id)?;
    zip.finish().context("zip finish")?;
    println!("Wrote {}", out_path.display());
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
            zip.start_file(zip_path, file_opts)
                .context("zip start_file")?;
            std::io::copy(&mut f, zip).context("zip write")?;
        }
    }
    Ok(())
}
