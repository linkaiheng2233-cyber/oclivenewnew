//! 会话级 `plugin_backends` 覆盖：单模块写入、整块覆盖、作者建议一键应用。

use crate::domain::chat_engine::conversation_state_role_id;
use crate::domain::role_info_snapshot::get_role_info_snapshot;
use crate::error::{AppError, Result};
use crate::models::dto::{RoleInfo, SetSessionPluginBackendRequest};
use crate::models::plugin_backends::{
    AgentBackend, ComplexEmotionBackend, EmotionBackend, EventBackend, LlmBackend, MemoryBackend,
    PluginBackendsOverride, PromptBackend,
};
use crate::state::KernelAppState;
use serde::de::DeserializeOwned;
use serde_json::Value;

fn parse_backend_wire<T: DeserializeOwned>(module: &str, value: &str) -> Result<T> {
    let t = value.trim();
    if t.is_empty() {
        return Err(AppError::InvalidParameter(format!(
            "session backend override: module={} backend 不能为空",
            module
        )));
    }
    serde_json::from_value::<T>(Value::String(t.to_string())).map_err(|_| {
        AppError::InvalidParameter(format!(
            "session backend override: module={} backend={} 非法",
            module, t
        ))
    })
}

pub async fn set_session_plugin_backend(
    state: &KernelAppState,
    req: &SetSessionPluginBackendRequest,
) -> Result<RoleInfo> {
    state.load_role_cached(&req.role_id)?;
    let ns = conversation_state_role_id(&req.role_id, req.session_id.as_deref());
    state.db_manager.ensure_role_runtime(ns.as_str()).await?;
    let mut next = state
        .session_backend_override(ns.as_str())
        .unwrap_or_default();
    let module = req.module.trim().to_ascii_lowercase();
    if req.local_memory_provider_id.is_some() && module.as_str() != "memory" {
        return Err(AppError::InvalidParameter(
            "local_memory_provider_id only supports module=memory".to_string(),
        ));
    }
    if req.directory_plugin_id.is_some() && module.as_str() != "llm" {
        return Err(AppError::InvalidParameter(
            "directory_plugin_id only supports module=llm (for now)".to_string(),
        ));
    }
    match module.as_str() {
        "memory" => {
            if let Some(backend) = req.backend.as_ref() {
                next.memory = backend
                    .as_deref()
                    .map(|v| parse_backend_wire::<MemoryBackend>("memory", v))
                    .transpose()?;
            }
            if let Some(provider_id) = req.local_memory_provider_id.as_ref() {
                let t = provider_id.trim();
                if t.is_empty() {
                    next.local_memory_provider_id = None;
                } else {
                    next.local_memory_provider_id = Some(t.to_string());
                }
            }
        }
        "emotion" => {
            if let Some(backend) = req.backend.as_ref() {
                next.emotion = backend
                    .as_deref()
                    .map(|v| parse_backend_wire::<EmotionBackend>("emotion", v))
                    .transpose()?;
            }
        }
        "event" => {
            if let Some(backend) = req.backend.as_ref() {
                next.event = backend
                    .as_deref()
                    .map(|v| parse_backend_wire::<EventBackend>("event", v))
                    .transpose()?;
            }
        }
        "prompt" => {
            if let Some(backend) = req.backend.as_ref() {
                next.prompt = backend
                    .as_deref()
                    .map(|v| parse_backend_wire::<PromptBackend>("prompt", v))
                    .transpose()?;
            }
        }
        "llm" => {
            if let Some(backend) = req.backend.as_ref() {
                next.llm = backend
                    .as_deref()
                    .map(|v| parse_backend_wire::<LlmBackend>("llm", v))
                    .transpose()?;
            }
            if let Some(pid) = req.directory_plugin_id.as_ref() {
                if let Some(Some(raw)) = req.backend.as_ref() {
                    if !raw.trim().eq_ignore_ascii_case("directory") {
                        return Err(AppError::InvalidParameter(
                            "directory_plugin_id requires backend=directory".to_string(),
                        ));
                    }
                }
                let t = pid.trim();
                if t.is_empty() {
                    return Err(AppError::InvalidParameter(
                        "directory_plugin_id cannot be empty".to_string(),
                    ));
                }
                let mut slots = next.directory_plugins.take().unwrap_or_default();
                slots.llm = Some(t.to_string());
                next.directory_plugins = Some(slots);
            }
        }
        "agent" => {
            if let Some(backend) = req.backend.as_ref() {
                next.agent = backend
                    .as_deref()
                    .map(|v| parse_backend_wire::<AgentBackend>("agent", v))
                    .transpose()?;
            }
        }
        "complex_emotion" => {
            if let Some(backend) = req.backend.as_ref() {
                next.complex_emotion = backend
                    .as_deref()
                    .map(|v| parse_backend_wire::<ComplexEmotionBackend>("complex_emotion", v))
                    .transpose()?;
            }
        }
        _ => {
            return Err(AppError::InvalidParameter(format!(
                "session backend override: unknown module {}",
                req.module
            )));
        }
    }
    if next.is_empty() {
        state.clear_session_backend_override(ns.as_str());
    } else {
        state.set_session_backend_override(ns.as_str(), next);
    }
    get_role_info_snapshot(state, &req.role_id, req.session_id.as_deref()).await
}

pub async fn set_session_plugin_backends_override(
    state: &KernelAppState,
    role_id: &str,
    session_id: Option<&str>,
    override_backends: PluginBackendsOverride,
) -> Result<RoleInfo> {
    let role_id = role_id.trim();
    if role_id.is_empty() {
        return Err(AppError::InvalidParameter("role_id required".into()));
    }
    state.load_role_cached(role_id)?;
    let ns = conversation_state_role_id(role_id, session_id);
    state.db_manager.ensure_role_runtime(ns.as_str()).await?;
    if override_backends.is_empty() {
        state.clear_session_backend_override(ns.as_str());
    } else {
        state.set_session_backend_override(ns.as_str(), override_backends);
    }
    get_role_info_snapshot(state, role_id, session_id).await
}

pub async fn apply_author_suggested_plugin_backends(
    state: &KernelAppState,
    role_id: &str,
    session_id: Option<&str>,
) -> Result<RoleInfo> {
    let role_id = role_id.trim();
    if role_id.is_empty() {
        return Err(AppError::InvalidParameter("role_id required".into()));
    }
    let role = state.storage.load_role(role_id)?;
    let Some(sugg) = role
        .author_pack
        .as_ref()
        .and_then(|a| a.suggested_plugin_backends.as_ref())
        .cloned()
    else {
        return Err(AppError::InvalidParameter(
            "该角色包未提供 author.json suggested_plugin_backends".into(),
        ));
    };
    let ns = conversation_state_role_id(role_id, session_id);
    state.db_manager.ensure_role_runtime(ns.as_str()).await?;
    let ov = PluginBackendsOverride {
        memory: Some(sugg.memory),
        emotion: Some(sugg.emotion),
        event: Some(sugg.event),
        prompt: Some(sugg.prompt),
        llm: Some(sugg.llm),
        agent: Some(sugg.agent),
        complex_emotion: None,
        local_memory_provider_id: sugg.local_memory_provider_id.clone(),
        directory_plugins: Some(sugg.directory_plugins.clone()),
    };
    state.set_session_backend_override(ns.as_str(), ov);
    get_role_info_snapshot(state, role_id, session_id).await
}
