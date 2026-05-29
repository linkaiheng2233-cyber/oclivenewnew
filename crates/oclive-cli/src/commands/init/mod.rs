//! `oclive init`: scaffold kernel projects.
//!
//! 非交互模式下 **`--preset`** 决定基线；可选 **`--backend-*`** 逐项覆盖。
//! 预设与 `plugin_backends` 矩阵见 **`init --help` 末尾**（与生成项目根目录 **`CONFIG_REFERENCE.md`** 一致）。

mod init_config;
mod init_interactive;
mod init_smart;

pub use init_config::*;
pub use init_interactive::resolve_init_config;

use init_interactive::run_quick_init;
use init_smart::apply_smart_hints;

use crate::generator;
use crate::pipeline::PipelineArg;
use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

pub use crate::cli_english_init::PRESET_MATRIX_HELP;

#[derive(Parser, Debug, Clone)]
pub struct InitArgs {
    /// Output directory (created and populated with the new project)
    #[arg(short = 'o', long, default_value = "generated-kernel")]
    pub output: PathBuf,

    /// Quick mode: full preset, no Monolith, no role pack; interactive prompts only project name and output dir
    #[arg(short = 'q', long)]
    pub quick: bool,

    /// Non-interactive mode (use with --preset)
    #[arg(long)]
    pub non_interactive: bool,

    /// Skip config summary and completion hints (for scripts / tests)
    #[arg(long)]
    pub quiet: bool,

    /// Print kernel factory template matrix and exit (do not write output directory)
    #[arg(long)]
    pub list_templates: bool,

    /// Kernel factory template: robot-soul | robot-gateway | dialogue-only | headless-api | library-embed
    #[arg(long, value_enum)]
    pub template: Option<InitTemplateArg>,

    /// Preset: minimal | full | mixed
    #[arg(long)]
    pub preset: Option<String>,

    #[arg(long, default_value = "my_oclive_kernel")]
    pub project_name: String,

    /// Project type (required in non-interactive mode; optional in interactive mode)
    #[arg(long, value_enum)]
    pub project_type: Option<ProjectTypeArg>,

    /// Override memory slot (defaults to `--preset`)
    #[arg(long, value_enum)]
    pub backend_memory: Option<BackendImpl>,

    #[arg(long, value_enum)]
    pub backend_emotion: Option<BackendImpl>,

    #[arg(long, value_enum)]
    pub backend_event: Option<BackendImpl>,

    #[arg(long, value_enum)]
    pub backend_prompt: Option<BackendImpl>,

    #[arg(long, value_enum)]
    pub backend_llm: Option<BackendImpl>,

    #[arg(long, value_enum)]
    pub backend_agent: Option<BackendImpl>,

    #[arg(long, value_enum)]
    pub backend_complex_emotion: Option<BackendImpl>,

    /// Non-interactive: enable Monolith (kernel_server only; writes monolith.toml, vendor/, welded sources, dual [[bin]])
    #[arg(long)]
    pub monolith: bool,

    /// Generate example role pack with schema v3 blueprint and runtime_config.dual_core.enabled (kernel_server + role pack)
    #[arg(long)]
    pub dual_core: bool,

    /// Monolith weld tier (written to monolith.toml weld_modules when Monolith is enabled)
    #[arg(long, value_enum)]
    pub monolith_preset: Option<MonolithPresetArg>,

    /// After generation, auto-run Monolith vs standard bench (5 runs); also sets weld tier (same as --monolith-preset)
    #[arg(long, value_enum)]
    pub monolith_bench_preset: Option<MonolithPresetArg>,

    /// Non-interactive: do not write example role packs under `roles/`
    #[arg(long)]
    pub skip_role_pack: bool,

    /// Example role pack: robot-soul-minimal | default (defaults from --template or historical default)
    #[arg(long, value_enum)]
    pub with_role_pack: Option<RolePackKindArg>,

    /// Include llamacpp directory plugin example at plugins/com.oclive.example.llamacpp_llm/
    #[arg(long)]
    pub with_example_plugin: bool,

    /// Path to oclivenewnew repo root: generated project uses path deps on oclivenewnew-tauri / oclive_kernel_runtime
    #[arg(long)]
    pub kernel_source: Option<PathBuf>,

    /// Write `[package].authors` in generated Cargo.toml
    #[arg(long)]
    pub author: Option<String>,

    /// Write `[package].license` (default MIT)
    #[arg(long)]
    pub license: Option<String>,

    /// Write `[package].description` (omit if empty)
    #[arg(long)]
    pub description: Option<String>,

