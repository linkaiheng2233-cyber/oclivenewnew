//! # Blueprint v2 multi-instance slot executor (`SlotRunner`)
//!
//! **Role**: when `slot_registry` has multiple instances of the same `slot_type`, pick a **merge policy** per type (serial last-wins, memory dedup-merge, serial LLM, etc.) and invoke the corresponding `dyn` implementation.
//!
//! **Upstream**: [`SlotResolver`](../slot_resolver.rs) produces `ResolvedRoleSlots`; [`PluginHost`](../plugin_host.rs) provides `BackendRegistry`.
//! **Downstream**: co-present stages (emotion, event, memory ranking, Prompt, LLM); **multi directory-plugin Agent merge** lives in `PluginHost` / `SlotResolver::wrap_agent_if_merged`, not here.
//!
//! **Key decision**: merge policy follows slot semantics (see `*_last_wins` / `memory_merge_rank` doc comments)—e.g. memory needs **dedup-merge**, LLM only needs the **final reply**; avoid one-size-fits-all parallelism that corrupts context.

#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::too_many_arguments
)]

use crate::domain::complex_emotion::{
    ComplexEmotionInput, ComplexEmotionOutput, ComplexEmotionProvider,
};
use crate::domain::emotion_analyzer::EmotionResult;
use crate::domain::event_estimator::EventEstimator;
use crate::domain::event_impact_ai::EventImpactEstimate;
use crate::domain::memory_retrieval::{MemoryRetrieval, MemoryRetrievalInput};
use crate::domain::plugin_host::ResolvedRolePlugins;
use crate::domain::ports::LlmClient;
use crate::domain::prompt_assembler::PromptAssembler;
use crate::domain::prompt_builder::PromptInput;
use crate::domain::slot_resolver::{LlmMergePolicy, ResolvedRoleSlots};
use crate::domain::user_emotion_analyzer::UserEmotionAnalyzer;
use crate::error::Result;
use crate::models::knowledge::KnowledgeEventAugment;
use crate::models::{Emotion, Event, Memory, PersonalitySource, PersonalityVector, Role};
use std::collections::HashSet;
use std::sync::Arc;

/// First argument to `ollama_msg`: slot module name (matches registry `slot_type`).
mod slot_module {
    pub(super) const EMOTION: &str = "emotion";
    pub(super) const COMPLEX_EMOTION: &str = "complex_emotion";
    pub(super) const EVENT: &str = "event";
    pub(super) const PROMPT: &str = "prompt";
}
mod llm_merge;

pub struct SlotRunner;

impl SlotRunner {
    fn run_slot_sync<T: ?Sized, R, Pick, Multi, Single, Fallback>(
        slots: &Option<ResolvedRoleSlots>,
        pick: Pick,
        multi: Multi,
        single: Single,
        fallback: Fallback,
    ) -> R
    where
        Pick: FnOnce(&ResolvedRoleSlots) -> &[(String, Arc<T>)],
        Multi: FnOnce(&[(String, Arc<T>)]) -> R,
        Single: FnOnce(&Arc<T>) -> R,
        Fallback: FnOnce() -> R,
    {
        if let Some(instances) = registry_instances(slots, pick) {
            if instances.len() >= 2 {
                multi(instances)
            } else {
                single(&instances[0].1)
            }
        } else {
            fallback()
        }
    }

    async fn run_slot_async<T: ?Sized, R, Pick, Multi, Single, Fallback, FutM, FutS, FutF>(
        slots: &Option<ResolvedRoleSlots>,
        pick: Pick,
        multi: Multi,
        single: Single,
        fallback: Fallback,
    ) -> R
    where
        Pick: FnOnce(&ResolvedRoleSlots) -> &[(String, Arc<T>)],
        Multi: FnOnce(&[(String, Arc<T>)]) -> FutM,
        FutM: std::future::Future<Output = R>,
        Single: FnOnce(&Arc<T>) -> FutS,
        FutS: std::future::Future<Output = R>,
        Fallback: FnOnce() -> FutF,
        FutF: std::future::Future<Output = R>,
    {
        if let Some(instances) = registry_instances(slots, pick) {
            if instances.len() >= 2 {
                multi(instances).await
            } else {
                single(&instances[0].1).await
            }
        } else {
            fallback().await
        }
    }

