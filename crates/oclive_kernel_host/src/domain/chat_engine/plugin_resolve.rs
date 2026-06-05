//! Session-level plugin resolution: `chat_engine` binds via [`PluginHostPort`], not the concrete [`PluginHost`] type.

use crate::domain::plugin_host::ResolvedRolePlugins;
use crate::domain::ports::PluginHostPort;
use crate::models::{PluginBackends, Role};
use oclive_validation::SlotRegistryEntry;
use std::collections::BTreeMap;

/// Resolves plugin handles from role pack and session-effective `slot_registry` (falls back to pack defaults when no session).
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
