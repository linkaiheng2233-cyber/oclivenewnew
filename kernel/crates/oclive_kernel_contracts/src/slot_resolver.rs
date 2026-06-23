//! Blueprint v2 `slot_registry` multi-instance resolution port.
//!
//! The host-side `SlotResolver` is a stateless struct; implement this trait to plug into the orchestration anti-corruption layer.

use oclive_kernel_types::SlotRegistryEntry;
use std::collections::BTreeMap;

/// Resolves multi-instance plugin handles per `slot_registry` (the implementer provides the registry and the returned slots view type).
///
/// ## When to implement
///
/// - **Who**: the **host runtime** (Tauri's `SlotResolver` already implements it); ordinary plugin authors do **not** implement this trait.
/// - **When**: when a new host (embedded / headless) needs to bind the blueprint registry to a `BackendRegistry`.
///
/// ## When not to implement
///
/// - When writing a directory plugin or Remote service: implement **slot capability** traits such as `MemoryRetrieval` / `LlmClient` instead of this resolver.
pub trait SlotRegistryResolver: Send + Sync {
    /// The host `BackendRegistry` (or an equivalent registry).
    type Registry;
    /// The resolution result (typically `ResolvedRoleSlots` on the host).
    type ResolvedSlots: Clone + Send + Sync + 'static;

    /// Resolves multi-instance plugin handles per `slot_registry` entries.
    ///
    /// # Errors
    ///
    /// None; this method does not return a `Result`.
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn resolve(
        &self,
        registry: &Self::Registry,
        slot_registry: &BTreeMap<String, SlotRegistryEntry>,
    ) -> Self::ResolvedSlots;
}
