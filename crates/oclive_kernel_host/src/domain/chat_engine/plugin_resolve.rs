//! Session-level plugin resolution: `chat_engine` binds via [`PluginHostPort`], not the concrete [`PluginHost`] type.

use crate::domain::plugin_host::ResolvedRolePlugins;
use crate::domain::ports::PluginHostPort;
use crate::models::{PluginBackends, Role};
use oclive_validation::SlotRegistryEntry;
use std::collections::BTreeMap;

/// Session namespace → role defaults or session-effective six-slot backends (orchestration policy).
#[must_use]
pub fn resolve_plugins_for_session(
    host: &dyn PluginHostPort<Resolved = ResolvedRolePlugins>,
    role: &Role,
    session_namespace: Option<&str>,
    effective_backends: &PluginBackends,
    slot_registry: Option<&BTreeMap<String, SlotRegistryEntry>>,
) -> ResolvedRolePlugins {
    if session_namespace
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .is_none()
    {
        return host.resolve_for_role(role);
    }
    host.resolve_for_effective_backends(effective_backends, slot_registry, None)
}
