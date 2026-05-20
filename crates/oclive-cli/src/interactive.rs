//! 交互式收集 [`crate::init::ProjectConfig`]（支持 CLI 已指定项时智能跳过）。

use crate::init::{
    BackendImpl, BackendSlots, FeatureSelection, InitArgs, InitTemplateArg, PluginSelection,
    ProjectConfig, ProjectType, ProjectTypeArg, RolePackKind,
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
                "ollama（主应用默认本地 LLM，需本机 Ollama）",
                "remote（HTTP 侧车，需 OCLIVE_REMOTE_LLM_URL）",
                "directory（目录插件子进程）",
                "none（禁用主 LLM 链；仅用于实验/占位）",
            ],
        )
    } else {
        (
            &["builtin", "remote", "directory", "none"],
            &[
                "builtin（进程内默认实现）",
                "remote（HTTP JSON-RPC，需 OCLIVE_REMOTE_PLUGIN_URL 等）",
                "directory（目录插件；需配置 plugin_backends.directory_plugins）",
                "none（禁用该子系统；部分槽可能影响主对话链）",
            ],
        )
    };
    let i = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("{slot} 使用哪种实现？"))
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
        return Err(anyhow!("槽位 {slot} 不能使用 ollama（仅 llm 槽合法）。"));
    }
    Ok(())
}

fn pick_factory_template() -> Result<Option<InitTemplateArg>> {
    let mut labels: Vec<String> = vec!["不使用模板 — 手动配置 preset / 七槽".into()];
    for e in CATALOG {
        labels.push(format!(
            "{} — {}（preset={}, Monolith={}）",
            e.id, e.scene, e.preset, e.monolith
        ));
    }
    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择场景模板（内核工厂配方）")
        .items(&labels)
        .default(0)
        .interact()
        .context("template select")?;
    if idx == 0 {
        return Ok(None);
    }
    let entry = CATALOG.get(idx - 1).context("template index")?;
    crate::template_catalog::template_from_id(entry.id)
        .ok_or_else(|| anyhow!("未知模板: {}", entry.id))
        .map(Some)
}

