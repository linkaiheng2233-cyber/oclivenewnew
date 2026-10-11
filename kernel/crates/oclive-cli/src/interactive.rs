//! Interactive collection of [`crate::init::ProjectConfig`] (skips prompts already set via CLI).

use crate::init::{
    BackendImpl, BackendSlots, BaseExampleArg, ChatStorageBackend, FeatureSelection, InitArgs,
    InitTemplateArg, PluginSelection, ProjectConfig, ProjectType, ProjectTypeArg, RolePackKind,
};
use crate::template_catalog::{project_config_from_template, CATALOG};
use anyhow::{anyhow, Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};

const SLOT_LABELS: &[&str] = &[
    "memory",
    "emotion",
    "event",
    "prompt",
    "llm",
    "agent",
    "complex_emotion",
];

fn pick_impl(slot: &str) -> Result<BackendImpl> {
    let (items, labels): (&[&str], &[&str]) = if slot == "llm" {
        (
            &["ollama", "remote", "directory", "none"],
            &[
                "ollama (host default local LLM; requires Ollama on this machine)",
                "remote (HTTP sidecar; needs OCLIVE_REMOTE_LLM_URL)",
                "directory (directory plugin subprocess)",
                "none (disable main LLM chain; experimental / placeholder only)",
            ],
        )
    } else {
        (
            &["builtin", "remote", "directory", "none"],
            &[
                "builtin (in-process default implementation)",
                "remote (HTTP JSON-RPC; needs OCLIVE_REMOTE_PLUGIN_URL, etc.)",
                "directory (directory plugin; blueprint plugin(s), or legacy directory_plugins)",
                "none (disable this subsystem; some slots may affect the main chat path)",
            ],
        )
    };
    let i = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Which implementation for {slot}?"))
        .items(labels)
        .default(0)
        .interact()
        .context("select backend impl")?;
    let raw = items.get(i).copied().unwrap_or("builtin");
    Ok(match raw {
        "ollama" => BackendImpl::Ollama,
        "remote" => BackendImpl::Remote,
        "directory" => BackendImpl::Directory,
        "none" => BackendImpl::None,
        _ => BackendImpl::Builtin,
    })
}

fn validate_slot_choice(slot: &str, b: BackendImpl) -> Result<()> {
    if slot != "llm" && b == BackendImpl::Ollama {
        return Err(anyhow!(
            "Slot {slot} cannot use ollama (valid only for the llm slot)."
        ));
    }
    Ok(())
}

fn pick_factory_template() -> Result<Option<InitTemplateArg>> {
    let mut labels: Vec<String> =
        vec!["No template — configure preset / modules 1-6 manually".into()];
    for e in CATALOG {
        labels.push(format!(
            "{} — {} (preset={}, Monolith={})",
            e.id, e.scene, e.preset, e.monolith
        ));
    }
    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose scenario template (kernel factory recipe)")
        .items(labels)
        .default(0)
        .interact()
        .context("template select")?;
    if idx == 0 {
        return Ok(None);
    }
    let entry = CATALOG.get(idx - 1).context("template index")?;
    crate::template_catalog::template_from_id(entry.id)
        .ok_or_else(|| anyhow!("Unknown template: {}", entry.id))
        .map(Some)
}

fn pick_project_type(default_kernel: bool) -> Result<ProjectType> {
    let default = if default_kernel { 0 } else { 1 };
    let pt_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Project type")
        .items([
            "Headless service (kernel_server)",
            "Embedded library (library)",
        ])
        .default(default)
        .interact()
        .context("project type")?;
    Ok(if pt_idx == 0 {
        ProjectType::KernelServer
    } else {
        ProjectType::Library
    })
}

fn pick_slots_manual() -> Result<BackendSlots> {
    let chosen: Vec<usize> = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Backend slots to enable (at least memory / emotion / prompt / llm)")
        .items(SLOT_LABELS)
        .defaults(&[true, true, true, true, true, true, true])
        .interact()
        .context("backend multiselect")?;
    let required = [0usize, 1, 3, 4];
    for r in required {
        if !chosen.contains(&r) {
            return Err(anyhow!(
                "At least memory, emotion, prompt, and llm slots must be enabled."
            ));
        }
    }
    let mut slots = BackendSlots::all(BackendImpl::None);
    for &idx in &chosen {
        let slot = SLOT_LABELS[idx];
        let b = pick_impl(slot)?;
        validate_slot_choice(slot, b)?;
        match idx {
            0 => slots.memory = b,
            1 => slots.emotion = b,
            2 => slots.event = b,
            3 => slots.prompt = b,
            4 => slots.llm = b,
            5 => slots.agent = b,
            6 => slots.complex_emotion = b,
            _ => {}
        }
    }
    Ok(slots)
}

