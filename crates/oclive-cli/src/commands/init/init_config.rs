//! Init presets, project configuration types, and builders.

use super::InitArgs;
use serde::{Deserialize, Serialize};
/// Kernel factory recipe (`--template`).
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

/// Monolith performance tier (`--monolith-preset`).
#[derive(clap::ValueEnum, Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum MonolithPresetArg {
    Latency,
    Memory,
    Embedded,
}

/// Example role pack kind (`--with-role-pack`).
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

#[must_use]
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
    /// Valid only for the `llm` slot; maps to host `LlmBackend::Ollama`.
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
    #[must_use]
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
    /// 生成 `Cargo.toml` 的 `[package].authors`。
    pub cargo_author: Option<String>,
    /// 生成 `Cargo.toml` 的 `[package].license`（缺省 MIT）。
    pub cargo_license: Option<String>,
    /// 生成 `Cargo.toml` 的 `[package].description`。
    pub cargo_description: Option<String>,
    /// 编排模式（生成 `docs/PIPELINE_CUSTOM.md` 与 `src/oclive_pipeline_order.rs`）。
    pub pipeline: crate::pipeline::PipelineArg,
    /// 自定义 `monolith.toml` 的 `weld_modules`（TUI 或 `--weld-modules`）。
    pub custom_weld_modules: Option<Vec<String>>,
}

impl ProjectConfig {
    pub fn print_summary(&self) {
        println!("—— Configuration summary ——");
        println!("Project name: {}", self.project_name);
        println!("Type: {:?}", self.project_type);
        println!(
            "Backends: memory={:?} emotion={:?} event={:?} prompt={:?} llm={:?} agent={:?} complex_emotion={:?}",
            self.backends.memory,
            self.backends.emotion,
            self.backends.event,
            self.backends.prompt,
            self.backends.llm,
            self.backends.agent,
            self.backends.complex_emotion
        );
        println!(
            "Plugins: directory={} kernel_server_doc={} oocp_doc={}",
            self.plugins.directory_plugins, self.plugins.kernel_server, self.plugins.oocp
        );
        println!(
            "Role pack template: {}",
            if self.skip_role_pack {
                "none (no roles/)"
            } else {
                match self.role_pack_kind {
                    RolePackKind::None => "none (no roles/)",
                    RolePackKind::DefaultExample => "default (example roles/default)",
                    RolePackKind::RobotSoulMinimal => {
                        "robot-soul-minimal (seven dims + prompts/system.md)"
                    }
                }
            }
        );
        if self.monolith_enabled {
            let preset = self
                .monolith_preset
                .map(|p| format!("{p:?}"))
                .unwrap_or_else(|| "default (all seven weld keys welded)".into());
            println!(
                "Developer build: Monolith (weld tier: {preset}; see monolith.toml; run `oclive build` to regenerate)"
            );
        }
        if self.with_example_plugin {
            println!("Example plugin: plugins/com.oclive.example.llamacpp_llm/");
        }
        if let Some(t) = self.factory_template {
            println!("Factory template: {t:?}");
        }
        if self.run_monolith_bench_after_init {
            println!("After generation: auto Monolith bench (5 runs) → bench_results/report.json");
        }
        if let Some(ks) = &self.kernel_source {
            println!(
                "Kernel source: {} (runtime / HTTP entry wired)",
                ks.display()
            );
        }
        println!("——————————————");
    }
}

/// 与 `PRESET_MATRIX_HELP` / `CONFIG_REFERENCE.md` 完全一致。
#[must_use]
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
        cargo_author: None,
        cargo_license: None,
        cargo_description: None,
        pipeline: crate::pipeline::PipelineArg::Default,
        custom_weld_modules: None,
    }
}

/// 解析 Monolith 焊接档位（无 `--monolith-preset` 时默认七焊接键全焊）。
#[must_use]
pub fn resolve_monolith_weld_modules(cfg: &ProjectConfig) -> Vec<String> {
    if !cfg.monolith_enabled {
        return vec![];
    }
    if let Some(ref custom) = cfg.custom_weld_modules {
        return custom.clone();
    }
    match cfg.monolith_preset {
        Some(p) => crate::monolith_codegen::weld_modules_for_preset(p)
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
        None => crate::monolith_config::SLOT_IDS
            .iter()
            .map(|s| s.to_string())
            .collect(),
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

#[must_use]
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

pub(crate) fn git_config_user_name() -> Option<String> {
    std::process::Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub(crate) fn apply_cargo_metadata_cli(cfg: &mut ProjectConfig, args: &InitArgs) {
    if let Some(a) = args
        .author
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        cfg.cargo_author = Some(a.to_string());
    }
    if let Some(l) = args
        .license
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        cfg.cargo_license = Some(l.to_string());
    }
    if let Some(d) = args
        .description
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        cfg.cargo_description = Some(d.to_string());
    }
}

