//! `init` 子命令：解析参数、合并预设、调用生成器。
//!
//! 非交互模式下 **`--preset`** 决定基线；可选 **`--backend-*`** 逐项覆盖。
//! 预设与 `plugin_backends` 矩阵见 **`init --help` 末尾**（与生成项目根目录 **`CONFIG_REFERENCE.md`** 一致）。

use crate::generator;
use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// `init --help` / `init -h` 末尾：预设与 `plugin_backends` 矩阵（与生成项目内 `CONFIG_REFERENCE.md` 一致）。
pub const PRESET_MATRIX_HELP: &str = r#"预设与 plugin_backends（逻辑槽位）

┌───────────────────┬─────────┬─────────┬────────┐
│ 槽位              │ minimal │ mixed   │ full   │
├───────────────────┼─────────┼─────────┼────────┤
│ memory            │ builtin │ builtin │ builtin│
│ emotion           │ builtin │ builtin │ builtin│
│ event             │ builtin │ builtin │ builtin│
│ prompt            │ builtin │ builtin │ builtin│
│ llm               │ ollama  │ ollama  │ remote │
│ agent             │ none*   │ builtin │ builtin│
│ complex_emotion   │ none    │ builtin │ remote │
└───────────────────┴─────────┴─────────┴────────┘

* agent = none：写入 settings.json 时省略 agent 键（内核无 none 枚举；加载时回退默认 builtin）。

llm 使用 ollama 表示进程内默认本地客户端；无本机模型时请改为 remote 并配置 OCLIVE_REMOTE_LLM_URL（见 PLUGIN_V1）。

开发者编译选项（计划中）：高耦合编译模式可消除热路径上的模块虚调用开销；`monolith.toml` 由 init 生成、build 读取。详见 creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md。
"#;

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

    /// 覆盖 memory 槽（缺省沿用 `--preset`）
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
}

#[derive(clap::ValueEnum, Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectTypeArg {
    KernelServer,
    Library,
}

/// 与 `settings.json` → `plugin_backends` 对齐的脚手架内部表示。
///
/// - **`Ollama`**：仅用于 **`llm`** 槽，序列化为 JSON 字符串 **`ollama`**（主应用默认本地 LLM 后端）。
/// - **`None`**：用于 **`agent`** 时表示「不在 JSON 中写 agent 键」；用于 **`complex_emotion`** 时写 **`none`**（宿主反序列化六槽结构时忽略该扩展键）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum, Default)]
#[serde(rename_all = "snake_case")]
#[clap(rename_all = "snake_case")]
pub enum BackendImpl {
    #[default]
    Builtin,
    Remote,
    Directory,
    /// 仅 `llm` 槽合法；对应主应用 `LlmBackend::Ollama`。
    Ollama,
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

/// 与 `PRESET_MATRIX_HELP` / `CONFIG_REFERENCE.md` 完全一致。
pub fn preset_config(name: &str, preset: &str) -> ProjectConfig {
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
            llm: BackendImpl::Remote,
            agent: BackendImpl::Builtin,
            complex_emotion: BackendImpl::Remote,
        }
    } else if preset == "mixed" {
        BackendSlots {
            memory: BackendImpl::Builtin,
            emotion: BackendImpl::Builtin,
            event: BackendImpl::Builtin,
            prompt: BackendImpl::Builtin,
            llm: BackendImpl::Ollama,
            agent: BackendImpl::Builtin,
            complex_emotion: BackendImpl::Builtin,
        }
    } else {
        // minimal（含未知 preset 名时回退为 minimal）
        BackendSlots {
            memory: BackendImpl::Builtin,
            emotion: BackendImpl::Builtin,
            event: BackendImpl::Builtin,
            prompt: BackendImpl::Builtin,
            llm: BackendImpl::Ollama,
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
    // 始终生成示例角色包与 settings.json，便于对照 CONFIG_REFERENCE.md 验收。
    let with_example_role = true;
    ProjectConfig {
        project_name,
        project_type,
        backends,
        plugins,
        features,
        with_example_role,
    }
}

pub(crate) fn apply_backend_cli_overrides(cfg: &mut ProjectConfig, args: &InitArgs) {
    if let Some(v) = args.backend_memory {
        cfg.backends.memory = v;
    }
    if let Some(v) = args.backend_emotion {
        cfg.backends.emotion = v;
    }
    if let Some(v) = args.backend_event {
        cfg.backends.event = v;
    }
    if let Some(v) = args.backend_prompt {
        cfg.backends.prompt = v;
    }
    if let Some(v) = args.backend_llm {
        cfg.backends.llm = v;
    }
    if let Some(v) = args.backend_agent {
        cfg.backends.agent = v;
    }
    if let Some(v) = args.backend_complex_emotion {
        cfg.backends.complex_emotion = v;
    }
    cfg.features.use_complex_emotion = cfg.backends.complex_emotion != BackendImpl::None;
}

pub fn run(args: InitArgs) -> Result<()> {
    let cfg = if args.non_interactive {
        let preset = args
            .preset
            .as_deref()
            .unwrap_or("minimal")
            .to_ascii_lowercase();
        let mut c = preset_config(&args.project_name, &preset);
        apply_backend_cli_overrides(&mut c, &args);
        if let Some(t) = args.project_type {
            c.project_type = match t {
                ProjectTypeArg::KernelServer => ProjectType::KernelServer,
                ProjectTypeArg::Library => ProjectType::Library,
            };
        }
        c
    } else {
        let mut c = crate::interactive::run_interactive()?;
        apply_backend_cli_overrides(&mut c, &args);
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
