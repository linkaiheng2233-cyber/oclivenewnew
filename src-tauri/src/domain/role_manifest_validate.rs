//! Pre-load validation for role pack `manifest.json` with human-readable errors.
//!
//! `validate_disk_manifest` matches the shared crate [`oclive_validation`].

use crate::infrastructure::remote_plugin::RemotePluginHttpConfig;
use crate::models::plugin_backends::{
    EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PromptBackend,
};
use crate::models::role::Role;
use crate::models::InteractionMode;

pub use oclive_validation::validate_disk_manifest;
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// Validate merged `interaction_mode` from `settings.json` when present.
pub fn validate_role_interaction_mode(role: &Role) -> Result<(), String> {
    InteractionMode::validate_optional_pack_field(role.interaction_mode.as_deref())
}

/// Log a warning when `plugin_backends` includes `remote` but env vars are unset (does not block load; runtime still falls back per PLUGIN_V1).
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
        tracing::warn!(
            target: "oclive_plugin",
            "role_id={} plugin_backends 含 remote（memory/emotion/event/prompt 之一），但未设置 OCLIVE_REMOTE_PLUGIN_URL；侧车未启用，相关子系统将使用占位/回退内置",
            role.id
        );
    }
    if needs_llm_url && !llm_url_ok {
        tracing::warn!(
            target: "oclive_plugin",
            "role_id={} plugin_backends.llm=remote，但未设置 OCLIVE_REMOTE_LLM_URL；将委托进程内默认 LLM",
            role.id
        );
    }

    let needs_dir_memory = matches!(pb.memory, MemoryBackend::Directory);
    let needs_dir_emotion = matches!(pb.emotion, EmotionBackend::Directory);
    let needs_dir_event = matches!(pb.event, EventBackend::Directory);
    let needs_dir_prompt = matches!(pb.prompt, PromptBackend::Directory);
    let needs_dir_llm = matches!(pb.llm, LlmBackend::Directory);
    let slots = &pb.directory_plugins;
    if needs_dir_memory && slots.memory.as_ref().is_none_or(|s| s.trim().is_empty()) {
        tracing::warn!(
            target: "oclive_plugin",
            "role_id={} plugin_backends.memory=directory 但未配置 directory_plugins.memory；运行时回退 builtin",
            role.id
        );
    }
    if needs_dir_emotion && slots.emotion.as_ref().is_none_or(|s| s.trim().is_empty()) {
        tracing::warn!(
            target: "oclive_plugin",
            "role_id={} plugin_backends.emotion=directory 但未配置 directory_plugins.emotion；运行时回退 builtin",
            role.id
        );
    }
    if needs_dir_event && slots.event.as_ref().is_none_or(|s| s.trim().is_empty()) {
        tracing::warn!(
            target: "oclive_plugin",
            "role_id={} plugin_backends.event=directory 但未配置 directory_plugins.event；运行时回退 builtin",
            role.id
        );
    }
    if needs_dir_prompt && slots.prompt.as_ref().is_none_or(|s| s.trim().is_empty()) {
        tracing::warn!(
            target: "oclive_plugin",
            "role_id={} plugin_backends.prompt=directory 但未配置 directory_plugins.prompt；运行时回退 builtin",
            role.id
        );
    }
    if needs_dir_llm && slots.llm.as_ref().is_none_or(|s| s.trim().is_empty()) {
        tracing::warn!(
            target: "oclive_plugin",
            "role_id={} plugin_backends.llm=directory 但未配置 directory_plugins.llm；运行时回退 Ollama",
            role.id
        );
    }
}