fn pick_plugins() -> Result<PluginSelection> {
    let plug_idx: Vec<usize> = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Optional plugins")
        .items([
            "directory-plugins (directory plugin placeholder docs)",
            "kernel-server (README: how to wire oclive_kernel_server)",
            "oocp (README: OOCP black-box test entry)",
        ])
        .interact()
        .context("plugins")?;
    Ok(PluginSelection {
        directory_plugins: plug_idx.contains(&0),
        kernel_server: plug_idx.contains(&1),
        oocp: plug_idx.contains(&2),
    })
}

fn pick_monolith_for_kernel(template_default_on: bool) -> Result<bool> {
    let dev_opt = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Enable developer compile options (Monolith)?")
        .default(template_default_on)
        .interact()
        .context("developer compile option")?;
    if !dev_opt {
        return Ok(false);
    }
    let mode_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Compile mode")
        .items([
            "Standard (loosely coupled, modules stay swappable; recommended)",
            "Monolith — weld all seven keys statically (modules 1-6 + complex_emotion)",
            "Monolith — custom weld scope (edit monolith.toml after generate, then run oclive build)",
        ])
        .default(if template_default_on { 1 } else { 0 })
        .interact()
        .context("compile mode")?;
    Ok(mode_idx == 1 || mode_idx == 2)
}

fn pick_cargo_metadata(args: &InitArgs, cfg: &mut ProjectConfig) -> Result<()> {
    if args.author.is_some() || args.license.is_some() || args.description.is_some() {
        return Ok(());
    }
    let default_author =
        crate::init::git_config_user_name().unwrap_or_else(|| "oclive".to_string());
    let author: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Author (written to Cargo.toml authors)")
        .default(default_author)
        .interact_text()
        .context("author")?;
    if !author.trim().is_empty() {
        cfg.cargo_author = Some(author.trim().to_string());
    }
    let licenses = ["Apache-2.0", "MIT", "GPL-3.0", "AGPL-3.0"];
    let lic_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("License (SPDX)")
        .items(licenses)
        .default(0)
        .interact()
        .context("license")?;
    cfg.cargo_license = Some(
        licenses
            .get(lic_idx)
            .copied()
            .unwrap_or("Apache-2.0")
            .to_string(),
    );
    let desc: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Short description (optional; leave empty to omit description)")
        .allow_empty(true)
        .interact_text()
        .context("description")?;
    if !desc.trim().is_empty() {
        cfg.cargo_description = Some(desc.trim().to_string());
    }
    Ok(())
}

fn pick_init_project_name(args: &InitArgs) -> Result<String> {
    if !args.project_name.is_empty() && args.project_name != "my_oclive_kernel" {
        return Ok(args.project_name.clone());
    }
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Project name (Cargo package and directory name)")
        .default("my_oclive_kernel".into())
        .interact_text()
        .context("project name")
}

fn pick_chat_storage_backend() -> Result<ChatStorageBackend> {
    let labels = [
        "Hybrid (recommended): SQLite + JSON mirror — performance and transparent files",
        "File only: JSON files under app data — lightweight, user-manageable",
        "SQLite only: no JSON mirror — highest performance, no mirror files",
    ];
    let i = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Chat history storage backend")
        .items(labels)
        .default(0)
        .interact()
        .context("chat storage backend")?;
    Ok(match i {
        1 => ChatStorageBackend::File,
        2 => ChatStorageBackend::Sqlite,
        _ => ChatStorageBackend::Hybrid,
    })
}

fn pick_chat_storage_location() -> Result<String> {
    let labels = [
        "跟随角色包（推荐）：聊天记录存在角色包目录下的 chats/ 子目录",
        "全局位置：存在 app_data/chats/ 下（与角色包位置独立）",
    ];
    let i = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("聊天记录存储位置")
        .items(labels)
        .default(0)
        .interact()
        .context("chat storage location")?;
    Ok(if i == 1 {
        "global".to_string()
    } else {
        "role_pack".to_string()
    })
}

