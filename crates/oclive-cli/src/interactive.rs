//! 交互式收集 [`crate::init::ProjectConfig`]。

use crate::init::{
    BackendImpl, BackendSlots, FeatureSelection, PluginSelection, ProjectConfig, ProjectType,
};
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

/// 至少包含 memory / emotion / prompt / llm 四条线（与产品最小对话链一致）。
pub fn run_interactive() -> Result<ProjectConfig> {
    let project_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("项目名（用于 Cargo package 与目录名）")
        .default("my_oclive_kernel".into())
        .interact_text()
        .context("project name")?;

    let pt_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("项目类型")
        .items(&["无头服务 (kernel_server)", "嵌入式库 (library)"])
        .default(0)
        .interact()
        .context("project type")?;
    let project_type = if pt_idx == 0 {
        ProjectType::KernelServer
    } else {
        ProjectType::Library
    };

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

    let plug_idx: Vec<usize> = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("可选插件")
        .items(&[
            "directory-plugins（生成目录插件占位说明）",
            "kernel-server（README 中说明如何接入 oclive_kernel_server）",
            "oocp（README 中说明 OOCP 对照测试入口）",
        ])
        .interact()
        .context("plugins")?;

    let plugins = PluginSelection {
        directory_plugins: plug_idx.contains(&0),
        kernel_server: plug_idx.contains(&1),
        oocp: plug_idx.contains(&2),
    };

    let feats = FeatureSelection {
        use_complex_emotion: slots.complex_emotion != BackendImpl::None,
    };

    let example_role = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("是否生成示例角色包（roles/default）？")
        .default(true)
        .interact()
        .context("example role")?;
    let role_pack_kind = if example_role {
        crate::init::RolePackKind::DefaultExample
    } else {
        crate::init::RolePackKind::None
    };

    let mut monolith_enabled = false;
    if project_type == ProjectType::KernelServer {
        let dev_opt = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("是否启用开发者编译选项?")
            .default(false)
            .interact()
            .context("developer compile option")?;
        if dev_opt {
            let mode_idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("编译模式")
                .items(&[
                    "标准模式（低耦合，保留模块可替换性，推荐）",
                    "高耦合模式 — 七槽全部静态焊接",
                    "高耦合模式 — 自定义焊接范围（生成后编辑 monolith.toml，再运行 oclive build）",
                ])
                .default(0)
                .interact()
                .context("compile mode")?;
            if mode_idx == 1 || mode_idx == 2 {
                monolith_enabled = true;
            }
        }
    }

    Ok(ProjectConfig {
        monolith_preset: None,
        with_example_plugin: false,
        project_name,
        project_type,
        backends: slots,
        plugins,
        features: feats,
        role_pack_kind,
        monolith_enabled,
        skip_role_pack: false,
        kernel_source: None,
    })
}
