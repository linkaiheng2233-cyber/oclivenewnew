//! Local plugin provider registration port.

use oclive_kernel_types::{LocalPluginCapability, LocalPluginProviderDescriptor};
use std::sync::Arc;

/// Register and list in-process local plugin providers (memory, etc.).
pub trait LocalPluginRegistryPort: Send + Sync {
    /// # Errors
    ///
    /// Returns an error string when the descriptor is invalid or already registered.
    fn register_local_provider(
        &self,
        descriptor: LocalPluginProviderDescriptor,
    ) -> Result<(), String>;

    fn local_providers_for(
        &self,
        capability: LocalPluginCapability,
    ) -> Vec<Arc<LocalPluginProviderDescriptor>>;

    fn local_all_providers(&self) -> Vec<Arc<LocalPluginProviderDescriptor>>;
}
