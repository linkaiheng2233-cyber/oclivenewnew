//! Local plugin discovery bridge trait.

use oclive_kernel_types::LocalPluginProviderDescriptor;

/// Provider discovery bridge interface (may later be backed by WASM / Native Process implementations).
///
/// ## When to implement
///
/// - **Who**: the host's local plugin runtime (scanning the `plugins/` directory, WASM / subprocess bridge).
/// - **When**: when directory plugins need to be discovered and registered as `LocalPluginProviderDescriptor`.
///
/// ## When not to implement
///
/// - When only builtin / Remote slots are used and directory plugins are not enabled.
pub trait LocalPluginBridge: Send + Sync {
    /// Returns the bridge implementation name (for diagnostics and logging).
    ///
    /// # Errors
    ///
    /// None; this method does not return a `Result`.
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn bridge_name(&self) -> &'static str;

    /// Scans and returns the list of available local plugin Provider descriptors.
    ///
    /// # Errors
    ///
    /// None; this method does not return a `Result`; on discovery failure the implementation should return an empty list or skip invalid entries.
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn discover_providers(&self) -> Vec<LocalPluginProviderDescriptor>;
}
