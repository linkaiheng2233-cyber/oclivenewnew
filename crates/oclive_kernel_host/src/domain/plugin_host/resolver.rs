use crate::domain::slot_resolver::SlotResolver;
use crate::models::{PluginBackends, PluginBackendsOverride};
use oclive_validation::SlotRegistryEntry;
use std::collections::BTreeMap;

use oclive_kernel_contracts::PluginBackendRegistryPort;

use super::ResolvedRolePlugins;

/// Resolve layer: merges role pack default backends with optional session override into effective backends and binds implementations.
pub struct PluginResolver;

impl PluginResolver {
    pub(crate) fn resolve(
        registry: &dyn PluginBackendRegistryPort,
        role_backends: &PluginBackends,
        session_override: Option<&PluginBackendsOverride>,
        slot_registry: Option<&BTreeMap<String, SlotRegistryEntry>>,
    ) -> ResolvedRolePlugins {
        let merged_effective = session_override.map(|ov| ov.apply_to(role_backends));
        let effective = merged_effective.as_ref().unwrap_or(role_backends);
        let mut agent = registry.agent_for_plugin_backends(effective);
        let mut complex_emotion = registry.pick_complex_emotion_winner(
            slot_registry.unwrap_or(&BTreeMap::new()),
        );
        let mut slots = None;
        let mut merged_agent_directory_plugin_ids = Vec::new();
        if let Some(reg) = slot_registry {
            slots = Some(SlotResolver::resolve_with_session_backends(
                registry,
                reg,
                Some(effective),
            ));
            complex_emotion = registry.pick_complex_emotion_winner(reg);
            // Agent: merge tool sets from multiple directory instances (parallel semantics at assembly layer, not SlotRunner)
            agent = SlotResolver::wrap_agent_if_merged(agent, reg);
            merged_agent_directory_plugin_ids =
                oclive_validation::merged_agent_directory_plugin_ids(reg);
        }
        ResolvedRolePlugins {
            memory: registry.memory_retrieval_for_plugin_backends(effective),
            emotion: registry.user_emotion_analyzer_for_backends(effective),
            event: registry.event_estimator_for_backends(effective),
            prompt: registry.prompt_assembler_for_backends(effective),
            llm: registry.llm_for_plugin_backends(effective),
            agent,
            complex_emotion,
            slots,
            merged_agent_directory_plugin_ids,
        }
    }
}
