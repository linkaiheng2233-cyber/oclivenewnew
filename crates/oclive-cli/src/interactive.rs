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
    let items = &["builtin", "stub", "none"];
    let i = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("{slot} 使用哪种实现？"))
        .items(items)
        .default(0)
        .interact()
        .context("select backend impl")?;
    Ok(match i {
        0 => BackendImpl::Builtin,
        1 => BackendImpl::Stub,
        _ => BackendImpl::None,
    })
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
        .defaults(&[true, true, true, true, true, false, false])
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

    let with_example_role = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("是否生成示例角色包（roles/default）？")
        .default(true)
        .interact()
        .context("example role")?;

    Ok(ProjectConfig {
        project_name,
        project_type,
        backends: slots,
        plugins,
        features: feats,
        with_example_role,
    })
}
