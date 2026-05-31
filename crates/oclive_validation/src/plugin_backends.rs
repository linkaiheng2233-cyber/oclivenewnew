//! Role pack `settings.json` → `plugin_backends`: matches the host `PluginBackends` serde shape (see `PLUGIN_V1.md`).

use serde::{Deserialize, Serialize};

/// Memory retrieval backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryBackend {
    #[default]
    Builtin,
    BuiltinV2,
    Remote,
    /// Locally registered memory provider (see `_local_plugins`)
    Local,
    /// Directory plugin subprocess JSON-RPC
    Directory,
}

/// User emotion analysis backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmotionBackend {
    #[default]
    Builtin,
    BuiltinV2,
    Remote,
    Directory,
}

/// Event impact estimation backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventBackend {
    #[default]
    Builtin,
    BuiltinV2,
    Remote,
    Directory,
}

/// Prompt assembly backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptBackend {
    #[default]
    Builtin,
    BuiltinV2,
    Remote,
    Directory,
}

/// Agent task orchestration backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentBackend {
    #[default]
    Builtin,
    Remote,
    Directory,
}

/// Main conversation LLM call backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmBackend {
    #[default]
    Ollama,
    Remote,
    Directory,
}

/// Plugin manifest `id` for each module when it uses `*_backend = directory`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirectoryPluginSlots {
    #[serde(default)]
    pub memory: Option<String>,
    #[serde(default)]
    pub emotion: Option<String>,
    #[serde(default)]
    pub event: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub llm: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
}

impl DirectoryPluginSlots {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.memory.as_ref().is_none_or(|s| s.trim().is_empty())
            && self.emotion.as_ref().is_none_or(|s| s.trim().is_empty())
            && self.event.as_ref().is_none_or(|s| s.trim().is_empty())
            && self.prompt.as_ref().is_none_or(|s| s.trim().is_empty())
            && self.llm.as_ref().is_none_or(|s| s.trim().is_empty())
            && self.agent.as_ref().is_none_or(|s| s.trim().is_empty())
    }
}

/// Matches `DiskRoleSettings.plugin_backends` / the runtime `Role.plugin_backends`
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginBackends {
    #[serde(default)]
    pub memory: MemoryBackend,
    #[serde(default)]
    pub local_memory_provider_id: Option<String>,
    #[serde(default)]
    pub emotion: EmotionBackend,
    #[serde(default)]
    pub event: EventBackend,
    #[serde(default)]
    pub prompt: PromptBackend,
    #[serde(default)]
    pub llm: LlmBackend,
    #[serde(default)]
    pub agent: AgentBackend,
    #[serde(default)]
    pub directory_plugins: DirectoryPluginSlots,
}

/// Effective backend source (session / pack default, etc.; for host runtime extension)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginBackendSource {
    #[default]
    PackDefault,
    SessionOverride,
    EnvOverride,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginBackendsSourceMap {
    #[serde(default)]
    pub memory: PluginBackendSource,
    #[serde(default)]
    pub emotion: PluginBackendSource,
    #[serde(default)]
    pub event: PluginBackendSource,
    #[serde(default)]
    pub prompt: PluginBackendSource,
    #[serde(default)]
    pub llm: PluginBackendSource,
    #[serde(default)]
    pub agent: PluginBackendSource,
}

/// Session-level override
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginBackendsOverride {
    #[serde(default)]
    pub memory: Option<MemoryBackend>,
    #[serde(default)]
    pub local_memory_provider_id: Option<String>,
    #[serde(default)]
    pub emotion: Option<EmotionBackend>,
    #[serde(default)]
    pub event: Option<EventBackend>,
    #[serde(default)]
    pub prompt: Option<PromptBackend>,
    #[serde(default)]
    pub llm: Option<LlmBackend>,
    #[serde(default)]
    pub agent: Option<AgentBackend>,
    #[serde(default)]
    pub directory_plugins: Option<DirectoryPluginSlots>,
}

impl PluginBackendsOverride {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.memory.is_none()
            && self.local_memory_provider_id.is_none()
            && self.emotion.is_none()
            && self.event.is_none()
            && self.prompt.is_none()
            && self.llm.is_none()
            && self.agent.is_none()
            && self.directory_plugins.is_none()
    }

    #[must_use]
    pub fn apply_to(&self, base: &PluginBackends) -> PluginBackends {
        let local_memory_provider_id = match &self.local_memory_provider_id {
            None => base.local_memory_provider_id.clone(),
            Some(s) => {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            }
        };
        let directory_plugins = match &self.directory_plugins {
            None => base.directory_plugins.clone(),
            Some(ov) => DirectoryPluginSlots {
                memory: trimmed_or_fallback(
                    ov.memory.as_deref(),
                    base.directory_plugins.memory.as_deref(),
                ),
                emotion: trimmed_or_fallback(
                    ov.emotion.as_deref(),
                    base.directory_plugins.emotion.as_deref(),
                ),
                event: trimmed_or_fallback(
                    ov.event.as_deref(),
                    base.directory_plugins.event.as_deref(),
                ),
                prompt: trimmed_or_fallback(
                    ov.prompt.as_deref(),
                    base.directory_plugins.prompt.as_deref(),
                ),
                llm: trimmed_or_fallback(ov.llm.as_deref(), base.directory_plugins.llm.as_deref()),
                agent: trimmed_or_fallback(
                    ov.agent.as_deref(),
                    base.directory_plugins.agent.as_deref(),
                ),
            },
        };
        PluginBackends {
            memory: self.memory.unwrap_or(base.memory),
            local_memory_provider_id,
            emotion: self.emotion.unwrap_or(base.emotion),
            event: self.event.unwrap_or(base.event),
            prompt: self.prompt.unwrap_or(base.prompt),
            llm: self.llm.unwrap_or(base.llm),
            agent: self.agent.unwrap_or(base.agent),
            directory_plugins,
        }
    }
}

fn trimmed_or_fallback(ov: Option<&str>, base: Option<&str>) -> Option<String> {
    match ov {
        None => base.map(|s| s.to_string()),
        Some(s) => {
            let t = s.trim();
            if t.is_empty() {
                base.map(|x| x.to_string())
            } else {
                Some(t.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_replaces_local_memory_provider_id() {
        let base = PluginBackends {
            memory: MemoryBackend::Local,
            local_memory_provider_id: Some("a".into()),
            ..Default::default()
        };
        let ov = PluginBackendsOverride {
            local_memory_provider_id: Some("b".into()),
            ..Default::default()
        };
        let eff = ov.apply_to(&base);
        assert_eq!(eff.local_memory_provider_id.as_deref(), Some("b"));
        assert_eq!(eff.memory, MemoryBackend::Local);
    }

    #[test]
    fn plugin_backends_size_audit() {
        let n = std::mem::size_of::<PluginBackends>();
        println!("PluginBackends size_of = {n}");
        // Round-12 Opus gate: Arc optimization only if >= 128 bytes.
        assert!(n >= 128, "size {n} below Arc threshold");
    }

    #[test]
    fn override_whitespace_local_memory_provider_id_clears() {
        let base = PluginBackends {
            memory: MemoryBackend::Local,
            local_memory_provider_id: Some("a".into()),
            ..Default::default()
        };
        let ov = PluginBackendsOverride {
            local_memory_provider_id: Some("   ".into()),
            ..Default::default()
        };
        let eff = ov.apply_to(&base);
        assert!(eff.local_memory_provider_id.is_none());
    }
}
