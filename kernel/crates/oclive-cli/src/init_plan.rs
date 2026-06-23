//! Dry-run project tree preview for `init --dry-run`.

use anyhow::Result;
use serde::Serialize;

use crate::init::{InitArgs, InitTemplateArg, ProjectConfig, ProjectType, RolePackKind};

#[derive(Serialize)]
pub struct DryRunPlan {
    pub schema_version: u32,
    pub project_name: String,
    pub output_dir: String,
    pub template: Option<String>,
    pub project_type: String,
    pub preset: String,
    pub monolith_enabled: bool,
    pub monolith_preset: Option<String>,
    pub kernel_linked: bool,
    pub tree: Vec<String>,
}

pub fn print_dry_run(cfg: &ProjectConfig, args: &InitArgs) -> Result<()> {
    let tree = build_tree_lines(cfg, &args.output);
    let plan = DryRunPlan {
        schema_version: 1,
        project_name: cfg.project_name.clone(),
        output_dir: args.output.display().to_string(),
        template: cfg
            .factory_template
            .map(|t| format!("{t:?}"))
            .map(|s| s.replace("InitTemplateArg::", "").to_lowercase()),
        project_type: match cfg.project_type {
            ProjectType::KernelServer => "kernel_server".into(),
            ProjectType::Library => "library".into(),
        },
        preset: args.preset.clone().unwrap_or_else(|| "minimal".into()),
        monolith_enabled: cfg.monolith_enabled,
        monolith_preset: cfg.monolith_preset.map(|p| format!("{p:?}").to_lowercase()),
        kernel_linked: cfg.kernel_source.is_some(),
        tree: tree.clone(),
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(());
    }

    println!("oclive init --dry-run (no files written)\n");
    println!("Project: {}", cfg.project_name);
    if let Some(t) = cfg.factory_template {
        println!("Template: {t:?}");
    }
    println!(
        "Type: {}",
        match cfg.project_type {
            ProjectType::KernelServer => "kernel_server",
            ProjectType::Library => "library",
        }
    );
    println!(
        "Monolith: {}",
        if cfg.monolith_enabled {
            format!(
                "enabled ({})",
                cfg.monolith_preset
                    .map(|p| format!("{p:?}"))
                    .unwrap_or_else(|| "latency".into())
            )
        } else {
            "disabled".into()
        }
    );
    if cfg.kernel_source.is_some() {
        println!("Kernel: linked (--kernel-source)");
    }
    println!("\nDirectory structure:");
    for line in &tree {
        println!("{line}");
    }
    Ok(())
}

pub fn build_tree_lines(cfg: &ProjectConfig, output: &std::path::Path) -> Vec<String> {
    let name = output
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project");
    let mut lines = vec![format!("{name}/")];
    lines.push("  Cargo.toml".into());
    lines.push("  README.md".into());
    lines.push("  CONFIG_REFERENCE.md".into());
    lines.push("  src/".into());
    match cfg.project_type {
        ProjectType::KernelServer => {
            lines.push("    main.rs".into());
            if cfg.monolith_enabled {
                lines.push("    main_monolith.rs".into());
                lines.push("    process_message_monolith.rs".into());
                lines.push("  monolith.toml".into());
                lines.push("  vendor/oclive_monolith_builtin/".into());
            }
        }
        ProjectType::Library => {
            lines.push("    lib.rs".into());
        }
    }
    lines.push("  docs/".into());
    lines.push("    BLUEPRINT_V2_POINTER.md".into());
    if cfg.pipeline != crate::pipeline::PipelineArg::Default {
        lines.push("    PIPELINE_CUSTOM.md".into());
        lines.push("  src/oclive_pipeline_order.rs".into());
    }
    lines.push("  plugins/".into());
    lines.push("    README.md".into());
    if cfg.with_example_plugin {
        lines.push("    com.oclive.example.llamacpp_llm/".into());
    }
    if cfg.factory_template == Some(InitTemplateArg::RobotGateway) {
        lines.push("  mcp_servers/".into());
    }
    if !cfg.skip_role_pack && cfg.role_pack_kind != RolePackKind::None {
        lines.push("  roles/".into());
        match cfg.role_pack_kind {
            RolePackKind::RobotSoulMinimal => {
                lines.push("    robot-soul-minimal/".into());
            }
            RolePackKind::DefaultExample => {
                lines.push("    default/".into());
            }
            RolePackKind::None => {}
        }
    }
    lines
}
