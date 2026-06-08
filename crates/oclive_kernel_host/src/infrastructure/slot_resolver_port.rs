//! Host `impl SlotRegistryResolver for SlotResolver` (registry type lives in infrastructure).

use crate::domain::slot_resolver::{ResolvedRoleSlots, SlotResolver};
use crate::infrastructure::backend_registry::BackendRegistry;
use oclive_kernel_contracts::SlotRegistryResolver;
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
