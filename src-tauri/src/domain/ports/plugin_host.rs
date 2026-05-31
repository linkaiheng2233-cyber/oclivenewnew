//! Plugin host resolution port (definition in [`oclive_kernel_contracts::PluginHostPort`]).

pub use oclive_kernel_runtime::PluginHostPort;

use crate::domain::plugin_host::{PluginHost, ResolvedRolePlugins};
use crate::models::{PluginBackends, PluginBackendsOverride, Role};
use oclive_validation::SlotRegistryEntry;
use std::collections::BTreeMap;

impl PluginHostPort for PluginHost {
    type Resolved = ResolvedRolePlugins;

    fn resolve_for_role(&self, role: &Role) -> Self::Resolved {
        PluginHost::resolve_for_role(self, role)
    }

    fn resolve_for_effective_backends(
        &self,
        effective: &PluginBackends,
        slot_registry: Option<&BTreeMap<String, SlotRegistryEntry>>,
        backend_override: Option<&PluginBackendsOverride>,
    ) -> Self::Resolved {
        PluginHost::resolve_for_effective_backends(
            self,
            effective,
            slot_registry,
            backend_override,
        )
    }
}
