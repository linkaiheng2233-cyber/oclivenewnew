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

开发者编译选项：非交互可加 **`--monolith`**（仅 kernel_server）；交互流程末尾询问。生成 `monolith.toml`（由 init 生成、**`oclive build`** 读取并再生成 `process_message_monolith.rs`）。子命令 **`build`** / **`bench`** 见 **`cargo run -p oclive-cli -- --help`**。详见 creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md。

内核工厂模板（`--template`，与 `--preset` / `--project-type` / `--monolith` 可叠加；显式 CLI 参数优先）：

┌─────────────────┬─────────┬──────────────────┬────────────────┬──────────────────────────────┐
│ template        │ preset  │ monolith 默认    │ project-type   │ 默认 --with-role-pack        │
├─────────────────┼─────────┼──────────────────┼────────────────┼──────────────────────────────┤
│ robot-soul      │ minimal │ 启用             │ kernel_server  │ robot-soul-minimal           │
│ robot-gateway   │ mixed   │ 启用             │ kernel_server  │ gateway 骨架 + mcp_servers/  │
│ dialogue-only   │ full    │ 关闭（可加 --monolith） │ kernel_server  │ default（通用示例）          │
│ headless-api    │ full    │ 关闭（可加 --monolith） │ kernel_server  │ 无（空 roles/）              │
│ library-embed   │ minimal │ 关闭             │ library        │ 无                           │
└─────────────────┴─────────┴──────────────────┴────────────────┴──────────────────────────────┘

`--monolith-preset`（仅 `--monolith` 或模板默认启用 Monolith 时生效）：`latency`（七槽全焊）| `memory`（memory+prompt+llm）| `embedded`（emotion+memory+llm）。生成 `monolith.toml` 的 `weld_modules` 可事后手改。

`--with-role-pack`：`robot-soul-minimal` | `default`；未指定且未用模板时，非交互仍生成通用 `roles/default`（与历史行为一致）。`--skip-role-pack` 强制不生成 `roles/`。

`--with-example-plugin`：复制主仓 `examples/directory-plugin-llamacpp/` 到 `plugins/com.oclive.example.llamacpp_llm/`（默认关闭）。

`--list-templates`：打印上表模板矩阵后退出（不生成工程）。

`--monolith-bench-preset`（仅 Monolith 启用时）：生成后自动 `cargo build --release`（双二进制）并 `bench --runs 5`，结果写入 `bench_results/report.json`；失败不阻塞生成。
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

    /// 打印内核工厂模板矩阵后退出（不写入输出目录）
    #[arg(long)]
    pub list_templates: bool,

    /// 内核工厂模板：robot-soul | robot-gateway | dialogue-only | headless-api | library-embed
    #[arg(long, value_enum)]
    pub template: Option<InitTemplateArg>,

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

    /// 非交互：启用 Monolith（仅 `kernel_server` 生效；生成 `monolith.toml`、`vendor/`、焊接源码与双 `[[bin]]`）
    #[arg(long)]
    pub monolith: bool,

    /// Monolith 焊接档位（仅 Monolith 启用时写入 monolith.toml 的 weld_modules）
    #[arg(long, value_enum)]
    pub monolith_preset: Option<MonolithPresetArg>,

    /// 生成后自动跑 Monolith vs 标准 bench（5 轮）；同时可设定焊接档位（等同 --monolith-preset）
    #[arg(long, value_enum)]
    pub monolith_bench_preset: Option<MonolithPresetArg>,

    /// 非交互：不在生成项目中写入 `roles/` 示例角色包
    #[arg(long)]
    pub skip_role_pack: bool,

    /// 生成示例角色包：robot-soul-minimal | default（未指定时由 --template 或历史默认决定）
    #[arg(long, value_enum)]
    pub with_role_pack: Option<RolePackKindArg>,

    /// 附带 llamacpp 目录插件示例到 plugins/com.oclive.example.llamacpp_llm/
    #[arg(long)]
    pub with_example_plugin: bool,

    /// 指向 oclivenewnew 仓库根：生成项目写入 `oclivenewnew-tauri` / `oclive_kernel_runtime` path 依赖
    #[arg(long)]
    pub kernel_source: Option<PathBuf>,
}

/// 内核工厂套餐（`--template`）。
#[derive(clap::ValueEnum, Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum InitTemplateArg {
    RobotSoul,
    RobotGateway,
    DialogueOnly,
    HeadlessApi,
    LibraryEmbed,
}

/// Monolith 性能档位（`--monolith-preset`）。
#[derive(clap::ValueEnum, Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum MonolithPresetArg {
    Latency,
    Memory,
    Embedded,
}

