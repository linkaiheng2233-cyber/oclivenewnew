//! Blueprint slot resolution port (definition in [`oclive_kernel_contracts::SlotRegistryResolver`]).

pub use oclive_kernel_runtime::SlotRegistryResolver;

use crate::domain::plugin_host::BackendRegistry;
use crate::domain::slot_resolver::{ResolvedRoleSlots, SlotResolver};
use oclive_validation::SlotRegistryEntry;
use std::collections::BTreeMap;

impl SlotRegistryResolver for SlotResolver {
    type Registry = BackendRegistry;
    type ResolvedSlots = ResolvedRoleSlots;

    fn resolve(
        &self,
        registry: &Self::Registry,
        slot_registry: &BTreeMap<String, SlotRegistryEntry>,
    ) -> Self::ResolvedSlots {
        SlotResolver::resolve(registry, slot_registry)
    }
}