fn pick_project_type(default_kernel: bool) -> Result<ProjectType> {
    let default = if default_kernel { 0 } else { 1 };
    let pt_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("项目类型")
        .items(&["无头服务 (kernel_server)", "嵌入式库 (library)"])
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
        .with_prompt("要启用的后端槽位（至少勾选 memory / emotion / prompt / llm）")
        .items(SLOT_LABELS)
        .defaults(&[true, true, true, true, true, true, true])
        .interact()
        .context("backend multiselect")?;
    let required = [0usize, 1, 3, 4];
    for r in required {
        if !chosen.contains(&r) {
            return Err(anyhow!(
                "至少需要启用 memory、emotion、prompt、llm 四个槽位。"
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
        .with_prompt("可选插件")
        .items(&[
            "directory-plugins（生成目录插件占位说明）",
            "kernel-server（README 中说明如何接入 oclive_kernel_server）",
            "oocp（README 中说明 OOCP 对照测试入口）",
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
        .with_prompt("是否启用开发者编译选项（Monolith）？")
        .default(template_default_on)
        .interact()
        .context("developer compile option")?;
    if !dev_opt {
        return Ok(false);
    }
    let mode_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("编译模式")
        .items(&[
            "标准模式（低耦合，保留模块可替换性，推荐）",
            "高耦合模式 — 七槽全部静态焊接",
            "高耦合模式 — 自定义焊接范围（生成后编辑 monolith.toml，再运行 oclive build）",
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
        .with_prompt("作者（写入 Cargo.toml authors）")
        .default(default_author)
        .interact_text()
        .context("author")?;
    if !author.trim().is_empty() {
        cfg.cargo_author = Some(author.trim().to_string());
    }
    let licenses = ["MIT", "Apache-2.0", "GPL-3.0", "AGPL-3.0"];
    let lic_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("许可证（SPDX）")
        .items(&licenses)
        .default(0)
        .interact()
        .context("license")?;
    cfg.cargo_license = Some(licenses.get(lic_idx).copied().unwrap_or("MIT").to_string());
    let desc: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("简短描述（可选，留空不写 description）")
        .allow_empty(true)
        .interact_text()
        .context("description")?;
    if !desc.trim().is_empty() {
        cfg.cargo_description = Some(desc.trim().to_string());
    }
    Ok(())
}

fn resolve_project_name(args: &InitArgs) -> Result<String> {
    if !args.project_name.is_empty() && args.project_name != "my_oclive_kernel" {
        return Ok(args.project_name.clone());
    }
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("项目名（用于 Cargo package 与目录名）")
        .default("my_oclive_kernel".into())
        .interact_text()
        .context("project name")
}

/// 至少包含 memory / emotion / prompt / llm 四条线（与产品最小对话链一致）。
pub fn run_interactive(args: &InitArgs) -> Result<ProjectConfig> {
    let project_name = resolve_project_name(args)?;

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

    let (mut cfg, template_default_monolith) = match (template_choice, args.preset.as_deref()) {
        (Some(t), _) => {
            let c = project_config_from_template(&project_name, t);
            let monolith_default = c.monolith_enabled;
            println!(
                "\n已应用模板「{}」：preset={}，Monolith 默认={}，角色包={}。",
                CATALOG
                    .iter()
                    .find(|e| crate::template_catalog::template_from_id(e.id) == Some(t))
                    .map(|e| e.id)
                    .unwrap_or("?"),
                crate::init::template_defaults(t).preset,
                if c.monolith_enabled { "启用" } else { "关闭" },
                crate::template_catalog::role_pack_label(c.role_pack_kind)
            );
            println!("CLI 已传入的参数将覆盖模板默认值。\n");
            (c, monolith_default)
        }
        (None, Some(p)) => {
            let c = crate::init::preset_config(&project_name, p);
            println!("\n已使用 CLI 指定 preset={p}，跳过七槽多选。\n");
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
                run_monolith_bench_after_init: false,
                kernel_source: None,
                cargo_author: None,
                cargo_license: None,
                cargo_description: None,
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
            cfg.project_type =
                pick_project_type(cfg.project_type == ProjectType::KernelServer)?;
        }
    } else {
        cfg.project_type = match args.project_type.unwrap() {
            ProjectTypeArg::KernelServer => ProjectType::KernelServer,
            ProjectTypeArg::Library => ProjectType::Library,
        };
    }

    if manual_config {
        cfg.backends = pick_slots_manual()?;
        cfg.features.use_complex_emotion = cfg.backends.complex_emotion != BackendImpl::None;
        cfg.plugins = pick_plugins()?;
    }

    let skip_role_prompt = args.skip_role_pack
        || args.with_role_pack.is_some()
        || template_choice.is_some();
    if !skip_role_prompt {
        let default_role = cfg.role_pack_kind != RolePackKind::None;
        let example_role = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("是否生成示例角色包（roles/）？")
            .default(default_role)
            .interact()
            .context("example role")?;
        cfg.role_pack_kind = if example_role {
            RolePackKind::DefaultExample
        } else {
            RolePackKind::None
        };
    }

    let skip_monolith_prompt = args.monolith
        || args.monolith_bench_preset.is_some()
        || template_choice.is_some();
    if args.monolith {
        cfg.monolith_enabled = true;
    } else if !skip_monolith_prompt
        && cfg.project_type == ProjectType::KernelServer
    {
        cfg.monolith_enabled = pick_monolith_for_kernel(template_default_monolith)?;
    }

    if !args.monolith_bench_preset.is_some()
        && cfg.monolith_enabled
        && cfg.project_type == ProjectType::KernelServer
    {
        let bench_after = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("生成后自动跑 Monolith vs 标准 bench（5 轮，写入 bench_results/）？")
            .default(false)
            .interact()
            .context("bench after init")?;
        cfg.run_monolith_bench_after_init = bench_after;
    }

    Ok(cfg)
}
