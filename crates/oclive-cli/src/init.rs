//! `init` 子命令：解析参数、合并预设、调用生成器。

use crate::generator;
use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
pub struct InitArgs {
    /// 输出目录（将创建该目录并写入新项目）
    #[arg(short = 'o', long, default_value = "generated-kernel")]
    pub output: PathBuf,

    /// 非交互模式（与 --preset 联用）
    #[arg(long)]
    pub non_interactive: bool,

    /// 跳过配置摘要与完成提示（脚本 / 测试用）
    #[arg(long)]
    pub quiet: bool,

    /// 预设：minimal | full | mixed
    #[arg(long)]
    pub preset: Option<String>,

    #[arg(long, default_value = "my_oclive_kernel")]
    pub project_name: String,

    /// 项目类型（非交互时必填；交互时可省略）
    #[arg(long, value_enum)]
    pub project_type: Option<ProjectTypeArg>,
}

#[derive(clap::ValueEnum, Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectTypeArg {
    KernelServer,
    Library,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendImpl {
    Builtin,
    Stub,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendSlots {
    pub memory: BackendImpl,
    pub emotion: BackendImpl,
    pub event: BackendImpl,
    pub prompt: BackendImpl,
    pub llm: BackendImpl,
    pub agent: BackendImpl,
    /// 与「复杂情感」扩展路线对应；写入 `_oclive_cli` 元数据，便于后续接入独立 crate。
    pub complex_emotion: BackendImpl,
}

impl BackendSlots {
    pub fn all(v: BackendImpl) -> Self {
        Self {
            memory: v,
            emotion: v,
            event: v,
            prompt: v,
            llm: v,
            agent: v,
            complex_emotion: v,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSelection {
    pub directory_plugins: bool,
    pub kernel_server: bool,
    pub oocp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSelection {
    pub use_complex_emotion: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
    KernelServer,
    Library,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project_name: String,
    pub project_type: ProjectType,
    pub backends: BackendSlots,
    pub plugins: PluginSelection,
    pub features: FeatureSelection,
    pub with_example_role: bool,
}

impl ProjectConfig {
    pub fn print_summary(&self) {
        println!("—— 配置摘要 ——");
        println!("项目名: {}", self.project_name);
        println!("类型: {:?}", self.project_type);
        println!(
            "后端: memory={:?} emotion={:?} event={:?} prompt={:?} llm={:?} agent={:?} complex_emotion={:?}",
            self.backends.memory,
            self.backends.emotion,
            self.backends.event,
            self.backends.prompt,
            self.backends.llm,
            self.backends.agent,
            self.backends.complex_emotion
        );
        println!(
            "插件: directory={} kernel_server_doc={} oocp_doc={}",
            self.plugins.directory_plugins, self.plugins.kernel_server, self.plugins.oocp
        );
        println!("示例角色包: {}", self.with_example_role);
        println!("——————————————");
    }
}

fn preset_config(name: &str, preset: &str) -> Result<ProjectConfig> {
    let project_name = if name.trim().is_empty() {
        "my_oclive_kernel".into()
    } else {
        name.to_string()
    };
    let project_type = ProjectType::KernelServer;
    let backends = if preset == "full" {
        BackendSlots {
            memory: BackendImpl::Builtin,
            emotion: BackendImpl::Builtin,
            event: BackendImpl::Builtin,
            prompt: BackendImpl::Builtin,
            llm: BackendImpl::Builtin,
            agent: BackendImpl::Builtin,
            complex_emotion: BackendImpl::Builtin,
        }
    } else if preset == "mixed" {
        BackendSlots {
            memory: BackendImpl::Builtin,
            emotion: BackendImpl::Stub,
            event: BackendImpl::None,
            prompt: BackendImpl::Builtin,
            llm: BackendImpl::Stub,
            agent: BackendImpl::None,
            complex_emotion: BackendImpl::Stub,
        }
    } else {
        BackendSlots {
            memory: BackendImpl::Stub,
            emotion: BackendImpl::Stub,
            event: BackendImpl::None,
            prompt: BackendImpl::Stub,
            llm: BackendImpl::Stub,
            agent: BackendImpl::None,
            complex_emotion: BackendImpl::None,
        }
    };
    let plugins = match preset {
        "full" => PluginSelection {
            directory_plugins: true,
            kernel_server: true,
            oocp: true,
        },
        "mixed" => PluginSelection {
            directory_plugins: true,
            kernel_server: false,
            oocp: true,
        },
        _ => PluginSelection {
            directory_plugins: false,
            kernel_server: false,
            oocp: false,
        },
    };
    let features = FeatureSelection {
        use_complex_emotion: backends.complex_emotion != BackendImpl::None,
    };
    let with_example_role = preset != "minimal";
    Ok(ProjectConfig {
        project_name,
        project_type,
        backends,
        plugins,
        features,
        with_example_role,
    })
}

pub fn run(args: InitArgs) -> Result<()> {
    let cfg = if args.non_interactive {
        let preset = args
            .preset
            .as_deref()
            .unwrap_or("minimal")
            .to_ascii_lowercase();
        let mut c = preset_config(&args.project_name, &preset)?;
        if let Some(t) = args.project_type {
            c.project_type = match t {
                ProjectTypeArg::KernelServer => ProjectType::KernelServer,
                ProjectTypeArg::Library => ProjectType::Library,
            };
        }
        c
    } else {
        let mut c = crate::interactive::run_interactive()?;
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

    if !args.non_interactive {
        cfg.print_summary();
        if !dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("确认生成到该目录？")
            .default(true)
            .interact()
            .context("confirm")?
        {
            println!("已取消。");
            return Ok(());
        }
    } else if !args.quiet {
        cfg.print_summary();
    }

    generator::write_project(&cfg, &args.output)?;
    if !args.quiet {
        println!(
            "已生成项目: {} （请在该目录执行 cargo build）",
            args.output.display()
        );
    }
    Ok(())
}
