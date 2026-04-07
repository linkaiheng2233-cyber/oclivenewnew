//! 角色包 `manifest.json` 加载前校验，给出可读中文错误。
//!
//! `validate_disk_manifest` 与共享 crate [`oclive_validation`] 一致。

use crate::infrastructure::remote_plugin::RemotePluginHttpConfig;
use crate::models::plugin_backends::{
    EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PromptBackend,
};
use crate::models::role::Role;
use crate::models::InteractionMode;

pub use oclive_validation::validate_disk_manifest;

/// 校验 `settings.json` 合并后的 `interaction_mode`（若有）。
pub fn validate_role_interaction_mode(role: &Role) -> Result<(), String> {
    InteractionMode::validate_optional_pack_field(role.interaction_mode.as_deref())
}

/// `plugin_backends` 含 `remote` 但未配置对应环境变量时记一条警告（不阻止加载；运行时仍按 PLUGIN_V1 回退）。
pub fn log_plugin_backends_remote_missing_env(role: &Role) {
    let pb = &role.plugin_backends;
    let plugin_url_ok = RemotePluginHttpConfig::from_env_plugin().is_some();
    let llm_url_ok = RemotePluginHttpConfig::from_env_llm().is_some();

    let needs_plugin_url = matches!(pb.memory, MemoryBackend::Remote)
        || matches!(pb.emotion, EmotionBackend::Remote)
        || matches!(pb.event, EventBackend::Remote)
        || matches!(pb.prompt, PromptBackend::Remote);
    let needs_llm_url = matches!(pb.llm, LlmBackend::Remote);

    if needs_plugin_url && !plugin_url_ok {
        log::warn!(
            target: "oclive_plugin",
            "role_id={} plugin_backends 含 remote（memory/emotion/event/prompt 之一），但未设置 OCLIVE_REMOTE_PLUGIN_URL；侧车未启用，相关子系统将使用占位/回退内置",
            role.id
        );
    }
    if needs_llm_url && !llm_url_ok {
        log::warn!(
            target: "oclive_plugin",
            "role_id={} plugin_backends.llm=remote，但未设置 OCLIVE_REMOTE_LLM_URL；将委托进程内默认 LLM",
            role.id
        );
    }
}