/// 示例角色包种类（`--with-role-pack`）。
#[derive(clap::ValueEnum, Clone, Debug, Copy, PartialEq, Eq)]
#[clap(rename_all = "kebab-case")]
pub enum RolePackKindArg {
    RobotSoulMinimal,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RolePackKind {
    None,
    DefaultExample,
    RobotSoulMinimal,
}

impl From<RolePackKindArg> for RolePackKind {
    fn from(v: RolePackKindArg) -> Self {
        match v {
            RolePackKindArg::Default => RolePackKind::DefaultExample,
            RolePackKindArg::RobotSoulMinimal => RolePackKind::RobotSoulMinimal,
        }
    }
}

pub struct TemplateDefaults {
    pub preset: &'static str,
    pub project_type: ProjectType,
    /// 模板是否默认启用 Monolith（仅 kernel_server；可被 `--monolith` 覆盖为启用）
    pub monolith_default: bool,
    pub role_pack: RolePackKind,
}

pub fn template_defaults(t: InitTemplateArg) -> TemplateDefaults {
    match t {
        InitTemplateArg::RobotSoul => TemplateDefaults {
            preset: "minimal",
            project_type: ProjectType::KernelServer,
            monolith_default: true,
            role_pack: RolePackKind::RobotSoulMinimal,
        },
        InitTemplateArg::RobotGateway => TemplateDefaults {
            preset: "mixed",
            project_type: ProjectType::KernelServer,
            monolith_default: true,
            role_pack: RolePackKind::None,
        },
        InitTemplateArg::DialogueOnly => TemplateDefaults {
            preset: "full",
            project_type: ProjectType::KernelServer,
            monolith_default: false,
            role_pack: RolePackKind::DefaultExample,
        },
        InitTemplateArg::HeadlessApi => TemplateDefaults {
            preset: "full",
            project_type: ProjectType::KernelServer,
            monolith_default: false,
            role_pack: RolePackKind::None,
        },
        InitTemplateArg::LibraryEmbed => TemplateDefaults {
            preset: "minimal",
            project_type: ProjectType::Library,
            monolith_default: false,
            role_pack: RolePackKind::None,
        },
    }
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
    /// 生成哪种示例角色包（`None` = 不创建 `roles/`）。
    pub role_pack_kind: RolePackKind,
    /// 仅 `kernel_server` 且为 true 时生成 `monolith.toml` 与 Monolith 构建配置。
    pub monolith_enabled: bool,
    /// Monolith 焊接档位；仅 `monolith_enabled` 时用于 init 生成的 `monolith.toml`。
    pub monolith_preset: Option<MonolithPresetArg>,
    /// 为 true 时不生成 `roles/` 目录（空白内核模板）。
    pub skip_role_pack: bool,
    /// 复制 llamacpp 示例目录插件到 `plugins/`。
    pub with_example_plugin: bool,
    /// 使用的内核工厂模板（生成 robot-gateway MCP 等产物）。
    pub factory_template: Option<InitTemplateArg>,
    /// 生成完成后自动执行 Monolith 基准测试。
    pub run_monolith_bench_after_init: bool,
    /// 指向 oclivenewnew 仓库根；生成项目写入 path 依赖并替换占位 `main`/`lib`。
    pub kernel_source: Option<std::path::PathBuf>,
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
        println!(
            "角色包模板: {}",
            if self.skip_role_pack {
                "无（不生成 roles/）"
            } else {
                match self.role_pack_kind {
                    RolePackKind::None => "无（不生成 roles/）",
                    RolePackKind::DefaultExample => "default（通用示例 roles/default）",
                    RolePackKind::RobotSoulMinimal => "robot-soul-minimal（七维 + prompts/system.md）",
                }
            }
        );
        if self.monolith_enabled {
            let preset = self
                .monolith_preset
                .map(|p| format!("{p:?}"))
                .unwrap_or_else(|| "默认（七槽全焊）".into());
            println!(
                "开发者编译: Monolith（焊接档位: {preset}；见 monolith.toml；`oclive build` 再生成）"
            );
        }
        if self.with_example_plugin {
            println!("示例插件: plugins/com.oclive.example.llamacpp_llm/");
        }
        if let Some(t) = self.factory_template {
            println!("工厂模板: {t:?}");
        }
        if self.run_monolith_bench_after_init {
            println!("生成后: 自动 Monolith bench（5 轮）→ bench_results/report.json");
        }
        if let Some(ks) = &self.kernel_source {
            println!("内核源码: {}（已接 runtime / HTTP 入口）", ks.display());
        }
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
    ProjectConfig {
        project_name,
        project_type,
        backends,
        plugins,
        features,
        role_pack_kind: RolePackKind::DefaultExample,
        monolith_enabled: false,
        monolith_preset: None,
        skip_role_pack: false,
        with_example_plugin: false,
        factory_template: None,
        run_monolith_bench_after_init: false,
        kernel_source: None,
    }
}

/// 解析 Monolith 焊接档位（无 `--monolith-preset` 时默认七槽全焊）。
pub fn resolve_monolith_weld_modules(cfg: &ProjectConfig) -> Vec<&'static str> {
    if !cfg.monolith_enabled {
        return vec![];
    }
    match cfg.monolith_preset {
        Some(p) => crate::monolith_codegen::weld_modules_for_preset(p),
        None => crate::monolith_config::SLOT_IDS.to_vec(),
    }
}