    /// Fold six-slot LLM, or the `llm` instance with largest `position` in the registry.
    #[must_use]
    pub fn primary_llm(pl: &ResolvedRolePlugins) -> Arc<dyn LlmClient> {
        pl.slots
            .as_ref()
            .and_then(|s| s.llm.last().map(|(_, l)| Arc::clone(l)))
            .unwrap_or_else(|| Arc::clone(&pl.llm))
    }

    /// `emotion`: serial calls, **last-wins** (≥2 instances); single instance uses registry entry.
    ///
    /// **Why last-wins**: emotion analysis updates **current user emotion state**; intermediate states need not be kept—the last analysis overwrites prior results.
    /// **Why not parallel**: analyzers share the same input and produce mutually exclusive outputs; parallelism wastes compute and adds merge ambiguity.
    /// **Limitation**: when earlier instances fail, only logs are emitted; there may still be no valid result (see `emotion_last_wins`).
    pub fn analyze_emotion(pl: &ResolvedRolePlugins, text: &str) -> Result<EmotionResult> {
        Self::run_slot_sync(
            &pl.slots,
            |s| &s.emotion,
            |instances| Self::emotion_last_wins(instances, text),
            |analyzer| analyzer.analyze(text),
            || pl.emotion.analyze(text),
        )
    }

    /// `complex_emotion`: serial, **last-wins** (multi-instance slot policy).
    pub fn resolve_complex_emotion(
        pl: &ResolvedRolePlugins,
        input: &ComplexEmotionInput,
    ) -> Result<ComplexEmotionOutput> {
        Self::run_slot_sync(
            &pl.slots,
            |s| &s.complex_emotion,
            |instances| Self::complex_emotion_last_wins(instances, input),
            |provider| provider.resolve_turn(input),
            || pl.complex_emotion.resolve_turn(input),
        )
    }

    /// `event`: serial estimate, **last-wins** (intermediate instances log at debug).
    pub async fn estimate_event(
        pl: &ResolvedRolePlugins,
        ollama_model: &str,
        user_message: &str,
        user_emotion: &Emotion,
        personality: &PersonalityVector,
        personality_source: PersonalitySource,
        recent_turns: &[(String, String)],
        recent_events: &[Event],
        knowledge_augment: Option<&KnowledgeEventAugment>,
    ) -> Result<EventImpactEstimate> {
        let llm = Self::primary_llm(pl);
        let ollama_model = ollama_model.to_string();
        let user_message = user_message.to_string();
        let user_emotion = user_emotion.clone();
        let personality = personality.clone();
        let recent_turns = recent_turns.to_vec();
        let recent_events = recent_events.to_vec();
        let knowledge_augment = knowledge_augment.cloned();
        Self::run_slot_async(
            &pl.slots,
            |s| &s.event,
            |instances| {
                let instances = clone_instances(instances);
                let llm = Arc::clone(&llm);
                let ollama_model = ollama_model.clone();
                let user_message = user_message.clone();
                let user_emotion = user_emotion.clone();
                let personality = personality.clone();
                let recent_turns = recent_turns.clone();
                let recent_events = recent_events.clone();
                let knowledge_augment = knowledge_augment.clone();
                async move {
                    Self::event_last_wins(
                        &instances,
                        &llm,
                        &ollama_model,
                        &user_message,
                        &user_emotion,
                        &personality,
                        personality_source,
                        &recent_turns,
                        &recent_events,
                        knowledge_augment.as_ref(),
                    )
                    .await
                }
            },
            |estimator| {
                let llm = Arc::clone(&llm);
                let estimator = Arc::clone(estimator);
                let ollama_model = ollama_model.clone();
                let user_message = user_message.clone();
                let user_emotion = user_emotion.clone();
                let personality = personality.clone();
                let recent_turns = recent_turns.clone();
                let recent_events = recent_events.clone();
                let knowledge_augment = knowledge_augment.clone();
                async move {
                    estimator
                        .estimate(
                            &llm,
                            &ollama_model,
                            &user_message,
                            &user_emotion,
                            &personality,
                            personality_source,
                            &recent_turns,
                            &recent_events,
                            knowledge_augment.as_ref(),
                        )
                        .await
                }
            },
            || {
                let llm = Arc::clone(&llm);
                let event = Arc::clone(&pl.event);
                let ollama_model = ollama_model.clone();
                let user_message = user_message.clone();
                let user_emotion = user_emotion.clone();
                let personality = personality.clone();
                let recent_turns = recent_turns.clone();
                let recent_events = recent_events.clone();
                let knowledge_augment = knowledge_augment.clone();
                async move {
                    event
                        .estimate(
                            &llm,
                            &ollama_model,
                            &user_message,
                            &user_emotion,
                            &personality,
                            personality_source,
                            &recent_turns,
                            &recent_events,
                            knowledge_augment.as_ref(),
                        )
                        .await
                }
            },
        )
        .await
    }

