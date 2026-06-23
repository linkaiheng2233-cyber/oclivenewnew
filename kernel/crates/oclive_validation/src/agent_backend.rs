//! Agent backend sanitization and validation helpers.

use crate::plugin_backends::{AgentBackend, PluginBackends};

/// Result of [`sanitize_unimplemented_agent_backend`].
#[derive(Debug, Clone, Default)]
pub struct AgentBackendSanitizeResult {
    pub backends: PluginBackends,
    pub warnings: Vec<String>,
}

/// Pass-through: agent remote/directory are implemented (host-orchestrated MCP bridge).
#[must_use]
pub fn sanitize_unimplemented_agent_backend(
    backends: PluginBackends,
) -> AgentBackendSanitizeResult {
    AgentBackendSanitizeResult {
        backends,
        warnings: Vec::new(),
    }
}

fn agent_backend_label(b: AgentBackend) -> &'static str {
    match b {
        AgentBackend::Builtin => "builtin",
        AgentBackend::Remote => "remote",
        AgentBackend::Directory => "directory",
        AgentBackend::None => "none",
    }
}

/// Returns a validation error when `plugin_backends.agent` is invalid.
#[must_use]
pub fn validate_implemented_agent_backend(backends: &PluginBackends) -> Option<String> {
    let _ = agent_backend_label(backends.agent);
    None
}

/// Collect validation warnings for agent slots in a blueprint `slot_registry`.
#[must_use]
pub fn validate_agent_slot_backends(
    _registry: &std::collections::BTreeMap<String, crate::blueprint_v2::SlotRegistryEntry>,
) -> Vec<String> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin_backends::DirectoryPluginSlots;

    #[test]
    fn validate_implemented_agent_backend_allows_remote() {
        let pb = PluginBackends {
            agent: AgentBackend::Remote,
            ..Default::default()
        };
        assert!(validate_implemented_agent_backend(&pb).is_none());
    }

    #[test]
    fn agent_directory_keeps_backend_without_warning() {
        let pb = PluginBackends {
            agent: AgentBackend::Directory,
            directory_plugins: DirectoryPluginSlots {
                agent: Some("my-agent".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        let res = sanitize_unimplemented_agent_backend(pb);
        assert_eq!(res.backends.agent, AgentBackend::Directory);
        assert_eq!(
            res.backends.directory_plugins.agent.as_deref(),
            Some("my-agent")
        );
        assert!(res.warnings.is_empty());
    }

    #[test]
    fn agent_builtin_unchanged() {
        let pb = PluginBackends::default();
        let res = sanitize_unimplemented_agent_backend(pb);
        assert!(res.warnings.is_empty());
        assert_eq!(res.backends.agent, AgentBackend::Builtin);
    }
}
