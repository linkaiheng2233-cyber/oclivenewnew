//! # Blueprint `slot_registry` → executable slot list
//!
//! **Role**: reads validated `slot_registry` from the role pack (`pipeline.ocblueprint`), binds each instance to a concrete implementation in the backend registry port, and produces [`ResolvedRoleSlots`] for [`SlotRunner`](super::slot_runner::SlotRunner) merge execution.

use crate::domain::agent::AgentProvider;
use crate::domain::complex_emotion::ComplexEmotionProvider;
use crate::domain::event_estimator::EventEstimator;
use crate::domain::memory_retrieval::MemoryRetrieval;
use crate::domain::ports::LlmClient;
use crate::domain::prompt_assembler::PromptAssembler;
use crate::domain::user_emotion_analyzer::UserEmotionAnalyzer;
use crate::models::PluginBackends;
use oclive_kernel_contracts::SlotBackendFactoryPort;
use oclive_validation::{
    plugin_backends_for_slot_entry, slot_registry_instances_sorted, SlotRegistryEntry,
};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Multi-LLM instance merge policy (`policy` field on `slot_registry` entries).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LlmMergePolicy {
    #[default]
    Ensemble,
    Fastest,
    Fallback,
}

impl LlmMergePolicy {
    #[must_use]
    pub fn parse(raw: Option<&str>) -> Self {
        match raw.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
            Some("fastest") => Self::Fastest,
            Some("fallback") => Self::Fallback,
            Some("ensemble") => Self::Ensemble,
            _ => Self::Ensemble,
        }
    }
}

/// Multi-instance resolution grouped by `type` (ascending `position`).
#[derive(Clone, Default)]
pub struct ResolvedRoleSlots {
    pub memory: Vec<(String, Arc<dyn MemoryRetrieval>)>,
    pub emotion: Vec<(String, Arc<dyn UserEmotionAnalyzer>)>,
    pub event: Vec<(String, Arc<dyn EventEstimator>)>,
    pub prompt: Vec<(String, Arc<dyn PromptAssembler>)>,
    pub llm: Vec<(String, Arc<dyn LlmClient>)>,
    pub agent: Vec<(String, Arc<dyn AgentProvider>)>,
    pub complex_emotion: Vec<(String, Arc<dyn ComplexEmotionProvider>)>,
    /// Multi-LLM merge policy (from `policy` on the llm slot with largest `position`).
    pub llm_merge_policy: LlmMergePolicy,
}

pub struct SlotResolver;

impl SlotResolver {
    /// Maps validated `slot_registry` to `Arc<dyn …>` instance lists (bucketed by type, sorted by `position` within each bucket).
    #[must_use]
    pub fn resolve(
        registry: &dyn SlotBackendFactoryPort,
        slot_registry: &BTreeMap<String, SlotRegistryEntry>,
    ) -> ResolvedRoleSlots {
        Self::resolve_with_session_backends(registry, slot_registry, None)
    }

/// Resolves `slot_registry`; `session_effective_backends` overrides blueprint `llm` slot `backend` (session policy merge).
    #[must_use]
    pub fn resolve_with_session_backends(
        registry: &dyn SlotBackendFactoryPort,
        slot_registry: &BTreeMap<String, SlotRegistryEntry>,
        session_effective_backends: Option<&PluginBackends>,
    ) -> ResolvedRoleSlots {
        let mut out = ResolvedRoleSlots::default();
        for (key, entry) in slot_registry_instances_sorted(slot_registry, "memory") {
            let pb = plugin_backends_for_slot_entry(&entry);
            out.memory
                .push((key, registry.memory_retrieval_for_plugin_backends(&pb)));
        }
        for (key, entry) in slot_registry_instances_sorted(slot_registry, "emotion") {
            let pb = plugin_backends_for_slot_entry(&entry);
            out.emotion
                .push((key, registry.user_emotion_analyzer_for_backends(&pb)));
        }
        for (key, entry) in slot_registry_instances_sorted(slot_registry, "event") {
            let pb = plugin_backends_for_slot_entry(&entry);
            out.event
                .push((key, registry.event_estimator_for_backends(&pb)));
        }
        for (key, entry) in slot_registry_instances_sorted(slot_registry, "prompt") {
            let pb = plugin_backends_for_slot_entry(&entry);
            out.prompt
                .push((key, registry.prompt_assembler_for_backends(&pb)));
        }
        for (key, entry) in slot_registry_instances_sorted(slot_registry, "llm") {
            out.llm_merge_policy = LlmMergePolicy::parse(entry.policy.as_deref());
            let mut pb = plugin_backends_for_slot_entry(&entry);
            if let Some(eff) = session_effective_backends {
                pb.llm = eff.llm;
            }
            out.llm.push((key, registry.llm_for_plugin_backends(&pb)));
        }
        for (key, entry) in slot_registry_instances_sorted(slot_registry, "agent") {
            let pb = plugin_backends_for_slot_entry(&entry);
            out.agent
                .push((key, registry.agent_for_plugin_backends(&pb)));
        }
        for (key, entry) in slot_registry_instances_sorted(slot_registry, "complex_emotion") {
            out.complex_emotion.push((
                key.clone(),
                registry.pick_complex_emotion_for_entry(&entry),
            ));
        }
        out
    }

    /// Agent directory multi-instance merge is not implemented; returns `inner` unchanged.
    #[must_use]
    pub fn wrap_agent_if_merged(
        inner: Arc<dyn AgentProvider>,
        _slot_registry: &BTreeMap<String, SlotRegistryEntry>,
    ) -> Arc<dyn AgentProvider> {
        inner
    }
}
