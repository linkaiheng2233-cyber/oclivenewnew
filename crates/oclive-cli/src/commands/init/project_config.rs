//! Project type, backend slots, and `ProjectConfig` builders.

use super::preset_config::{InitTemplateArg, MonolithPresetArg, RolePackKind};
use super::InitArgs;
use serde::{Deserialize, Serialize};

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
