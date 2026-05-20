//! 交互式收集 [`crate::init::ProjectConfig`]。

use crate::init::{
    BackendImpl, BackendSlots, FeatureSelection, InitTemplateArg, PluginSelection, ProjectConfig,
    ProjectType, RolePackKind,
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

/// 交互选择场景模板；`None` = 不使用模板、手动配置。
fn pick_factory_template() -> Result<Option<InitTemplateArg>> {
    let mut labels: Vec<String> = vec![
        "不使用模板 — 手动配置 preset / 七槽".into(),
    ];
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

fn pick_monolith_for_kernel(project_type: ProjectType, template_default_on: bool) -> Result<bool> {
    if project_type != ProjectType::KernelServer {
        return Ok(false);
    }
    let default_yes = template_default_on;
    let dev_opt = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("是否启用开发者编译选项（Monolith）？")
        .default(default_yes)
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

/// 至少包含 memory / emotion / prompt / llm 四条线（与产品最小对话链一致）。
pub fn run_interactive() -> Result<ProjectConfig> {
    let project_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("项目名（用于 Cargo package 与目录名）")
        .default("my_oclive_kernel".into())
        .interact_text()
        .context("project name")?;

    let template_choice = pick_factory_template()?;
    let manual_config = template_choice.is_none();

    let (mut cfg, template_default_monolith) = if let Some(t) = template_choice {
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
        println!("后续步骤仍可覆盖；非交互可用 --preset / --monolith 等显式参数优先。\n");
        (c, monolith_default)
    } else {
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
        };
        (c, false)
    };

    cfg.project_name = project_name;

    if manual_config {
        cfg.project_type = pick_project_type(true)?;
        cfg.backends = pick_slots_manual()?;
        cfg.features.use_complex_emotion = cfg.backends.complex_emotion != BackendImpl::None;
        cfg.plugins = pick_plugins()?;
    } else {
        cfg.project_type =
            pick_project_type(cfg.project_type == ProjectType::KernelServer)?;
    }

    let default_role = cfg.role_pack_kind != RolePackKind::None;
    let example_role = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("是否生成示例角色包（roles/）？")
        .default(default_role)
        .interact()
        .context("example role")?;
    cfg.role_pack_kind = if example_role {
        if cfg.role_pack_kind == RolePackKind::None {
            RolePackKind::DefaultExample
        } else {
            cfg.role_pack_kind
        }
    } else {
        RolePackKind::None
    };

    cfg.monolith_enabled =
        pick_monolith_for_kernel(cfg.project_type, template_default_monolith)?;

    let bench_after = if cfg.monolith_enabled && cfg.project_type == ProjectType::KernelServer {
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("生成后自动跑 Monolith vs 标准 bench（5 轮，写入 bench_results/）？")
            .default(false)
            .interact()
            .context("bench after init")?
    } else {
        false
    };
    cfg.run_monolith_bench_after_init = bench_after;

    Ok(cfg)
}