pub(crate) fn ensure_cargo_license_default(cfg: &mut ProjectConfig) {
    if cfg.cargo_license.is_none() {
        cfg.cargo_license = Some("MIT".into());
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

/// `--quick` 默认配方：full 预设、无头服务、无 Monolith、无示例角色包。
#[must_use]
pub fn quick_project_config(project_name: &str) -> ProjectConfig {
    let mut cfg = preset_config(project_name, "full");
    cfg.monolith_enabled = false;
    cfg.skip_role_pack = true;
    cfg.role_pack_kind = RolePackKind::None;
    cfg.kernel_source = None;
    cfg.with_example_plugin = false;
    cfg.run_monolith_bench_after_init = false;
    cfg
}

#[cfg(test)]
mod template_tests {
    use std::path::PathBuf;

    use super::*;
    use crate::pipeline::PipelineArg;

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
            quick: false,
            skip_role_pack: false,
            with_role_pack: None,
            with_example_plugin: false,
            kernel_source: None,
            author: None,
            license: None,
            description: None,
            template_url: None,
            tui: false,
            smart: false,
            no_smart: true,
            pipeline: PipelineArg::Default,
            weld_modules: vec![],
            from_existing: None,
            share: false,
            json: false,
            dry_run: false,
            check: false,
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
            quick: false,
            skip_role_pack: false,
            with_role_pack: None,
            with_example_plugin: false,
            kernel_source: None,
            author: None,
            license: None,
            description: None,
            template_url: None,
            tui: false,
            smart: false,
            no_smart: true,
            pipeline: PipelineArg::Default,
            weld_modules: vec![],
            from_existing: None,
            share: false,
            json: false,
            dry_run: false,
            check: false,
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
            quick: false,
            skip_role_pack: false,
            with_role_pack: Some(RolePackKindArg::Default),
            with_example_plugin: false,
            kernel_source: None,
            author: None,
            license: None,
            description: None,
            template_url: None,
            tui: false,
            smart: false,
            no_smart: true,
            pipeline: PipelineArg::Default,
            weld_modules: vec![],
            from_existing: None,
            share: false,
            json: false,
            dry_run: false,
            check: false,
        };
        assert_eq!(resolve_role_pack_kind(&args), RolePackKind::DefaultExample);
    }

    #[test]
    fn quick_config_uses_full_without_roles() {
        let cfg = quick_project_config("q");
        assert_eq!(cfg.backends.llm, BackendImpl::Remote);
        assert!(!cfg.monolith_enabled);
        assert_eq!(cfg.role_pack_kind, RolePackKind::None);
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
            quick: false,
            skip_role_pack: true,
            with_role_pack: None,
            with_example_plugin: false,
            kernel_source: None,
            author: None,
            license: None,
            description: None,
            template_url: None,
            tui: false,
            smart: false,
            no_smart: true,
            pipeline: PipelineArg::Default,
            weld_modules: vec![],
            from_existing: None,
            share: false,
            json: false,
            dry_run: false,
            check: false,
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

    #[test]
    fn pipeline_memory_last_llm_before_memory_in_order_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut cfg = preset_config("pipe", "minimal");
        cfg.pipeline = PipelineArg::MemoryLast;
        cfg.monolith_enabled = false;
        crate::generator::write_project(&cfg, dir.path()).unwrap();
        let raw = std::fs::read_to_string(dir.path().join("src/oclive_pipeline_order.rs")).unwrap();
        let llm = raw.find("llm_generate").expect("llm_generate");
        let mem = raw.find("memory_rank").expect("memory_rank");
        assert!(
            llm < mem,
            "memory-last: llm before memory in OCLIVE_PIPELINE_STEPS"
        );
    }

    #[test]
    fn pipeline_emotion_first_memory_before_event() {
        let steps = PipelineArg::EmotionFirst.steps();
        let em = steps
            .iter()
            .position(|s| *s == "user_emotion_analyze")
            .unwrap();
        let ev = steps.iter().position(|s| *s == "event_estimate").unwrap();
        let mem = steps.iter().position(|s| *s == "memory_rank").unwrap();
        assert!(em < ev);
        assert!(mem < ev);
    }
}