//! Pure six-slot plugin backend resolution (legacy / v2 + session override + env LLM + host ceiling).
//!
//! SSOT for effective `PluginBackends` + `PluginBackendSource` without DB or `AppState`.
//! Host merges session overrides from SQLite; CLI loads pack defaults from disk.

use oclive_validation::{
    effective_slot_registry, sanitize_unimplemented_agent_backend,
    slot_registry_to_plugin_backends, AgentBackend, EmotionBackend, EventBackend, LlmBackend,
    MemoryBackend, PluginBackendSource, PluginBackends, PluginBackendsSourceMap, PromptBackend,
    SlotOverridePatch, SlotRegistryEntry,
};
use std::collections::BTreeMap;

use crate::distro_oclive_file::PluginBackendsToml;

/// Session-scoped namespace key (matches host `conversation_state_role_id`).
#[must_use]
pub fn session_namespace_for_role(manifest_role_id: &str, session_id: Option<&str>) -> String {
    const MAX_SUFFIX_CHARS: usize = 64;
    const MAX_TOTAL_CHARS: usize = 256;

    let sid = session_id.map(str::trim).filter(|s| !s.is_empty());
    match sid {
        None => manifest_role_id.chars().take(MAX_TOTAL_CHARS).collect(),
        Some(s) => {
            let safe: String = s
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                        c
                    } else {
                        '_'
                    }
                })
                .take(MAX_SUFFIX_CHARS)
                .collect();
            let out = format!("{}__sess__{}", manifest_role_id, safe);
            out.chars().take(MAX_TOTAL_CHARS).collect()
        }
    }
}

/// Host ceiling inputs (from `distro.oclive.toml` / `HostProfile`).
#[derive(Debug, Clone, Default)]
pub struct HostBackendCeiling {
    pub skip_agent: bool,
    pub backends_ceiling: Option<PluginBackends>,
}

/// Snapshot inputs for one session resolution pass (no I/O).
#[derive(Debug, Clone)]
pub struct SessionPluginResolutionInput {
    pub pack_plugin_backends: PluginBackends,
    pub pack_slot_registry: Option<BTreeMap<String, SlotRegistryEntry>>,
    pub session_slot_overrides: BTreeMap<String, SlotOverridePatch>,
    /// `cloud` | `local` | other — matches launcher injection semantics.
    pub user_llm_provider: String,
    pub llm_env_override: Option<LlmBackend>,
    pub remote_llm_url_token_configured: bool,
    pub host_ceiling: HostBackendCeiling,
}

/// Effective backends + provenance + sanitize warnings.
#[derive(Debug, Clone)]
pub struct SessionPluginResolution {
    pub backends: PluginBackends,
    pub sources: PluginBackendsSourceMap,
    pub warnings: Vec<String>,
}

/// Parse `OCLIVE_LLM_BACKEND` (`ollama` / `remote` / `directory`).
#[must_use]
pub fn pick_llm_backend_env_override() -> Option<LlmBackend> {
    let Ok(v) = std::env::var("OCLIVE_LLM_BACKEND") else {
        return None;
    };
    let t = v.trim();
    if t.is_empty() {
        return None;
    }
    if t.eq_ignore_ascii_case("ollama") {
        Some(LlmBackend::Ollama)
    } else if t.eq_ignore_ascii_case("remote") {
        Some(LlmBackend::Remote)
    } else if t.eq_ignore_ascii_case("directory") {
        Some(LlmBackend::Directory)
    } else {
        None
    }
}

/// Whether `OCLIVE_REMOTE_LLM_URL` and `OCLIVE_REMOTE_LLM_TOKEN` are both non-empty.
#[must_use]
pub fn remote_llm_url_token_configured() -> bool {
    std::env::var("OCLIVE_REMOTE_LLM_URL")
        .ok()
        .is_some_and(|u| !u.trim().is_empty())
        && std::env::var("OCLIVE_REMOTE_LLM_TOKEN")
            .ok()
            .is_some_and(|t| !t.trim().is_empty())
}

/// Apply host ceiling: distro `[plugin_backends]` caps role/session values; `skip_agent` forces agent none.
#[must_use]
pub fn apply_host_ceiling(role: &PluginBackends, host: &HostBackendCeiling) -> PluginBackends {
    let mut backends = if let Some(ref ceiling) = host.backends_ceiling {
        PluginBackends {
            memory: ceiling.memory,
            local_memory_provider_id: role.local_memory_provider_id.clone(),
            emotion: ceiling.emotion,
            event: ceiling.event,
            prompt: ceiling.prompt,
            llm: ceiling.llm,
            agent: ceiling.agent,
            directory_plugins: role.directory_plugins.clone(),
        }
    } else {
        role.clone()
    };
    if host.skip_agent {
        backends.agent = AgentBackend::None;
    }
    backends
}

