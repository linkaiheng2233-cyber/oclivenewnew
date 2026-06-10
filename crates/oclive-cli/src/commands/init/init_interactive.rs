//! Interactive and quick init flows.

use super::init_config::{
    apply_backend_cli_overrides, apply_cargo_metadata_cli, apply_template_layer,
    ensure_cargo_license_default, preset_config, quick_project_config, apply_monolith_options,
    pick_role_pack_kind, ProjectConfig, ProjectType, ProjectTypeArg, RolePackKind,
};
use super::InitArgs;
use crate::generator;
use anyhow::{Context, Result};
use std::path::PathBuf;

pub(crate) fn run_quick_init(args: &InitArgs) -> Result<()> {
    let mut project_name = args.project_name.clone();
    let mut output = args.output.clone();
    if !args.non_interactive {
        project_name = dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Project name (quick mode)")
            .default(project_name)
            .interact_text()
            .context("quick project name")?;
        let out_default = output.display().to_string();
        let out_str: String =
            dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt("Output directory")
                .default(out_default)
                .interact_text()
                .context("quick output")?;
        output = PathBuf::from(out_str);
    }
    let mut cfg = quick_project_config(&project_name);
    apply_backend_cli_overrides(&mut cfg, args);
    apply_cargo_metadata_cli(&mut cfg, args);
    ensure_cargo_license_default(&mut cfg);
    if !args.quiet {
        println!("—— Quick mode (--quick) ——");
        println!("preset=full, Monolith=off, no roles/, no --kernel-source");
        cfg.print_summary();
    }
    generator::write_project(&cfg, &output)?;
    if !args.quiet {
        println!("Generated: {}", output.display());
        println!("Suggested next: cargo run -p oclive-cli -- doctor");
        println!(
            "  then cd {} && cargo build && cargo run --release",
            output.display()
        );
    }
    Ok(())
}
/// Resolve init configuration without writing files to disk.
///
/// # Errors
///
/// Returns an error when interactive input, kernel-source validation, or TUI selection fails.
pub fn build_init_config(
    args: &InitArgs,
    skip_interactive_confirm: bool,
) -> Result<ProjectConfig> {
    if args.quick {
        let mut cfg = quick_project_config(&args.project_name);
        apply_backend_cli_overrides(&mut cfg, args);
        apply_cargo_metadata_cli(&mut cfg, args);
        ensure_cargo_license_default(&mut cfg);
        return Ok(cfg);
    }

    let mut cfg = if args.non_interactive || args.dry_run || args.check || skip_interactive_confirm
    {
        let preset = args
            .preset
            .as_deref()
            .unwrap_or("minimal")
            .to_ascii_lowercase();
        let mut c = preset_config(&args.project_name, &preset);
        apply_backend_cli_overrides(&mut c, args);
        if let Some(t) = args.project_type {
            c.project_type = match t {
                ProjectTypeArg::KernelServer => ProjectType::KernelServer,
                ProjectTypeArg::Library => ProjectType::Library,
            };
        }
        c
    } else {
        let mut c = crate::interactive::run_interactive(args)?;
        apply_backend_cli_overrides(&mut c, args);
        if let Some(t) = args.project_type {
            c.project_type = match t {
                ProjectTypeArg::KernelServer => ProjectType::KernelServer,
                ProjectTypeArg::Library => ProjectType::Library,
            };
        }
        if !args.project_name.is_empty() && args.project_name != "my_oclive_kernel" {
            c.project_name = args.project_name.clone();
        }
        c
    };

    apply_template_layer(args, &mut cfg);
    apply_monolith_options(args, &mut cfg);
    if cfg.monolith_enabled {
        cfg.monolith_preset = args.monolith_preset.or(args.monolith_bench_preset);
    }
    if args.monolith_bench_preset.is_some() && cfg.project_type == ProjectType::KernelServer {
        if !cfg.monolith_enabled {
            cfg.monolith_enabled = true;
        }
        cfg.monolith_preset = cfg.monolith_preset.or(args.monolith_bench_preset);
        cfg.run_monolith_bench_after_init = cfg.monolith_enabled;
    }
    cfg.factory_template = args.template;
    cfg.with_example_plugin = args.with_example_plugin;
    cfg.role_pack_kind = pick_role_pack_kind(args);
    if args.skip_role_pack {
        cfg.skip_role_pack = true;
        cfg.role_pack_kind = RolePackKind::None;
    }
    if let Some(ref ks) = args.kernel_source {
        let canonical = ks
            .canonicalize()
            .with_context(|| format!("kernel-source: {}", ks.display()))?;
        generator::validate_kernel_source(&canonical)?;
        cfg.kernel_source = Some(canonical);
    }
    apply_cargo_metadata_cli(&mut cfg, args);
    ensure_cargo_license_default(&mut cfg);
    cfg.pipeline = args.pipeline;
    cfg.dual_core_enabled = args.dual_core;
    if !args.weld_modules.is_empty() {
        cfg.custom_weld_modules = Some(args.weld_modules.clone());
        cfg.monolith_enabled = true;
    }
    if args.tui
        && !skip_interactive_confirm
        && cfg.monolith_enabled
        && cfg.custom_weld_modules.is_none()
        && matches!(cfg.project_type, ProjectType::KernelServer)
    {
        if let Some(w) = crate::init_tui::pick_weld_modules_tui()? {
            cfg.custom_weld_modules = Some(w);
        }
    }
    Ok(cfg)
}
