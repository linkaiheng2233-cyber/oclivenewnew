//! Six-slot backend factory port — used by [`SlotResolver`](crate::slot_resolver::SlotRegistryResolver) and plugin resolve paths.

use oclive_kernel_types::{
    AgentBackend, EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends,
    PromptBackend, SlotRegistryEntry,
};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::{
    AgentProvider, ComplexEmotionProvider, EventEstimator, LlmClient, MemoryRetrieval,
    PromptAssembler, UserEmotionAnalyzer,
};

/// Resolves `plugin_backends` / per-slot registry entries → executable trait handles.
pub trait SlotBackendFactoryPort: Send + Sync {
    fn agent_for_plugin_backends(&self, backends: &PluginBackends) -> Arc<dyn AgentProvider>;
    fn agent_for(&self, b: AgentBackend) -> Arc<dyn AgentProvider>;
    fn memory_retrieval_for_plugin_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn MemoryRetrieval>;
    fn memory_retrieval(&self, b: MemoryBackend) -> Arc<dyn MemoryRetrieval>;
    fn user_emotion_analyzer_for_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn UserEmotionAnalyzer>;
    fn user_emotion_analyzer(&self, b: EmotionBackend) -> Arc<dyn UserEmotionAnalyzer>;
    fn event_estimator_for_backends(&self, backends: &PluginBackends) -> Arc<dyn EventEstimator>;
    fn event_estimator(&self, b: EventBackend) -> Arc<dyn EventEstimator>;
    fn prompt_assembler_for_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn PromptAssembler>;
    fn prompt_assembler(&self, b: PromptBackend) -> Arc<dyn PromptAssembler>;
    fn llm_for_plugin_backends(&self, backends: &PluginBackends) -> Arc<dyn LlmClient>;
    fn llm_for(&self, b: LlmBackend) -> Arc<dyn LlmClient>;

    fn pick_complex_emotion_winner(
        &self,
        slot_registry: &BTreeMap<String, SlotRegistryEntry>,
    ) -> Arc<dyn ComplexEmotionProvider>;

    fn pick_complex_emotion_for_entry(
        &self,
        entry: &SlotRegistryEntry,
    ) -> Arc<dyn ComplexEmotionProvider>;
}