/// Resolve effective six-slot backends for a session namespace.
#[must_use]
pub fn resolve_session_plugin_backends(
    input: &SessionPluginResolutionInput,
) -> SessionPluginResolution {
    let slot_registry = input
        .pack_slot_registry
        .as_ref()
        .map(|pack| effective_slot_registry(pack, &input.session_slot_overrides));

    let mut backends = if let Some(ref eff) = slot_registry {
        slot_registry_to_plugin_backends(eff)
    } else {
        input.pack_plugin_backends.clone()
    };

    let provider = input.user_llm_provider.trim().to_ascii_lowercase();
    if provider == "cloud" {
        backends.llm = LlmBackend::Remote;
    } else if provider == "local" {
        backends.llm = LlmBackend::Ollama;
    } else if let Some(llm) = input.llm_env_override {
        backends.llm = llm;
    } else if input.remote_llm_url_token_configured {
        backends.llm = LlmBackend::Remote;
    }

    backends = apply_host_ceiling(&backends, &input.host_ceiling);
    let sanitized = sanitize_unimplemented_agent_backend(backends);

    let mut sources = PluginBackendsSourceMap::default();
    if let Some(reg) = input.pack_slot_registry.as_ref() {
        for key in input.session_slot_overrides.keys() {
            let Some(entry) = reg.get(key) else {
                continue;
            };
            match entry.slot_type.as_str() {
                "memory" => sources.memory = PluginBackendSource::SessionOverride,
                "emotion" => sources.emotion = PluginBackendSource::SessionOverride,
                "event" => sources.event = PluginBackendSource::SessionOverride,
                "prompt" => sources.prompt = PluginBackendSource::SessionOverride,
                "llm" => sources.llm = PluginBackendSource::SessionOverride,
                "agent" => sources.agent = PluginBackendSource::SessionOverride,
                _ => {}
            }
        }
    }
    if sources.llm == PluginBackendSource::PackDefault && input.llm_env_override.is_some() {
        sources.llm = PluginBackendSource::EnvOverride;
    }

    SessionPluginResolution {
        backends: sanitized.backends,
        sources,
        warnings: sanitized.warnings,
    }
}

/// Parse distro `[plugin_backends]` TOML section into a ceiling `PluginBackends`.
///
/// # Errors
///
/// Returns when an unknown backend wire value is encountered.
pub fn plugin_backends_from_distro_toml(pb: &PluginBackendsToml) -> Result<PluginBackends, String> {
    Ok(PluginBackends {
        memory: parse_memory(pb.memory.as_deref())?,
        emotion: parse_emotion(pb.emotion.as_deref())?,
        event: parse_event(pb.event.as_deref())?,
        prompt: parse_prompt(pb.prompt.as_deref())?,
        llm: parse_llm(pb.llm.as_deref())?,
        agent: parse_agent(pb.agent.as_deref())?,
        ..Default::default()
    })
}

/// Build [`HostBackendCeiling`] from parsed `distro.oclive.toml`.
#[must_use]
pub fn host_ceiling_from_distro_file(
    file: &crate::distro_oclive_file::DistroOcliveFile,
) -> HostBackendCeiling {
    let (skip_agent, _) = file.parse_distro_skip_flags();
    let backends_ceiling = file
        .plugin_backends
        .as_ref()
        .and_then(|pb| plugin_backends_from_distro_toml(pb).ok());
    HostBackendCeiling {
        skip_agent,
        backends_ceiling,
    }
}

fn parse_memory(s: Option<&str>) -> Result<MemoryBackend, String> {
    match s {
        None => Ok(MemoryBackend::Builtin),
        Some(v) => match v {
            "builtin" | "builtin_v2" => Ok(MemoryBackend::Builtin),
            "remote" => Ok(MemoryBackend::Remote),
            "local" => Ok(MemoryBackend::Local),
            "directory" => Ok(MemoryBackend::Directory),
            "none" => Ok(MemoryBackend::None),
            other => Err(format!("unknown memory backend: {other}")),
        },
    }
}