    /// `memory`: serial rank → dedupe by id → sort by `importance * weight`.
    pub fn rank_memories(
        pl: &ResolvedRolePlugins,
        input: MemoryRetrievalInput<'_>,
    ) -> Result<Vec<Memory>> {
        if let Some(instances) = registry_instances(&pl.slots, |s| &s.memory) {
            if instances.len() >= 2 {
                return Self::memory_merge_rank(instances, input);
            }
            return instances[0].1.rank_memories(input);
        }
        pl.memory.rank_memories(input)
    }

    /// `prompt`：`top_topic_hint` **last-wins**。
    pub fn top_topic_hint(pl: &ResolvedRolePlugins, role: &Role, scene_id: &str) -> Option<String> {
        Self::run_slot_sync(
            &pl.slots,
            |s| &s.prompt,
            |instances| Self::prompt_top_topic_last_wins(instances, role, scene_id),
            |assembler| assembler.top_topic_hint(role, scene_id),
            || pl.prompt.top_topic_hint(role, scene_id),
        )
    }

    /// `prompt`：`build_prompt` **last-wins**。
    pub fn build_prompt(pl: &ResolvedRolePlugins, input: &PromptInput<'_>) -> Result<String> {
        Self::run_slot_sync(
            &pl.slots,
            |s| &s.prompt,
            |instances| Self::prompt_build_last_wins(instances, input),
            |assembler| assembler.build_prompt(input),
            || pl.prompt.build_prompt(input),
        )
    }

    /// `llm`: serial **call all** (logged), **last-wins** as the final reply.
    pub async fn generate_llm(
        pl: &ResolvedRolePlugins,
        ollama_model: &str,
        prompt: &str,
    ) -> Result<String> {
        if let Some(instances) = registry_instances(&pl.slots, |s| &s.llm) {
            if instances.len() >= 2 {
                let policy = pl
                    .slots
                    .as_ref()
                    .map(|s| s.llm_merge_policy)
                    .unwrap_or(LlmMergePolicy::Ensemble);
                return match policy {
                    LlmMergePolicy::Fastest => {
                        Self::llm_fastest_wins(instances, ollama_model, prompt).await
                    }
                    LlmMergePolicy::Fallback => {
                        Self::llm_fallback_first(instances, ollama_model, prompt).await
                    }
                    LlmMergePolicy::Ensemble => {
                        Self::llm_serial_last_wins(instances, ollama_model, prompt).await
                    }
                };
            }
        }
        Self::run_slot_async(
            &pl.slots,
            |s| &s.llm,
            |instances| {
                let instances = clone_instances(instances);
                let ollama_model = ollama_model.to_string();
                let prompt = prompt.to_string();
                async move { Self::llm_serial_last_wins(&instances, &ollama_model, &prompt).await }
            },
            |llm| {
                let llm = Arc::clone(llm);
                let ollama_model = ollama_model.to_string();
                let prompt = prompt.to_string();
                async move { llm.generate(&ollama_model, &prompt).await }
            },
            || {
                let llm = Arc::clone(&pl.llm);
                let ollama_model = ollama_model.to_string();
                let prompt = prompt.to_string();
                async move { llm.generate(&ollama_model, &prompt).await }
            },
        )
        .await
    }