/// 合并 `--template` 与显式 CLI 覆盖（显式优先）。
pub fn apply_template_layer(args: &InitArgs, cfg: &mut ProjectConfig) {
    let Some(t) = args.template else {
        return;
    };
    let td = template_defaults(t);
    if args.preset.is_none() {
        let fresh = preset_config(&cfg.project_name, td.preset);
        cfg.backends = fresh.backends;
        cfg.plugins = fresh.plugins;
        cfg.features = fresh.features;
    }
    if args.project_type.is_none() {
        cfg.project_type = td.project_type;
    }
}

pub fn resolve_monolith(args: &InitArgs, cfg: &mut ProjectConfig) {
    if cfg.project_type != ProjectType::KernelServer {
        cfg.monolith_enabled = false;
        return;
    }
    if args.monolith {
        cfg.monolith_enabled = true;
        return;
    }
    if let Some(t) = args.template {
        let td = template_defaults(t);
        if td.monolith_default {
            cfg.monolith_enabled = true;
        }
    }
}

pub fn resolve_role_pack_kind(args: &InitArgs) -> RolePackKind {
    if args.skip_role_pack {
        return RolePackKind::None;
    }
    if let Some(rp) = args.with_role_pack {
        return rp.into();
    }
    if let Some(t) = args.template {
        return template_defaults(t).role_pack;
    }
    RolePackKind::DefaultExample
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
    if args.list_templates {
        crate::template_catalog::print_templates_table();
        return Ok(());
    }

    let mut cfg = if args.non_interactive {
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

    apply_template_layer(&args, &mut cfg);
    resolve_monolith(&args, &mut cfg);
    if cfg.monolith_enabled {
        cfg.monolith_preset = args.monolith_preset.or(args.monolith_bench_preset);
    }
    if args.monolith_bench_preset.is_some() {
        if cfg.project_type == ProjectType::KernelServer {
            if !cfg.monolith_enabled {
                cfg.monolith_enabled = true;
            }
            cfg.monolith_preset = cfg.monolith_preset.or(args.monolith_bench_preset);
            cfg.run_monolith_bench_after_init = cfg.monolith_enabled;
        } else if !args.quiet {
            eprintln!("⚠ --monolith-bench-preset 仅对 kernel_server 生效，已忽略。");
        }
    }
    cfg.factory_template = args.template;
    cfg.with_example_plugin = args.with_example_plugin;
    cfg.role_pack_kind = resolve_role_pack_kind(&args);
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
    if cfg.run_monolith_bench_after_init {
        crate::init_bench::try_post_init_monolith_bench(&args.output);
    }
    if !args.quiet {
        if cfg.monolith_enabled && matches!(cfg.project_type, ProjectType::KernelServer) {
            let slug = crate::generator::project_slug(&cfg);
            println!("✓ 项目已创建！");
            println!("  标准模式二进制: target/release/{slug}");
            println!("  高耦合模式二进制: target/release/{slug}-monolith");
            println!("  配置文件: monolith.toml");
            println!(
                "  修改焊接计划后: 于项目根执行 oclive build（或 cargo run -p oclive-cli -- build -o {}）",
                args.output.display()
            );
            println!(
                "  性能对比: oclive bench --release -o {}",
                args.output.display()
            );
            println!("  详细说明: 参见 creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md");
            println!("输出目录: {}", args.output.display());
        } else {
            println!(
                "已生成项目: {} （请在该目录执行 cargo build）",
                args.output.display()
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod template_tests {
    use super::*;

    #[test]
    fn template_robot_soul_defaults() {
        let td = template_defaults(InitTemplateArg::RobotSoul);
        assert_eq!(td.preset, "minimal");
        assert!(td.monolith_default);
        assert_eq!(td.project_type, ProjectType::KernelServer);
        assert_eq!(td.role_pack, RolePackKind::RobotSoulMinimal);
    }

    #[test]
    fn template_headless_api_defaults() {
        let td = template_defaults(InitTemplateArg::HeadlessApi);
        assert_eq!(td.preset, "full");
        assert!(!td.monolith_default);
        assert_eq!(td.role_pack, RolePackKind::None);
    }

    #[test]
    fn preset_override_wins_over_template() {
        let args = InitArgs {
            output: PathBuf::from("out"),
            non_interactive: true,
            quiet: true,
            template: Some(InitTemplateArg::RobotSoul),
            preset: Some("full".into()),
            project_name: "t".into(),
            project_type: None,
            backend_memory: None,
            backend_emotion: None,
            backend_event: None,
            backend_prompt: None,
            backend_llm: None,
            backend_agent: None,
            backend_complex_emotion: None,
            monolith: false,
            monolith_preset: None,
            monolith_bench_preset: None,
            list_templates: false,
            skip_role_pack: false,
            with_role_pack: None,
            with_example_plugin: false,
            kernel_source: None,
        };
        let preset = args.preset.as_deref().unwrap_or("minimal");
        let mut cfg = preset_config("t", preset);
        apply_template_layer(&args, &mut cfg);
        assert_eq!(cfg.backends.llm, BackendImpl::Remote);
    }

    #[test]
    fn robot_soul_template_enables_monolith_without_flag() {
        let args = InitArgs {
            output: PathBuf::from("out"),
            non_interactive: true,
            quiet: true,
            template: Some(InitTemplateArg::RobotSoul),
            preset: None,
            project_name: "t".into(),
            project_type: None,
            backend_memory: None,
            backend_emotion: None,
            backend_event: None,
            backend_prompt: None,
            backend_llm: None,
            backend_agent: None,
            backend_complex_emotion: None,
            monolith: false,
            monolith_preset: None,
            monolith_bench_preset: None,
            list_templates: false,
            skip_role_pack: false,
            with_role_pack: None,
            with_example_plugin: false,
            kernel_source: None,
        };
        let mut cfg = preset_config("t", "minimal");
        apply_template_layer(&args, &mut cfg);
        resolve_monolith(&args, &mut cfg);
        assert!(cfg.monolith_enabled);
    }

    #[test]
    fn with_role_pack_overrides_template_default() {
        let args = InitArgs {
            output: PathBuf::from("out"),
            non_interactive: true,
            quiet: true,
            template: Some(InitTemplateArg::RobotSoul),
            preset: None,
            project_name: "t".into(),
            project_type: None,
            backend_memory: None,
            backend_emotion: None,
            backend_event: None,
            backend_prompt: None,
            backend_llm: None,
            backend_agent: None,
            backend_complex_emotion: None,
            monolith: false,
            monolith_preset: None,
            monolith_bench_preset: None,
            list_templates: false,
            skip_role_pack: false,
            with_role_pack: Some(RolePackKindArg::Default),
            with_example_plugin: false,
            kernel_source: None,
        };
        assert_eq!(
            resolve_role_pack_kind(&args),
            RolePackKind::DefaultExample
        );
    }

    #[test]
    fn monolith_bench_preset_enables_post_bench() {
        let args = InitArgs {
            output: PathBuf::from("out"),
            non_interactive: true,
            quiet: true,
            template: None,
            preset: Some("minimal".into()),
            project_name: "t".into(),
            project_type: Some(ProjectTypeArg::KernelServer),
            backend_memory: None,
            backend_emotion: None,
            backend_event: None,
            backend_prompt: None,
            backend_llm: None,
            backend_agent: None,
            backend_complex_emotion: None,
            monolith: false,
            monolith_preset: None,
            monolith_bench_preset: Some(MonolithPresetArg::Latency),
            list_templates: false,
            skip_role_pack: true,
            with_role_pack: None,
            with_example_plugin: false,
            kernel_source: None,
        };
        let mut cfg = preset_config("t", "minimal");
        apply_backend_cli_overrides(&mut cfg, &args);
        cfg.project_type = ProjectType::KernelServer;
        resolve_monolith(&args, &mut cfg);
        if args.monolith_bench_preset.is_some() {
            cfg.monolith_enabled = true;
            cfg.monolith_preset = args.monolith_bench_preset;
            cfg.run_monolith_bench_after_init = true;
        }
        assert!(cfg.run_monolith_bench_after_init);
        assert!(cfg.monolith_enabled);
    }

    #[test]
    fn robot_gateway_template_defaults() {
        let td = template_defaults(InitTemplateArg::RobotGateway);
        assert_eq!(td.preset, "mixed");
        assert!(td.monolith_default);
        assert_eq!(td.role_pack, RolePackKind::None);
    }

    #[test]
    fn dialogue_only_template_defaults() {
        let td = template_defaults(InitTemplateArg::DialogueOnly);
        assert_eq!(td.preset, "full");
        assert!(!td.monolith_default);
        assert_eq!(td.role_pack, RolePackKind::DefaultExample);
    }

    #[test]
    fn monolith_preset_latency_welds_all_slots() {
        let mut cfg = preset_config("t", "minimal");
        cfg.monolith_enabled = true;
        cfg.monolith_preset = Some(MonolithPresetArg::Latency);
        let weld = resolve_monolith_weld_modules(&cfg);
        assert_eq!(weld.len(), 7);
    }
}
