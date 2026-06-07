//! Merge role/session plugin backends with distro host ceiling (P4).

use crate::domain::host_profile::HostProfile;
use crate::models::plugin_backends::PluginBackends;

/// Apply host ceiling: when `distro.oclive.toml` declares `[plugin_backends]`, those values cap the role pack.
#[must_use]
pub fn apply_host_ceiling(role: &PluginBackends, host: &HostProfile) -> PluginBackends {
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
        backends.agent = crate::models::plugin_backends::AgentBackend::None;
    }
    backends
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::host_profile::HostProfile;
    use crate::models::plugin_backends::{AgentBackend, PluginBackends};

    #[test]
    fn skip_agent_forces_agent_none() {
        let role = PluginBackends {
            agent: AgentBackend::Builtin,
            ..Default::default()
        };
        let host = HostProfile {
            skip_agent: true,
            ..HostProfile::default()
        };
        let out = apply_host_ceiling(&role, &host);
        assert_eq!(out.agent, AgentBackend::None);
    }
}