    /// Streaming variant of [`generate_llm`](Self::generate_llm).
    pub async fn generate_llm_stream(
        pl: &ResolvedRolePlugins,
        ollama_model: &str,
        prompt: &str,
        on_token: oclive_kernel_contracts::LlmTokenSink,
    ) -> Result<String> {
        if let Some(instances) = registry_instances(&pl.slots, |s| &s.llm) {
            if instances.len() >= 2 {
                let policy = pl
                    .slots
                    .as_ref()
                    .map(|s| s.llm_merge_policy)
                    .unwrap_or(LlmMergePolicy::Ensemble);
                return match policy {
                    LlmMergePolicy::Fastest
                    | LlmMergePolicy::Fallback
                    | LlmMergePolicy::Ensemble => {
                        Self::llm_serial_last_wins_stream(instances, ollama_model, prompt, on_token)
                            .await
                    }
                };
            }
        }
        Self::run_slot_async(
            &pl.slots,
            |s| &s.llm,
            |instances| {
                let instances = clone_instances(instances);
                let ollama_model = ollama_model.to_string();
                let prompt = prompt.to_string();
                let on_token = std::sync::Arc::clone(&on_token);
                async move {
                    Self::llm_serial_last_wins_stream(&instances, &ollama_model, &prompt, on_token)
                        .await
                }
            },
            |llm| {
                let llm = Arc::clone(llm);
                let ollama_model = ollama_model.to_string();
                let prompt = prompt.to_string();
                let on_token = std::sync::Arc::clone(&on_token);
                async move { llm.generate_stream(&ollama_model, &prompt, on_token).await }
            },
            || {
                let llm = Arc::clone(&pl.llm);
                let ollama_model = ollama_model.to_string();
                let prompt = prompt.to_string();
                let on_token = std::sync::Arc::clone(&on_token);
                async move { llm.generate_stream(&ollama_model, &prompt, on_token).await }
            },
        )
        .await
    }

    /// Emotion chain **last-wins**: serial by `position`, keep the last successful result.
    fn emotion_last_wins(
        instances: &[(String, Arc<dyn UserEmotionAnalyzer>)],
        text: &str,
    ) -> Result<EmotionResult> {
        let mut last: Option<EmotionResult> = None;
        for (key, analyzer) in instances {
            match analyzer.analyze(text) {
                Ok(r) => {
                    tracing::debug!(
                        target: "oclive_plugin",
                        slot_key = %key,
                        "emotion analyze slot (last-wins chain)"
                    );
                    last = Some(r);
                }
                Err(e) => {
                    tracing::warn!(
                        target: "oclive_plugin",
                        slot_key = %key,
                        err = %e,
                        "emotion analyze slot failed"
                    );
                }
            }
        }
        last.ok_or_else(|| {
            crate::domain::error_helpers::ollama_msg(
                slot_module::EMOTION,
                "no slot produced a result",
            )
        })
    }

    /// **complex_emotion last-wins**: same as emotion—narrative hint from the last successful `resolve_turn`.
    fn complex_emotion_last_wins(
        instances: &[(String, Arc<dyn ComplexEmotionProvider>)],
        input: &ComplexEmotionInput,
    ) -> Result<ComplexEmotionOutput> {
        let mut last: Option<ComplexEmotionOutput> = None;
        for (key, provider) in instances {
            match provider.resolve_turn(input) {
                Ok(r) => {
                    tracing::debug!(
                        target: "oclive_plugin",
                        slot_key = %key,
                        source = %r.source,
                        "complex_emotion slot (last-wins chain)"
                    );
                    last = Some(r);
                }
                Err(e) => {
                    tracing::warn!(
                        target: "oclive_plugin",
                        slot_key = %key,
                        err = %e,
                        "complex_emotion slot failed"
                    );
                }
            }
        }
        last.ok_or_else(|| {
            crate::domain::error_helpers::ollama_msg(
                slot_module::COMPLEX_EMOTION,
                "no slot produced a result",
            )
        })
    }

    /// **event serial last-wins**: event detectors run in order; keep the last successful estimate.
    ///
    /// **Problem solved**: different detectors may emit duplicate or conflicting event tags for the same turn.
    /// **Why last-wins**: event impact drives personality/memory policy; only **one** normalized estimate is needed; intermediate states log at debug.
    /// **Limitation**: supplementary signals from earlier detectors are not merged; only the last path applies.
    async fn event_last_wins(
        instances: &[(String, Arc<dyn EventEstimator>)],
        llm: &Arc<dyn LlmClient>,
        ollama_model: &str,
        user_message: &str,
        user_emotion: &Emotion,
        personality: &PersonalityVector,
        personality_source: PersonalitySource,
        recent_turns: &[(String, String)],
        recent_events: &[Event],
        knowledge_augment: Option<&KnowledgeEventAugment>,
    ) -> Result<EventImpactEstimate> {
        let mut last: Option<EventImpactEstimate> = None;
        for (key, estimator) in instances {
            match estimator
                .estimate(
                    llm,
                    ollama_model,
                    user_message,
                    user_emotion,
                    personality,
                    personality_source,
                    recent_turns,
                    recent_events,
                    knowledge_augment,
                )
                .await
            {
                Ok(est) => {
                    tracing::debug!(
                        target: "oclive_plugin",
                        slot_key = %key,
                        event_type = ?est.event_type,
                        impact = est.impact_factor,
                        "event_estimate slot (last-wins chain)"
                    );
                    last = Some(est);
                }
                Err(e) => {
                    tracing::warn!(
                        target: "oclive_plugin",
                        slot_key = %key,
                        err = %e,
                        "event_estimate slot failed"
                    );
                }
            }
        }
        last.ok_or_else(|| {
            crate::domain::error_helpers::ollama_msg(
                slot_module::EVENT,
                "no slot produced a result",
            )
        })
    }

