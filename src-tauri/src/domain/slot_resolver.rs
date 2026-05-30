//! # Blueprint `slot_registry` → executable slot list
//!
//! **Role**: reads validated `slot_registry` from the role pack (`pipeline.ocblueprint`), binds each instance to a concrete implementation in `BackendRegistry`, and produces [`ResolvedRoleSlots`] for [`SlotRunner`](super::slot_runner::SlotRunner) merge execution.
//!
//! **Upstream**: `oclive_validation::slot_registry_instances_sorted`; [`PluginHost::resolve_for_role`](super::plugin_host.rs) passes registry + `slot_registry`.
//! **Downstream**: [`SlotRunner`](super::slot_runner::SlotRunner); `groups` only affects frontend architecture graph grouping and **does not** change resolution order here.
//!
//! **Key decision**: sort instances by `slot_type` + `position`; `module_relations` are **not persisted**—the frontend **read-only derives** edges from `slot_registry` to avoid dual sources of truth.

use crate::domain::agent::AgentProvider;
use crate::domain::complex_emotion::{
    BuiltinKeywordComplexEmotionProvider, ComplexEmotionInput, ComplexEmotionOutput,
    ComplexEmotionProvider,
};
use crate::domain::event_estimator::EventEstimator;
use crate::domain::memory_retrieval::MemoryRetrieval;
use crate::domain::plugin_host::BackendRegistry;

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
use crate::domain::prompt_assembler::PromptAssembler;
use crate::domain::user_emotion_analyzer::UserEmotionAnalyzer;
use crate::domain::ports::LlmClient;
use crate::models::PluginBackends;
use crate::infrastructure::remote_plugin::{
    DirectoryComplexEmotionHttp, RemoteComplexEmotionHttp, RemotePluginHttpConfig,
};
use oclive_validation::{
    merged_agent_directory_plugin_ids, plugin_backends_for_slot_entry,
    slot_registry_instances_sorted, SlotRegistryEntry,
};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

/// Built-in complex emotion (wrapped as trait object).
pub struct BuiltinComplexEmotionArc;

impl ComplexEmotionProvider for BuiltinComplexEmotionArc {
    fn resolve_turn(
        &self,
        input: &ComplexEmotionInput,
    ) -> crate::error::Result<ComplexEmotionOutput> {
        Ok(BuiltinKeywordComplexEmotionProvider.resolve_turn_inner(input))
    }
}

struct RemoteComplexEmotionArc(Arc<RemoteComplexEmotionHttp>);

impl ComplexEmotionProvider for RemoteComplexEmotionArc {
    fn resolve_turn(
        &self,
        input: &ComplexEmotionInput,
    ) -> crate::error::Result<ComplexEmotionOutput> {
        self.0.resolve_turn(input)
    }
}

/// Multi-`agent` slot / `plugins[]` merge metadata; `process` still delegates to `inner` (tool merge deepened in P4).
pub struct CompositeAgentProvider {
    pub inner: Arc<dyn AgentProvider>,
    pub merged_directory_plugin_ids: Vec<String>,
}

