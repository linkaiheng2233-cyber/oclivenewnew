//! Narrow memory backend port (D-PORT-02 · phase 1).

use oclive_kernel_types::{MemoryBackend, PluginBackends};
use std::sync::Arc;

use crate::MemoryRetrieval;

/// Memory slot resolution (`builtin` / `remote` / `directory` / `none`).
pub trait MemoryBackendPort: Send + Sync {
    /// Resolve memory retrieval for effective `plugin_backends`.
    fn memory_retrieval_for_plugin_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn MemoryRetrieval>;

    /// Resolve memory retrieval for a single backend enum.
    fn memory_retrieval(&self, b: MemoryBackend) -> Arc<dyn MemoryRetrieval>;
}
