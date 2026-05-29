//! Init presets, template defaults, and preset builders.

use super::project_config::ChatStorageBackend;
use super::project_config::{
    default_storage_location, BackendImpl, BackendSlots, FeatureSelection, PluginSelection,
    ProjectConfig, ProjectType,
};
use super::InitArgs;
use serde::{Deserialize, Serialize};

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

#[derive(clap::ValueEnum, Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum MonolithPresetArg {
    Latency,
    Memory,
    Embedded,
}

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

fn backend_slots(llm: BackendImpl, agent: BackendImpl, complex_emotion: BackendImpl) -> BackendSlots {
    BackendSlots {
        memory: BackendImpl::Builtin,
        emotion: BackendImpl::Builtin,
        event: BackendImpl::Builtin,
        prompt: BackendImpl::Builtin,
        llm,
        agent,
        complex_emotion,
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
    let backends = if preset == "full" {
        backend_slots(
            BackendImpl::Remote,
            BackendImpl::Builtin,
            BackendImpl::Remote,
        )
    } else if preset == "mixed" {
        backend_slots(
            BackendImpl::Ollama,
            BackendImpl::Builtin,
            BackendImpl::Builtin,
        )
    } else {
        backend_slots(BackendImpl::Ollama, BackendImpl::None, BackendImpl::None)
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
        project_type: ProjectType::KernelServer,
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
        dual_core_enabled: false,
        chat_storage_backend: ChatStorageBackend::default(),
        chat_storage_location: default_storage_location(),
    }
}

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