    /// **memory serial dedup-merge**: dedupe multi-path retrieval by `memory.id`, then sort/truncate by importance×weight.
    ///
    /// **Problem solved**: the same event may be recalled twice from multiple memory instances (different providers).
    /// **Why serial not parallel**: later instances may depend on ranking heuristics already written; merge needs a global dedup set.
    /// **Why not last-wins**: users need the **union**, not a single path's Top-K.
    /// **Limitation**: a failed instance is skipped and may cause missed recall; `limit` is applied after merge.
    fn memory_merge_rank(
        instances: &[(String, Arc<dyn MemoryRetrieval>)],
        input: MemoryRetrievalInput<'_>,
    ) -> Result<Vec<Memory>> {
        if instances.len() == 1 {
            return instances[0].1.rank_memories(input);
        }
        let limit = input.limit;
        let mut seen = HashSet::new();
        let mut merged = Vec::new();
        for (key, retrieval) in instances {
            let step_input = MemoryRetrievalInput {
                memories: input.memories,
                user_query: input.user_query,
                scene_id: input.scene_id,
                limit,
            };
            match retrieval.rank_memories(step_input) {
                Ok(mut ranked) => {
                    tracing::debug!(
                        target: "oclive_plugin",
                        slot_key = %key,
                        ranked = ranked.len(),
                        "memory_rank slot (merge chain)"
                    );
                    for m in ranked.drain(..) {
                        if seen.insert(m.id.clone()) {
                            merged.push(m);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        target: "oclive_plugin",
                        slot_key = %key,
                        err = %e,
                        "memory_rank slot failed"
                    );
                }
            }
        }
        merged.sort_by(|a, b| {
            let sa = a.importance * a.weight;
            let sb = b.importance * b.weight;
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
        merged.truncate(limit);
        Ok(merged)
    }

    /// **prompt top_topic last-wins**: serial `top_topic_hint` across assemblers; keep the last `Some`.
    fn prompt_top_topic_last_wins(
        instances: &[(String, Arc<dyn PromptAssembler>)],
        role: &Role,
        scene_id: &str,
    ) -> Option<String> {
        let mut last = None;
        for (key, asm) in instances {
            let hint = asm.top_topic_hint(role, scene_id);
            tracing::debug!(
                target: "oclive_plugin",
                slot_key = %key,
                has_hint = hint.is_some(),
                "prompt top_topic_hint slot (last-wins chain)"
            );
            if hint.is_some() {
                last = hint;
            }
        }
        last
    }

    /// **prompt last-wins**: assemblers call `build_prompt` in order; final string is the last successful output.
    ///
    /// **Problem solved**: creators may stack Prompt plugins for experiments, but only one text may go to the LLM.
    /// **Why last-wins**: Prompt is the pipeline end that **builds final context**; later writes overwriting earlier ones match "last processing step" intuition.
    /// **Limitation**: cannot auto-concatenate multiple Prompt segments; merge within a single instance instead.
    fn prompt_build_last_wins(
        instances: &[(String, Arc<dyn PromptAssembler>)],
        input: &PromptInput<'_>,
    ) -> Result<String> {
        let mut last: Option<String> = None;
        for (key, asm) in instances {
            match asm.build_prompt(input) {
                Ok(p) => {
                    tracing::debug!(
                        target: "oclive_plugin",
                        slot_key = %key,
                        prompt_len = p.len(),
                        "build_prompt slot (last-wins chain)"
                    );
                    last = Some(p);
                }
                Err(e) => {
                    tracing::warn!(
                        target: "oclive_plugin",
                        slot_key = %key,
                        err = %e,
                        "build_prompt slot failed"
                    );
                }
            }
        }
        last.ok_or_else(|| {
            crate::domain::error_helpers::ollama_msg(
                slot_module::PROMPT,
                "no slot produced a result",
            )
        })
    }

    /// **llm fallback**: call in order; return on **first** success.
    async fn llm_fallback_first(
        instances: &[(String, Arc<dyn LlmClient>)],
        ollama_model: &str,
        prompt: &str,
    ) -> Result<String> {
        llm_merge::fallback_first(instances, ollama_model, prompt).await
    }

    /// **llm fastest-wins**: concurrent calls; return on **first** success and cancel remaining tasks.
    async fn llm_fastest_wins(
        instances: &[(String, Arc<dyn LlmClient>)],
        ollama_model: &str,
        prompt: &str,
    ) -> Result<String> {
        llm_merge::fastest_wins(instances, ollama_model, prompt).await
    }

    /// **llm serial last-wins**: multiple LLM instances generate on the **same prompt**; keep only the last successful reply.
    ///
    /// **Problem solved**: blueprint may configure multiple LLM slots (e.g. primary + fallback); runtime needs only **one** user-visible reply.
    /// **Why serial**: calls share the same prompt context; even without chain dependencies, avoids hammering GPU/quota with concurrency.
    /// **Why last-wins**: matches "final displayed reply" semantics; earlier successes are logged for comparison only.
    /// **Limitation**: not ensemble voting; failed instances are skipped; error if all fail.
    async fn llm_serial_last_wins(
        instances: &[(String, Arc<dyn LlmClient>)],
        ollama_model: &str,
        prompt: &str,
    ) -> Result<String> {
        llm_merge::serial_last_wins(instances, ollama_model, prompt).await
    }

    async fn llm_serial_last_wins_stream(
        instances: &[(String, Arc<dyn LlmClient>)],
        ollama_model: &str,
        prompt: &str,
        on_token: oclive_kernel_contracts::LlmTokenSink,
    ) -> Result<String> {
        llm_merge::serial_last_wins_stream(instances, ollama_model, prompt, on_token).await
    }
}

fn clone_instances<T: ?Sized>(instances: &[(String, Arc<T>)]) -> Vec<(String, Arc<T>)> {
    instances
        .iter()
        .map(|(key, value)| (key.clone(), Arc::clone(value)))
        .collect()
}

fn registry_instances<'a, T: ?Sized>(
    slots: &'a Option<ResolvedRoleSlots>,
    pick: impl FnOnce(&'a ResolvedRoleSlots) -> &'a [(String, Arc<T>)],
) -> Option<&'a [(String, Arc<T>)]> {
    let s = slots.as_ref()?;
    let v = pick(s);
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::memory_retrieval::BuiltinMemoryRetrieval;
    use crate::models::Memory;
    use chrono::Utc;

    fn sample_memories() -> Vec<Memory> {
        vec![
            Memory {
                id: "a".into(),
                role_id: "r".into(),
                content: "alpha".into(),
                importance: 0.9,
                weight: 1.0,
                created_at: Utc::now(),
                scene_id: None,
                mention_count: 1,
                accessed_at: None,
            },
            Memory {
                id: "b".into(),
                role_id: "r".into(),
                content: "beta".into(),
                importance: 0.5,
                weight: 1.0,
                created_at: Utc::now(),
                scene_id: None,
                mention_count: 1,
                accessed_at: None,
            },
        ]
    }

    #[test]
    fn memory_merge_dedupes_by_id() {
        let instances = [
            (
                "m1".into(),
                Arc::new(BuiltinMemoryRetrieval) as Arc<dyn MemoryRetrieval>,
            ),
            (
                "m2".into(),
                Arc::new(BuiltinMemoryRetrieval) as Arc<dyn MemoryRetrieval>,
            ),
        ];
        let mems = sample_memories();
        let ranked = SlotRunner::memory_merge_rank(
            &instances,
            MemoryRetrievalInput {
                memories: &mems,
                user_query: "test",
                scene_id: None,
                limit: 8,
            },
        )
        .expect("merge");
        let ids: HashSet<_> = ranked.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids.len(), ranked.len());
        assert!(ids.contains("a"));
    }
}