    /// Download `.oclive-template.tar.gz` from URL and extract to output directory
    #[arg(long)]
    pub template_url: Option<String>,

    /// Pick kernel factory template via terminal TUI (falls back to dialoguer if unavailable)
    #[arg(long)]
    pub tui: bool,

    /// Probe environment (Ollama / GPU / memory) and print recommended --preset / --monolith flags
    #[arg(long)]
    pub smart: bool,

    /// Skip automatic environment recommendations in interactive init
    #[arg(long)]
    pub no_smart: bool,

    /// Custom pipeline order: default | emotion-first | memory-last
    #[arg(long, value_enum, default_value_t = PipelineArg::Default)]
    pub pipeline: PipelineArg,

    /// TUI custom Monolith weld slots (comma-separated; overrides monolith-preset)
    #[arg(long, value_delimiter = ',')]
    pub weld_modules: Vec<String>,

    /// Analyze an existing project and print a full non-interactive `oclive init` reproduction command
    #[arg(long)]
    pub from_existing: Option<PathBuf>,

    /// With `--from-existing`: write `.oclive-share.toml` in the source project
    #[arg(long)]
    pub share: bool,

    /// Machine-readable output for `--from-existing`
    #[arg(long)]
    pub json: bool,

    /// Print planned directory tree without writing files
    #[arg(long)]
    pub dry_run: bool,

    /// Pre-flight checks for template and environment (no generation)
    #[arg(long)]
    pub check: bool,

    /// Non-interactive: role pack `config.json` → `chat_storage.location` (`global` | `role_pack`)
    #[arg(long, value_parser = parse_chat_storage_location_arg)]
    pub chat_storage_location: Option<String>,
}

fn parse_chat_storage_location_arg(raw: &str) -> Result<String, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "global" => Ok("global".to_string()),
        "role_pack" => Ok("role_pack".to_string()),
        other => Err(format!(
            "chat-storage-location must be global or role_pack, got {other:?}"
        )),
    }
}

/// Run the init subcommand (scaffold a kernel project).
///
/// # Errors
///
/// Returns an error when validation, user cancel, or project generation fails.
pub fn run(args: InitArgs) -> Result<()> {
    if let Some(ref existing) = args.from_existing {
        return crate::init_from_existing::run_from_existing(existing, &args);
    }

    if apply_smart_hints(&args)? {
        return Ok(());
    }

    if args.check {
        return crate::init_check::run_precheck(&args);
    }

    if args.list_templates {
        crate::template_catalog::print_templates_table();
        return Ok(());
    }

    if let Some(ref url) = args.template_url {
        if args.output.exists() {
            anyhow::bail!("Output directory already exists: {}", args.output.display());
        }
        return crate::publish_cmd::init_from_template_url(url, &args.output);
    }

    if args.quick {
        return run_quick_init(&args);
    }

    let cfg = resolve_init_config(&args, false)?;

    if args.dry_run {
        return crate::init_plan::print_dry_run(&cfg, &args);
    }

    if !args.non_interactive {
        cfg.print_summary();
        if !dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Generate project in this directory?")
            .default(true)
            .interact()
            .context("confirm")?
        {
            println!("Cancelled.");
            return Ok(());
        }
    } else if !args.quiet {
        cfg.print_summary();
    }

    generator::write_project(&cfg, &args.output)?;
    if let Err(e) = crate::registry::register_after_init(&cfg, &args.output) {
        eprintln!("⚠ Failed to register in local registry: {e}");
    }
    if cfg.run_monolith_bench_after_init {
        crate::init_bench::try_post_init_monolith_bench(&args.output);
    }
    if !args.quiet {
        if cfg.monolith_enabled && matches!(cfg.project_type, ProjectType::KernelServer) {
            let slug = crate::generator::project_slug(&cfg);
            println!("✓ Project created!");
            println!("  Standard binary: target/release/{slug}");
            println!("  Monolith binary: target/release/{slug}-monolith");
            println!("  Config file: monolith.toml");
            println!(
                "  After changing weld plan: run oclive build at project root (or cargo run -p oclive-cli -- build -o {})",
                args.output.display()
            );
            println!(
                "  Performance compare: oclive bench --release -o {}",
                args.output.display()
            );
            println!("  Details: see creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md");
            println!("Output directory: {}", args.output.display());
        } else {
            println!(
                "Generated project: {} (run cargo build in that directory)",
                args.output.display()
            );
        }
        println!("Environment check: cargo run -p oclive-cli -- doctor");
    }
    Ok(())
}