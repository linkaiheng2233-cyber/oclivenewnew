//! Agent backend sanitization and validation helpers.

use crate::plugin_backends::{AgentBackend, PluginBackends};

/// Result of [`sanitize_unimplemented_agent_backend`].
#[derive(Debug, Clone, Default)]
pub struct AgentBackendSanitizeResult {
    pub backends: PluginBackends,
    pub warnings: Vec<String>,
}

/// Agent `directory` / `remote` backends are not implemented yet; downgrade to `builtin`.
#[must_use]
pub fn sanitize_unimplemented_agent_backend(
    backends: PluginBackends,
) -> AgentBackendSanitizeResult {
    let mut warnings = Vec::new();
    let mut out = backends;
    if matches!(out.agent, AgentBackend::Directory | AgentBackend::Remote) {
        warnings.push(format!(
            "plugin_backends.agent={} 尚未实现，已降级为 builtin",
            agent_backend_label(out.agent)
        ));
        out.agent = AgentBackend::Builtin;
        out.directory_plugins.agent = None;
    }
    AgentBackendSanitizeResult {
        backends: out,
        warnings,
    }
}

fn agent_backend_label(b: AgentBackend) -> &'static str {
    match b {
        AgentBackend::Builtin => "builtin",
        AgentBackend::Remote => "remote",
        AgentBackend::Directory => "directory",
    }
}

/// Returns a validation error when `plugin_backends.agent` is an unimplemented backend.
#[must_use]
pub fn validate_implemented_agent_backend(backends: &PluginBackends) -> Option<String> {
    if matches!(
        backends.agent,
        AgentBackend::Directory | AgentBackend::Remote
    ) {
        Some(format!(
            "settings.json plugin_backends.agent={} 尚未实现（请使用 builtin）",
            agent_backend_label(backends.agent)
        ))
    } else {
        None
    }
}

/// Collect validation warnings for agent slots in a blueprint `slot_registry`.
#[must_use]
pub fn validate_agent_slot_backends(
    registry: &std::collections::BTreeMap<String, crate::blueprint_v2::SlotRegistryEntry>,
) -> Vec<String> {
    use crate::blueprint_v2::slot_registry_instances_sorted;
    let mut warnings = Vec::new();
    for (key, entry) in slot_registry_instances_sorted(registry, "agent") {
        let backend = entry.backend.trim().to_ascii_lowercase();
        if backend == "directory" || backend == "remote" {
            warnings.push(format!(
                "slot_registry.{key} agent backend={backend} 尚未实现（请使用 builtin）"
            ));
        }
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin_backends::DirectoryPluginSlots;

    #[test]
    fn validate_implemented_agent_backend_rejects_remote() {
        let pb = PluginBackends {
            agent: AgentBackend::Remote,
            ..Default::default()
        };
        let err = validate_implemented_agent_backend(&pb).expect("err");
        assert!(err.contains("remote"));
    }

    #[test]
    fn agent_directory_downgrades_to_builtin() {
        let pb = PluginBackends {
            agent: AgentBackend::Directory,
            directory_plugins: DirectoryPluginSlots {
                agent: Some("my-agent".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        let res = sanitize_unimplemented_agent_backend(pb);
        assert_eq!(res.backends.agent, AgentBackend::Builtin);
        assert!(res.backends.directory_plugins.agent.is_none());
        assert_eq!(res.warnings.len(), 1);
    }

    #[test]
    fn agent_builtin_unchanged() {
        let pb = PluginBackends::default();
        let res = sanitize_unimplemented_agent_backend(pb);
        assert!(res.warnings.is_empty());
        assert_eq!(res.backends.agent, AgentBackend::Builtin);
    }
}