fn parse_emotion(s: Option<&str>) -> Result<EmotionBackend, String> {
    match s {
        None => Ok(EmotionBackend::Builtin),
        Some(v) => match v {
            "builtin" | "builtin_v2" => Ok(EmotionBackend::Builtin),
            "remote" => Ok(EmotionBackend::Remote),
            "directory" => Ok(EmotionBackend::Directory),
            "none" => Ok(EmotionBackend::None),
            other => Err(format!("unknown emotion backend: {other}")),
        },
    }
}

fn parse_event(s: Option<&str>) -> Result<EventBackend, String> {
    match s {
        None => Ok(EventBackend::Builtin),
        Some(v) => match v {
            "builtin" | "builtin_v2" => Ok(EventBackend::Builtin),
            "remote" => Ok(EventBackend::Remote),
            "directory" => Ok(EventBackend::Directory),
            "none" => Ok(EventBackend::None),
            other => Err(format!("unknown event backend: {other}")),
        },
    }
}

fn parse_prompt(s: Option<&str>) -> Result<PromptBackend, String> {
    match s {
        None => Ok(PromptBackend::Builtin),
        Some(v) => match v {
            "builtin" | "builtin_v2" => Ok(PromptBackend::Builtin),
            "remote" => Ok(PromptBackend::Remote),
            "directory" => Ok(PromptBackend::Directory),
            "none" => Ok(PromptBackend::None),
            other => Err(format!("unknown prompt backend: {other}")),
        },
    }
}

fn parse_llm(s: Option<&str>) -> Result<LlmBackend, String> {
    match s {
        None => Ok(LlmBackend::Ollama),
        Some(v) => match v {
            "ollama" => Ok(LlmBackend::Ollama),
            "remote" => Ok(LlmBackend::Remote),
            "directory" => Ok(LlmBackend::Directory),
            "none" => Ok(LlmBackend::None),
            other => Err(format!("unknown llm backend: {other}")),
        },
    }
}

fn parse_agent(s: Option<&str>) -> Result<AgentBackend, String> {
    match s {
        None => Ok(AgentBackend::Builtin),
        Some(v) => match v {
            "builtin" | "builtin_v2" => Ok(AgentBackend::Builtin),
            "remote" => Ok(AgentBackend::Remote),
            "directory" => Ok(AgentBackend::Directory),
            "none" => Ok(AgentBackend::None),
            other => Err(format!("unknown agent backend: {other}")),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oclive_validation::SlotRegistryEntry;

    fn v2_memory_override_input() -> SessionPluginResolutionInput {
        let mut reg = BTreeMap::new();
        reg.insert(
            "memory.main".into(),
            SlotRegistryEntry {
                slot_type: "memory".into(),
                label: "Memory".into(),
                backend: "builtin".into(),
                position: 0,
                plugin: None,
                plugins: None,
                model: None,
                url: None,
                local_memory_provider_id: None,
                zone: None,
                policy: None,
            },
        );
        let mut ov = BTreeMap::new();
        ov.insert(
            "memory.main".into(),
            SlotOverridePatch {
                backend: Some("remote".into()),
                ..Default::default()
            },
        );
        SessionPluginResolutionInput {
            pack_plugin_backends: PluginBackends::default(),
            pack_slot_registry: Some(reg),
            session_slot_overrides: ov,
            user_llm_provider: String::new(),
            llm_env_override: None,
            remote_llm_url_token_configured: false,
            host_ceiling: HostBackendCeiling::default(),
        }
    }

    #[test]
    fn session_override_sets_memory_remote_and_source() {
        let out = resolve_session_plugin_backends(&v2_memory_override_input());
        assert_eq!(out.backends.memory, MemoryBackend::Remote);
        assert_eq!(out.sources.memory, PluginBackendSource::SessionOverride);
    }

    #[test]
    fn host_ceiling_caps_llm_when_declared() {
        let mut input = v2_memory_override_input();
        input.host_ceiling.backends_ceiling = Some(PluginBackends {
            llm: LlmBackend::Ollama,
            ..Default::default()
        });
        input.user_llm_provider = "cloud".into();
        let out = resolve_session_plugin_backends(&input);
        assert_eq!(out.backends.llm, LlmBackend::Ollama);
    }

    #[test]
    fn env_llm_override_surfaces_in_sources() {
        let mut input = v2_memory_override_input();
        input.llm_env_override = Some(LlmBackend::Remote);
        let out = resolve_session_plugin_backends(&input);
        assert_eq!(out.backends.llm, LlmBackend::Remote);
        assert_eq!(out.sources.llm, PluginBackendSource::EnvOverride);
    }
}