#[async_trait::async_trait]
impl AgentProvider for CompositeAgentProvider {
    async fn process(
        &self,
        input: crate::domain::agent::AgentInput,
    ) -> crate::error::Result<crate::domain::agent::AgentOutput> {
        self.inner.process(input).await
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
    ///
    /// Caller: `PluginHost::resolve_for_*` on an existing `BackendRegistry`; result goes to [`SlotRunner`](super::slot_runner::SlotRunner) for merging.
    #[must_use]
    pub fn resolve(
        registry: &BackendRegistry,
        slot_registry: &BTreeMap<String, SlotRegistryEntry>,
    ) -> ResolvedRoleSlots {
        Self::resolve_with_session_backends(registry, slot_registry, None)
    }

    /// Resolves `slot_registry`; `session_effective_backends` overrides blueprint `llm` slot `backend`
    /// (e.g. after user picks cloud in model manager, `effective_plugin_backends.llm = Remote`, avoiding pack `ollama`).
    #[must_use]
    pub fn resolve_with_session_backends(
        registry: &BackendRegistry,
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
                Self::resolve_complex_emotion_entry(registry, &entry),
            ));
        }
        out
    }

    /// Same-type **last-wins** (`position` max) complex emotion implementation.
    #[must_use]
    pub fn resolve_complex_emotion_winner(
        registry: &BackendRegistry,
        slot_registry: &BTreeMap<String, SlotRegistryEntry>,
    ) -> Arc<dyn ComplexEmotionProvider> {
        slot_registry_instances_sorted(slot_registry, "complex_emotion")
            .last()
            .map(|(_, e)| Self::resolve_complex_emotion_entry(registry, e))
            .unwrap_or_else(|| Arc::new(BuiltinComplexEmotionArc))
    }

    fn resolve_complex_emotion_entry(
        registry: &BackendRegistry,
        entry: &SlotRegistryEntry,
    ) -> Arc<dyn ComplexEmotionProvider> {
        let remote_fb = registry.remote_fallback_allowed();
        match entry.backend.trim() {
            "builtin" => Arc::new(BuiltinComplexEmotionArc),
            "remote" => {
                let cfg = entry
                    .url
                    .as_ref()
                    .map(|u| u.trim())
                    .filter(|u| !u.is_empty())
                    .map(|endpoint| RemotePluginHttpConfig {
                        endpoint: endpoint.to_string(),
                        timeout: Duration::from_millis(8_000),
                        bearer_token: None,
                    })
                    .or_else(RemotePluginHttpConfig::from_env_complex_emotion);
                let Some(cfg) = cfg else {
                    tracing::warn!(
                        target: "oclive_plugin",
                        "complex_emotion backend=remote but no url/env; using builtin"
                    );
                    return Arc::new(BuiltinComplexEmotionArc);
                };
                match RemoteComplexEmotionHttp::new(
                    cfg,
                    remote_fb.clone(),
                    registry.high_risk_grants(),
                ) {
                    Ok(http) => Arc::new(RemoteComplexEmotionArc(Arc::new(http))),
                    Err(e) => {
                        tracing::error!(
                            target: "oclive_plugin",
                            "complex_emotion remote client build failed: {}; using builtin",
                            e
                        );
                        Arc::new(BuiltinComplexEmotionArc)
                    }
                }
            }
            "directory" => Self::resolve_complex_emotion_directory(registry, entry, remote_fb),
            _ => Arc::new(BuiltinComplexEmotionArc),
        }
    }

    fn resolve_complex_emotion_directory(
        registry: &BackendRegistry,
        entry: &SlotRegistryEntry,
        remote_fb: Arc<std::sync::atomic::AtomicBool>,
    ) -> Arc<dyn ComplexEmotionProvider> {
        let Some(rt) = registry.directory_runtime() else {
            tracing::warn!(
                target: "oclive_plugin",
                "complex_emotion backend=directory but runtime disabled; using builtin"
            );
            return Arc::new(BuiltinComplexEmotionArc);
        };
        let plugin_id = entry
            .plugin
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let Some(pid) = plugin_id else {
            tracing::warn!(
                target: "oclive_plugin",
                "complex_emotion backend=directory but plugin id missing; using builtin"
            );
            return Arc::new(BuiltinComplexEmotionArc);
        };
        if !rt.manifest_provides_capability(&pid, "complex_emotion") {
            tracing::warn!(
                target: "oclive_plugin",
                "directory plugin_id={} missing provides complex_emotion; using builtin",
                pid
            );
            return Arc::new(BuiltinComplexEmotionArc);
        }
        match rt.ensure_rpc_url(pid.as_str()) {
            Ok(url) => {
                let cfg = RemotePluginHttpConfig::for_directory_plugin_rpc(url, false);
                match DirectoryComplexEmotionHttp::new(cfg, remote_fb) {
                    Ok(http) => Arc::new(http),
                    Err(e) => {
                        tracing::error!(
                            target: "oclive_plugin",
                            "complex_emotion directory plugin_id={} client failed: {}; using builtin",
                            pid,
                            e
                        );
                        Arc::new(BuiltinComplexEmotionArc)
                    }
                }
            }
            Err(e) => {
                tracing::error!(
                    target: "oclive_plugin",
                    "complex_emotion directory plugin_id={} spawn failed: {}; using builtin",
                    pid,
                    e
                );
                Arc::new(BuiltinComplexEmotionArc)
            }
        }
    }

    /// **Multi agent instances**: merge multiple directory plugin IDs into a composite Agent (tool set exposed in parallel, not serial inside SlotRunner).
    ///
    /// **Why merge at resolve layer**: multi Agent slots have no LLM context dependency; only a unified tool list is needed for the `process_message` entry shortcut.
    #[must_use]
    pub fn wrap_agent_if_merged(
        inner: Arc<dyn AgentProvider>,
        slot_registry: &BTreeMap<String, SlotRegistryEntry>,
    ) -> Arc<dyn AgentProvider> {
        let merged = merged_agent_directory_plugin_ids(slot_registry);
        if merged.len() <= 1 {
            return inner;
        }
        tracing::info!(
            target: "oclive_plugin",
            "composite agent: merged {} directory plugin ids from slot_registry",
            merged.len()
        );
        Arc::new(CompositeAgentProvider {
            inner,
            merged_directory_plugin_ids: merged,
        })
    }
}
