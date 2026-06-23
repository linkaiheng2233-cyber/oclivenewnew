//! Plugin host resolution port: `chat_engine` depends on this trait without binding to a concrete `PluginHost` type.

use oclive_kernel_types::{PluginBackends, PluginBackendsOverride, Role, SlotRegistryEntry};
use std::collections::BTreeMap;

/// Resolves each module's implementation handle from the role pack / session-effective backends.
///
/// ## When to implement
///
/// - **Who**: runtimes that need to turn a `Role` into `ResolvedRolePlugins`, such as the Tauri desktop host (`PluginHost`) and the headless `oclive_kernel_server`.
/// - **When**: when the orchestration layer (`process_message` / `co_present`) needs to resolve plugin handles per role or session.
///
/// ## When not to implement
///
/// - Unit tests can assemble `ResolvedRolePlugins` by hand without mocking the entire host.
/// - Pure data-structure validation (`oclive_validation`) does not depend on this trait.
///
/// # Examples
///
/// ```no_run
/// use oclive_kernel_contracts::PluginHostPort;
/// use oclive_kernel_types::Role;
///
/// fn resolve_example(host: &impl PluginHostPort, role: &Role) {
///     let resolved = host.resolve_for_role(role);
///     let _ = resolved;
/// }
/// ```
pub trait PluginHostPort: Send + Sync {
    /// A single resolution result (typically `ResolvedRolePlugins` on the host side).
    type Resolved: Clone + Send + Sync + 'static;

    /// Resolves based on `role.plugin_backends` (no session override); policy anchor for six-slot wiring.
    ///
    /// # Errors
    ///
    /// Returns `Err` when the role pack's backend config is invalid, a directory plugin fails to load, or internal I/O fails.
    ///
    /// # Panics
    ///
    /// Does not panic; implementations should return `Result` or capture errors internally.
    fn resolve_for_role(&self, role: &Role) -> Self::Resolved;

    /// Resolves based on the merged effective slots + optional `slot_registry` and overrides (session policy merge).
    ///
    /// # Errors
    ///
    /// Returns `Err` when the `effective` / `slot_registry` / `backend_override` combination is invalid or a plugin handle cannot be resolved.
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn resolve_for_effective_backends(
        &self,
        effective: &PluginBackends,
        slot_registry: Option<&BTreeMap<String, SlotRegistryEntry>>,
        backend_override: Option<&PluginBackendsOverride>,
    ) -> Self::Resolved;
}
