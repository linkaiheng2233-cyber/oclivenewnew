//! Merge role/session plugin backends with distro host ceiling (P4).

use crate::domain::host_profile::HostProfile;
use crate::models::plugin_backends::PluginBackends;
pub use oclive_kernel_runtime::domain::plugin_resolution::{
    apply_host_ceiling, HostBackendCeiling,
};

/// Apply host ceiling: when `distro.oclive.toml` declares `[plugin_backends]`, those values cap the role pack.
#[must_use]
#[allow(dead_code)] // host_profile unit test + future host callers
pub fn apply_host_ceiling_from_profile(
    role: &PluginBackends,
    host: &HostProfile,
) -> PluginBackends {
    apply_host_ceiling(
        role,
        &HostBackendCeiling {
            skip_agent: host.skip_agent,
            backends_ceiling: host.backends_ceiling.clone(),
        },
    )
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
        let out = apply_host_ceiling_from_profile(&role, &host);
        assert_eq!(out.agent, AgentBackend::None);
    }
}