/// At least memory / emotion / prompt / llm (minimal product chat path).
pub fn run_interactive(args: &InitArgs) -> Result<ProjectConfig> {
    if args.template == Some(InitTemplateArg::KernelBase) {
        let mut foundation_args = args.clone();
        foundation_args.project_name = pick_init_project_name(args)?;
        return crate::kernel_base_scaffold::build_config(&collect_base_options(&foundation_args)?);
    }
    let project_name = pick_init_project_name(args)?;

    let template_choice = if let Some(t) = args.template {
        Some(t)
    } else if args.tui || crate::init_tui::terminal_supports_tui() {
        match crate::init_tui::pick_template_tui(&project_name)? {
            Some(t) => Some(t),
            None => pick_factory_template()?,
        }
    } else {
        pick_factory_template()?
    };
    let manual_config = template_choice.is_none() && args.preset.is_none();
    if template_choice == Some(InitTemplateArg::KernelBase) {
        let mut foundation_args = args.clone();
        foundation_args.template = template_choice;
        foundation_args.project_name = project_name;
        return crate::kernel_base_scaffold::build_config(&collect_base_options(&foundation_args)?);
    }

    let (mut cfg, template_default_monolith) = match (template_choice, args.preset.as_deref()) {
        (Some(t), _) => {
            let c = project_config_from_template(&project_name, t);
            let monolith_default = c.monolith_enabled;
            println!(
                "\nApplied template \"{}\": preset={}, Monolith default={}, role pack={}.",
                CATALOG
                    .iter()
                    .find(|e| crate::template_catalog::template_from_id(e.id) == Some(t))
                    .map(|e| e.id)
                    .unwrap_or("?"),
                crate::init::template_defaults(t).preset,
                if c.monolith_enabled { "on" } else { "off" },
                crate::template_catalog::role_pack_label(c.role_pack_kind)
            );
            println!("CLI flags override template defaults.\n");
            (c, monolith_default)
        }
        (None, Some(p)) => {
            let c = crate::init::preset_config(&project_name, p);
            println!("\nUsing CLI preset={p}; skipping seven-backend multiselect (six slots + complex_emotion facility).\n");
            (c, false)
        }
        (None, None) => {
            let c = ProjectConfig {
                project_name: project_name.clone(),
                project_type: ProjectType::KernelServer,
                backends: BackendSlots::all(BackendImpl::Builtin),
                plugins: PluginSelection {
                    directory_plugins: false,
                    kernel_server: false,
                    oocp: false,
                },
                features: FeatureSelection {
                    use_complex_emotion: false,
                },
                role_pack_kind: RolePackKind::DefaultExample,
                monolith_enabled: false,
                monolith_preset: None,
                skip_role_pack: false,
                with_example_plugin: false,
                factory_template: None,
                base_memory_example: false,
                run_monolith_bench_after_init: false,
                kernel_source: None,
                cargo_author: None,
                cargo_license: None,
                cargo_description: None,
                pipeline: crate::pipeline::PipelineArg::Default,
                custom_weld_modules: None,
                dual_core_enabled: false,
                chat_storage_backend: ChatStorageBackend::default(),
                chat_storage_location: crate::init::default_storage_location(),
            };
            (c, false)
        }
    };

    cfg.project_name = project_name;
    pick_cargo_metadata(args, &mut cfg)?;

    if args.project_type.is_none() {
        if manual_config {
            cfg.project_type = pick_project_type(true)?;
        } else {
            cfg.project_type = pick_project_type(cfg.project_type == ProjectType::KernelServer)?;
        }
    } else if let Some(pt) = args.project_type {
        cfg.project_type = match pt {
            ProjectTypeArg::KernelServer => ProjectType::KernelServer,
            ProjectTypeArg::Library => ProjectType::Library,
        };
    }

    if manual_config {
        cfg.backends = pick_slots_manual()?;
        cfg.features.use_complex_emotion = cfg.backends.complex_emotion != BackendImpl::None;
        cfg.plugins = pick_plugins()?;
    }

    let skip_role_prompt =
        args.skip_role_pack || args.with_role_pack.is_some() || template_choice.is_some();
    if !skip_role_prompt {
        let default_role = cfg.role_pack_kind != RolePackKind::None;
        let example_role = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Generate example role pack (roles/)?")
            .default(default_role)
            .interact()
            .context("example role")?;
        cfg.role_pack_kind = if example_role {
            RolePackKind::DefaultExample
        } else {
            RolePackKind::None
        };
        if cfg.role_pack_kind != RolePackKind::None {
            cfg.chat_storage_backend = pick_chat_storage_backend()?;
            cfg.chat_storage_location = pick_chat_storage_location()?;
        }
    }

    let skip_monolith_prompt =
        args.monolith || args.monolith_bench_preset.is_some() || template_choice.is_some();
    if args.monolith {
        cfg.monolith_enabled = true;
    } else if !skip_monolith_prompt && cfg.project_type == ProjectType::KernelServer {
        cfg.monolith_enabled = pick_monolith_for_kernel(template_default_monolith)?;
    }

    if args.monolith_bench_preset.is_none()
        && cfg.monolith_enabled
        && cfg.project_type == ProjectType::KernelServer
    {
        let bench_after = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("After generation, auto-run Monolith vs standard bench (5 runs, writes bench_results/)?")
            .default(false)
            .interact()
            .context("bench after init")?;
        cfg.run_monolith_bench_after_init = bench_after;
    }

    Ok(cfg)
}

fn collect_base_options(args: &InitArgs) -> Result<InitArgs> {
    collect_base_options_with(args, |label, default, allow_empty| {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt(label)
            .default(default.to_string())
            .allow_empty(allow_empty)
            .interact_text()
            .context("kernel-base input")
    })
}

/// Tests supply answers to the same collection flow; validation/generation remain shared.
fn collect_base_options_with(
    args: &InitArgs,
    mut ask: impl FnMut(&str, &str, bool) -> Result<String>,
) -> Result<InitArgs> {
    let mut collected = args.clone();
    if collected.kernel_source.is_none() {
        collected.kernel_source =
            Some(ask("OCLive source root (contracts/types; no Host)", ".", false)?.into());
    }
    if collected.base_example.is_none() {
        let answer = ask(
            "Teaching example: memory or none (blank; contracts stay available)",
            "memory",
            false,
        )?;
        collected.base_example = Some(match answer.trim().to_ascii_lowercase().as_str() {
            "memory" => BaseExampleArg::Memory,
            "none" => BaseExampleArg::None,
            _ => anyhow::bail!("Teaching example must be memory or none"),
        });
    }
    if collected.author.is_none() {
        let answer = ask("Author (optional)", "", true)?;
        if !answer.trim().is_empty() {
            collected.author = Some(answer.trim().into());
        }
    }
    if collected.license.is_none() {
        collected.license = Some(ask("License (SPDX)", "MIT", false)?);
    }
    if collected.description.is_none() {
        let answer = ask("Short description (optional)", "", true)?;
        if !answer.trim().is_empty() {
            collected.description = Some(answer.trim().into());
        }
    }
    Ok(collected)
}

pub(crate) fn pick_base_output(args: &InitArgs) -> Result<std::path::PathBuf> {
    if args.output != std::path::Path::new("generated-kernel") {
        return Ok(args.output.clone());
    }
    let output: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Output directory (new or empty)")
        .default(args.output.display().to_string())
        .interact_text()
        .context("kernel-base output")?;
    Ok(output.into())
}

#[cfg(test)]
mod foundation_questions {
    use super::*;
    use clap::Parser;

    #[test]
    fn answers_reach_the_same_foundation_config_as_explicit_arguments() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let mut answers = [
            root.display().to_string(),
            "none".into(),
            "Module Author".into(),
            "Apache-2.0".into(),
            "My module".into(),
        ]
        .into_iter();
        let args = InitArgs::parse_from([
            "init",
            "--template",
            "kernel-base",
            "--project-name",
            "custom-module",
        ]);
        let collected =
            collect_base_options_with(&args, |_, _, _| Ok(answers.next().unwrap())).unwrap();
        assert!(answers.next().is_none());
        let from_answers = crate::kernel_base_scaffold::build_config(&collected).unwrap();
        let explicit = InitArgs::parse_from([
            "init",
            "--non-interactive",
            "--template",
            "kernel-base",
            "--project-name",
            "custom-module",
            "--kernel-source",
            root.to_str().unwrap(),
            "--base-example",
            "none",
            "--author",
            "Module Author",
            "--license",
            "Apache-2.0",
            "--description",
            "My module",
        ]);
        let from_flags = crate::init::build_init_config(&explicit, false).unwrap();
        assert_eq!(
            serde_json::to_value(from_answers).unwrap(),
            serde_json::to_value(from_flags).unwrap()
        );
    }

    #[test]
    fn explicit_options_skip_questions_and_invalid_answers_are_rejected() {
        let args = InitArgs::parse_from([
            "init",
            "--template",
            "kernel-base",
            "--kernel-source",
            ".",
            "--base-example",
            "memory",
            "--author",
            "",
            "--license",
            "MIT",
            "--description",
            "",
        ]);
        collect_base_options_with(&args, |_, _, _| panic!("explicit option was asked again"))
            .unwrap();
        let args =
            InitArgs::parse_from(["init", "--template", "kernel-base", "--kernel-source", "."]);
        assert!(collect_base_options_with(&args, |_, _, _| Ok("llm".into())).is_err());
    }
}
