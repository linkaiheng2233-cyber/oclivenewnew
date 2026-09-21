//! # Blueprint v2 multi-instance slot executor (`SlotRunner`)
//!
//! **Role**: when `slot_registry` has multiple instances of the same `slot_type`, apply the type-specific execution policy (serial last-wins, memory dedup-merge, configurable non-streaming LLM selection, etc.) and invoke the corresponding `dyn` implementation.
//!
//! **Upstream**: [`SlotResolver`](../slot_resolver.rs) produces `ResolvedRoleSlots`; [`PluginHost`](../plugin_host.rs) provides `BackendRegistry`.
//! **Downstream**: co-present stages (emotion, event, memory ranking, Prompt, LLM). Agent execution uses the folded provider in `PluginHost`; `SlotResolver::wrap_agent_if_merged` is currently a no-op (`K-AGENT-MERGE-01`).
//!
//! **Key decision**: merge policy follows slot semantics (see `*_last_wins`, `memory_merge_rank`, and [`LlmMergePolicy`])—e.g. memory needs **dedup-merge**, while non-streaming LLMs may choose ensemble, fastest, or fallback behavior. Avoid one-size-fits-all parallelism that corrupts context.

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
use crate::domain::prompt_builder::{PromptBuilder, PromptInput, PromptSegments};
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
        use_event_impact_llm: bool,
    ) -> Result<EventImpactEstimate> {
        if !use_event_impact_llm {
            return crate::domain::event_impact_ai::estimate_event_impact_rules_only(
                user_message,
                user_emotion,
                knowledge_augment,
            );
        }
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

    /// Stable prefix-cache segment builder for Fast/Deep turns.
    ///
    /// This is builtin-only; orchestration must keep directory/remote prompt backends on
    /// [`build_prompt`](Self::build_prompt) so plugin prompt contracts are not bypassed.
    pub fn build_prompt_segments(
        pl: &ResolvedRolePlugins,
        input: &PromptInput<'_>,
    ) -> Result<PromptSegments> {
        Self::run_slot_sync(
            &pl.slots,
            |s| &s.prompt,
            |_instances| Ok(PromptBuilder::build_prompt_segments(input)),
            |_assembler| Ok(PromptBuilder::build_prompt_segments(input)),
            || Ok(PromptBuilder::build_prompt_segments(input)),
        )
    }

    pub(crate) async fn generate_llm_single(
        llm: &Arc<dyn LlmClient>,
        ollama_model: &str,
        prompt: &str,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        llm.generate_with_opts(ollama_model, prompt, opts).await
    }

    pub(crate) async fn generate_llm_stream_single(
        llm: &Arc<dyn LlmClient>,
        ollama_model: &str,
        prompt: &str,
        on_token: oclive_kernel_contracts::LlmTokenSink,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        llm.generate_stream_with_opts(ollama_model, prompt, on_token, opts)
            .await
    }

    /// `llm`: for multiple instances, apply the last LLM entry's [`LlmMergePolicy`].
    /// `Ensemble` is serial last-wins, `Fastest` returns the first concurrent success,
    /// and `Fallback` returns the first ordered success.
    pub async fn generate_llm(
        pl: &ResolvedRolePlugins,
        ollama_model: &str,
        prompt: &str,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        if let Some(instances) = registry_instances(&pl.slots, |s| &s.llm) {
            if instances.len() >= 2 {
                let policy = pl
                    .slots
                    .as_ref()
                    .map(|s| s.llm_merge_policy)
                    .unwrap_or(LlmMergePolicy::Ensemble);
                return match policy {
                    LlmMergePolicy::Fastest => {
                        Self::llm_fastest_wins(instances, ollama_model, prompt, opts).await
                    }
                    LlmMergePolicy::Fallback => {
                        Self::llm_fallback_first(instances, ollama_model, prompt, opts).await
                    }
                    LlmMergePolicy::Ensemble => {
                        Self::llm_serial_last_wins(instances, ollama_model, prompt, opts).await
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
                async move {
                    Self::llm_serial_last_wins(&instances, &ollama_model, &prompt, opts).await
                }
            },
            |llm| {
                let llm = Arc::clone(llm);
                let ollama_model = ollama_model.to_string();
                let prompt = prompt.to_string();
                async move {
                    Self::generate_llm_single(&llm, &ollama_model, &prompt, opts).await
                }
            },
            || {
                let llm = Arc::clone(&pl.llm);
                let ollama_model = ollama_model.to_string();
                let prompt = prompt.to_string();
                async move {
                    Self::generate_llm_single(&llm, &ollama_model, &prompt, opts).await
                }
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
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
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
                        Self::llm_serial_last_wins_stream(
                            instances,
                            ollama_model,
                            prompt,
                            on_token,
                            opts,
                        )
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
                    Self::llm_serial_last_wins_stream(
                        &instances,
                        &ollama_model,
                        &prompt,
                        on_token,
                        opts,
                    )
                    .await
                }
            },
            |llm| {
                let llm = Arc::clone(llm);
                let ollama_model = ollama_model.to_string();
                let prompt = prompt.to_string();
                let on_token = std::sync::Arc::clone(&on_token);
                async move {
                    Self::generate_llm_stream_single(&llm, &ollama_model, &prompt, on_token, opts)
                        .await
                }
            },
            || {
                let llm = Arc::clone(&pl.llm);
                let ollama_model = ollama_model.to_string();
                let prompt = prompt.to_string();
                let on_token = std::sync::Arc::clone(&on_token);
                async move {
                    Self::generate_llm_stream_single(&llm, &ollama_model, &prompt, on_token, opts)
                        .await
                }
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
        _opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        let reply = llm_merge::fallback_first(instances, ollama_model, prompt).await?;
        Ok(LlmGenerateOutcome {
            reply,
            prompt_eval_ms: None,
        })
    }

    /// **llm fastest-wins**: concurrent calls; return on **first** success and cancel remaining tasks.
    async fn llm_fastest_wins(
        instances: &[(String, Arc<dyn LlmClient>)],
        ollama_model: &str,
        prompt: &str,
        _opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        let reply = llm_merge::fastest_wins(instances, ollama_model, prompt).await?;
        Ok(LlmGenerateOutcome {
            reply,
            prompt_eval_ms: None,
        })
    }

    /// **llm serial last-wins**: multiple LLM instances generate on the **same prompt**; keep only the last successful reply.
    async fn llm_serial_last_wins(
        instances: &[(String, Arc<dyn LlmClient>)],
        ollama_model: &str,
        prompt: &str,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        if instances.len() == 1 {
            return Self::generate_llm_single(&instances[0].1, ollama_model, prompt, opts).await;
        }
        let reply = llm_merge::serial_last_wins(instances, ollama_model, prompt).await?;
        Ok(LlmGenerateOutcome {
            reply,
            prompt_eval_ms: None,
        })
    }

    async fn llm_serial_last_wins_stream(
        instances: &[(String, Arc<dyn LlmClient>)],
        ollama_model: &str,
        prompt: &str,
        on_token: oclive_kernel_contracts::LlmTokenSink,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        if instances.len() == 1 {
            return Self::generate_llm_stream_single(
                &instances[0].1,
                ollama_model,
                prompt,
                on_token,
                opts,
            )
            .await;
        }
        let reply =
            llm_merge::serial_last_wins_stream(instances, ollama_model, prompt, on_token).await?;
        Ok(LlmGenerateOutcome {
            reply,
            prompt_eval_ms: None,
        })
    }
}

pub use crate::domain::ports::{LlmGenerateOpts, LlmGenerateOutcome};

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

    // ---------------------------------------------------------------------------------------
    // CP-B3-ALL unit M: the new builtin provider through the real consumer paths. Tests only —
    // production routing, merge policy and limit semantics are untouched by this unit.
    // ---------------------------------------------------------------------------------------

    fn query_memories() -> Vec<Memory> {
        vec![
            Memory {
                id: "high-unmatched".into(),
                role_id: "r".into(),
                content: "今天星期三".into(),
                importance: 1.0,
                weight: 1.0,
                created_at: Utc::now(),
                scene_id: None,
                mention_count: 1,
                accessed_at: None,
            },
            Memory {
                id: "low-matched".into(),
                role_id: "r".into(),
                content: "用户喜欢咖啡".into(),
                importance: 0.2,
                weight: 1.0,
                created_at: Utc::now(),
                scene_id: None,
                mention_count: 1,
                accessed_at: None,
            },
            Memory {
                id: "mid-matched".into(),
                role_id: "r".into(),
                content: "用户喜欢咖啡和茶".into(),
                importance: 0.6,
                weight: 1.0,
                created_at: Utc::now(),
                scene_id: None,
                mention_count: 1,
                accessed_at: None,
            },
        ]
    }

    fn query_plugins() -> crate::domain::plugin_host::ResolvedRolePlugins {
        crate::domain::plugin_host::ResolvedRolePlugins {
            memory: Arc::new(oclive_kernel_runtime::domain::query_memory::QueryMemoryRetrieval),
            emotion: Arc::new(crate::domain::noop_slot_backends::NoopUserEmotionAnalyzer),
            event: Arc::new(crate::domain::noop_slot_backends::NoopEventEstimator),
            prompt: Arc::new(crate::domain::noop_slot_backends::NoopPromptAssembler),
            llm: Arc::new(crate::domain::noop_slot_backends::NoopLlmClient),
            agent: Arc::new(crate::domain::noop_slot_backends::NoopAgentProvider),
            complex_emotion: Arc::new(
                crate::domain::complex_emotion::BuiltinKeywordComplexEmotionProvider,
            ),
            slots: None,
            merged_agent_directory_plugin_ids: Vec::new(),
        }
    }

    /// The single-instance consumer path (`pl.memory`, no registry) forwards the query to the new
    /// builtin provider and returns only the matched rows in the existing weighted order.
    #[test]
    fn cp_b3_all_memory_single_instance_consumer_answers_the_query() {
        let pl = query_plugins();
        let mems = query_memories();
        let ranked = SlotRunner::rank_memories(
            &pl,
            MemoryRetrievalInput {
                memories: &mems,
                user_query: "咖啡",
                scene_id: None,
                limit: 8,
            },
        )
        .expect("rank");
        let ids: Vec<&str> = ranked.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, vec!["mid-matched", "low-matched"]);

        // limit still truncates after the weighted order, and 0 still selects nothing.
        let limited = SlotRunner::rank_memories(
            &pl,
            MemoryRetrievalInput {
                memories: &mems,
                user_query: "咖啡",
                scene_id: None,
                limit: 1,
            },
        )
        .expect("rank");
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].id, "mid-matched");
        let none = SlotRunner::rank_memories(
            &pl,
            MemoryRetrievalInput {
                memories: &mems,
                user_query: "咖啡",
                scene_id: None,
                limit: 0,
            },
        )
        .expect("rank");
        assert!(none.is_empty());

        // An empty query keeps selecting every candidate (the documented convention), and a
        // non-matching query keeps the Host compatibility fallback.
        assert_eq!(
            SlotRunner::rank_memories(
                &pl,
                MemoryRetrievalInput {
                    memories: &mems,
                    user_query: "",
                    scene_id: None,
                    limit: 8,
                },
            )
            .expect("rank")
            .len(),
            mems.len()
        );
        assert_eq!(
            SlotRunner::rank_memories(
                &pl,
                MemoryRetrievalInput {
                    memories: &mems,
                    user_query: "完全不相关词",
                    scene_id: None,
                    limit: 8,
                },
            )
            .expect("rank")
            .len(),
            mems.len()
        );
    }

    /// Multi-instance merge stays dedup-merge by id, keeps the existing sort/limit and skips a
    /// failing instance — with the new provider as one of the instances.
    #[test]
    fn cp_b3_all_memory_merge_policy_is_unchanged() {
        struct FailingRetrieval;
        impl MemoryRetrieval for FailingRetrieval {
            fn rank_memories(
                &self,
                _input: MemoryRetrievalInput<'_>,
            ) -> crate::error::Result<Vec<Memory>> {
                Err(crate::error::AppError::InvalidParameter(
                    "instance failed".into(),
                ))
            }
            fn build_context(
                &self,
                memories: &[Memory],
                max_tokens: usize,
            ) -> crate::models::MemoryContext {
                crate::domain::memory_engine::MemoryEngine::build_context(memories, max_tokens)
            }
            fn search_memories(&self, keyword: &str, memories: &[Memory]) -> Vec<Memory> {
                crate::domain::memory_engine::MemoryEngine::search_memories(keyword, memories)
            }
        }

        let instances: [(String, Arc<dyn MemoryRetrieval>); 3] = [
            (
                "failing".into(),
                Arc::new(FailingRetrieval) as Arc<dyn MemoryRetrieval>,
            ),
            (
                "query".into(),
                Arc::new(oclive_kernel_runtime::domain::query_memory::QueryMemoryRetrieval)
                    as Arc<dyn MemoryRetrieval>,
            ),
            (
                "builtin".into(),
                Arc::new(BuiltinMemoryRetrieval) as Arc<dyn MemoryRetrieval>,
            ),
        ];
        let mems = query_memories();
        let ranked = SlotRunner::memory_merge_rank(
            &instances,
            MemoryRetrievalInput {
                memories: &mems,
                user_query: "咖啡",
                scene_id: None,
                limit: 8,
            },
        )
        .expect("merge");
        // Dedup by id: the union of the two successful instances, each id once.
        let ids: HashSet<&str> = ranked.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids.len(), ranked.len());
        assert_eq!(ranked.len(), mems.len());
        assert!(ids.contains("low-matched"));
        assert!(ids.contains("high-unmatched"));

        // The merge order stays the existing importance×weight order over the deduped union.
        let importances: Vec<f64> = ranked.iter().map(|m| m.importance).collect();
        for pair in importances.windows(2) {
            assert!(
                pair[0] >= pair[1],
                "merge must keep the existing weighted order: {importances:?}"
            );
        }

        // limit still applies to the merged union.
        let limited = SlotRunner::memory_merge_rank(
            &instances,
            MemoryRetrievalInput {
                memories: &mems,
                user_query: "咖啡",
                scene_id: None,
                limit: 1,
            },
        )
        .expect("merge");
        assert_eq!(limited.len(), 1);
    }

    // ---------------------------------------------------------------------------------------
    // CP-B3-V1: offline combination evidence for the neighbouring consumer points of this Host.
    //
    // Evidence layer of this section: the real `SlotRunner` methods are called directly with the
    // real builtin Memory/Prompt/Emotion implementations and a recording LLM double, and the
    // hand-off values are compared against literals written here. It is **not** a full turn, not a
    // product assembly run and not a model call: nothing below builds an `AppState`, a registry, a
    // database, a socket or an MCP client, and the doubles never reach a model or a tool.
    //
    // The counting wrappers below delegate to the real implementations; the real implementation is
    // what produces every asserted value, and the counter only records that the consumer was
    // entered. The "which slots are called" fixture is explicit: Memory, Prompt, Emotion and the
    // LLM consumer are the ones under test, while Agent and Event are recording doubles whose
    // counters must stay 0 (a silent `Noop` success would hide an unexpected call).
    //
    // CP-B3-V1-R1 wording: these tests are **hand-combined** calls of real methods, not one fully
    // connected product chain. Each pair of neighbouring calls is wired by the test (the test passes
    // the previous result on), so the evidence says "these values are handed over like this", not
    // "every edge is joined this way in a running Host". The values the tests hand over are literals
    // or local values as their own comments state; no cross-`Pending` borrowing, lifetime or
    // concurrency evidence is claimed anywhere in this section.
    // ---------------------------------------------------------------------------------------

    use crate::domain::agent::{AgentInput, AgentOutput, AgentProvider};
    use crate::models::{EventType, MemoryContext};
    use oclive_kernel_contracts::LlmGenerateOpts;
    use parking_lot::Mutex;
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Delegates to the real builtin memory provider and counts entries.
    ///
    /// The counter records that the consumer was entered; the selection itself is produced by the
    /// real `QueryMemoryRetrieval`, not by this wrapper.
    struct CountingMemory {
        inner: oclive_kernel_runtime::domain::query_memory::QueryMemoryRetrieval,
        calls: Arc<AtomicUsize>,
    }

    impl MemoryRetrieval for CountingMemory {
        fn rank_memories(&self, input: MemoryRetrievalInput<'_>) -> Result<Vec<Memory>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.inner.rank_memories(input)
        }

        fn build_context(&self, memories: &[Memory], max_tokens: usize) -> MemoryContext {
            self.inner.build_context(memories, max_tokens)
        }

        fn search_memories(&self, keyword: &str, memories: &[Memory]) -> Vec<Memory> {
            self.inner.search_memories(keyword, memories)
        }
    }

    /// Delegates to the real builtin prompt assembler and counts `build_prompt` entries.
    struct CountingPrompt {
        inner: oclive_kernel_runtime::domain::prompt_assembler::BuiltinPromptAssembler,
        calls: Arc<AtomicUsize>,
    }

    impl PromptAssembler for CountingPrompt {
        fn build_prompt(&self, input: &PromptInput<'_>) -> Result<String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.inner.build_prompt(input)
        }

        fn top_topic_hint(&self, role: &Role, scene_id: &str) -> Option<String> {
            self.inner.top_topic_hint(role, scene_id)
        }
    }

    /// Delegates to the real builtin emotion analyzer and counts entries.
    struct CountingEmotion {
        inner: oclive_kernel_runtime::domain::user_emotion_analyzer::BuiltinUserEmotionAnalyzer,
        calls: Arc<AtomicUsize>,
    }

    impl UserEmotionAnalyzer for CountingEmotion {
        fn analyze(&self, text: &str) -> Result<EmotionResult> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.inner.analyze(text)
        }
    }

    /// One recorded LLM entry: the exact model, prompt and opts the consumer handed over.
    #[derive(Debug, Clone)]
    struct LlmEntry {
        model: String,
        prompt: String,
        opts: Option<LlmGenerateOpts>,
    }

    /// Recording LLM double.
    ///
    /// It **overrides `generate_with_opts`** — the entry `SlotRunner` actually uses — so the opts
    /// parameter is observed instead of being dropped by the trait's default implementation.
    /// `generate` and `generate_tag` only count: if the production path bypassed the opts entry, the
    /// recorded entry list would stay empty and those counters would move instead.
    struct RecordingLlm {
        entries: Mutex<Vec<LlmEntry>>,
        script: Mutex<VecDeque<Result<oclive_kernel_contracts::LlmGenerateOutcome>>>,
        generate_calls: AtomicUsize,
        tag_calls: AtomicUsize,
    }

    impl RecordingLlm {
        fn new(
            script: impl IntoIterator<Item = Result<oclive_kernel_contracts::LlmGenerateOutcome>>,
        ) -> Arc<Self> {
            Arc::new(Self {
                entries: Mutex::new(Vec::new()),
                script: Mutex::new(script.into_iter().collect()),
                generate_calls: AtomicUsize::new(0),
                tag_calls: AtomicUsize::new(0),
            })
        }

        fn entries(&self) -> Vec<LlmEntry> {
            self.entries.lock().clone()
        }

        fn entry_count(&self) -> usize {
            self.entries.lock().len()
        }
    }

    #[async_trait::async_trait]
    impl LlmClient for RecordingLlm {
        async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
            self.generate_calls.fetch_add(1, Ordering::SeqCst);
            Err(crate::error::AppError::InvalidParameter(
                "v1 double: the consumer must use generate_with_opts".into(),
            ))
        }

        async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
            self.tag_calls.fetch_add(1, Ordering::SeqCst);
            Err(crate::error::AppError::InvalidParameter(
                "v1 double: the consumer must use generate_with_opts".into(),
            ))
        }

        async fn generate_with_opts(
            &self,
            model: &str,
            prompt: &str,
            opts: Option<&LlmGenerateOpts>,
        ) -> Result<oclive_kernel_contracts::LlmGenerateOutcome> {
            self.entries.lock().push(LlmEntry {
                model: model.to_string(),
                prompt: prompt.to_string(),
                opts: opts.cloned(),
            });
            let scripted = self.script.lock().pop_front();
            match scripted {
                Some(outcome) => outcome,
                None => Ok(oclive_kernel_contracts::LlmGenerateOutcome {
                    reply: String::new(),
                    prompt_eval_ms: None,
                }),
            }
        }
    }

    /// Recording Agent double: it must never be entered by the selected chain.
    struct RecordingAgent {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait::async_trait]
    impl AgentProvider for RecordingAgent {
        async fn process(&self, _input: AgentInput) -> Result<AgentOutput> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(AgentOutput {
                handled: false,
                reply: String::new(),
            })
        }
    }

    /// Recording Event double: it must never be entered by the selected chain, and it reaches no
    /// model or tool even if it were.
    struct RecordingEvent {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait::async_trait]
    impl EventEstimator for RecordingEvent {
        async fn estimate(
            &self,
            _llm: &Arc<dyn LlmClient>,
            _ollama_model: &str,
            _user_message: &str,
            _user_emotion: &Emotion,
            _personality: &PersonalityVector,
            _personality_source: PersonalitySource,
            _recent_turns: &[(String, String)],
            _recent_events: &[Event],
            _knowledge_augment: Option<&KnowledgeEventAugment>,
        ) -> Result<EventImpactEstimate> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(EventImpactEstimate {
                event_type: EventType::Ignore,
                impact_factor: 0.0,
                confidence: 0.0,
            })
        }
    }

    /// The V1 harness: real builtin Memory/Prompt/Emotion behind counting wrappers, a recording LLM,
    /// and recording Agent/Event doubles.
    struct V1Harness {
        plugins: ResolvedRolePlugins,
        memory_calls: Arc<AtomicUsize>,
        prompt_calls: Arc<AtomicUsize>,
        emotion_calls: Arc<AtomicUsize>,
        agent_calls: Arc<AtomicUsize>,
        event_calls: Arc<AtomicUsize>,
        llm: Arc<RecordingLlm>,
    }

    impl V1Harness {
        fn new(
            script: impl IntoIterator<Item = Result<oclive_kernel_contracts::LlmGenerateOutcome>>,
        ) -> Self {
            let memory_calls = Arc::new(AtomicUsize::new(0));
            let prompt_calls = Arc::new(AtomicUsize::new(0));
            let emotion_calls = Arc::new(AtomicUsize::new(0));
            let agent_calls = Arc::new(AtomicUsize::new(0));
            let event_calls = Arc::new(AtomicUsize::new(0));
            let llm = RecordingLlm::new(script);
            let plugins = ResolvedRolePlugins {
                memory: Arc::new(CountingMemory {
                    inner: oclive_kernel_runtime::domain::query_memory::QueryMemoryRetrieval,
                    calls: Arc::clone(&memory_calls),
                }),
                emotion: Arc::new(CountingEmotion {
                    inner:
                        oclive_kernel_runtime::domain::user_emotion_analyzer::BuiltinUserEmotionAnalyzer,
                    calls: Arc::clone(&emotion_calls),
                }),
                event: Arc::new(RecordingEvent {
                    calls: Arc::clone(&event_calls),
                }),
                prompt: Arc::new(CountingPrompt {
                    inner: oclive_kernel_runtime::domain::prompt_assembler::BuiltinPromptAssembler,
                    calls: Arc::clone(&prompt_calls),
                }),
                llm: Arc::clone(&llm) as Arc<dyn LlmClient>,
                agent: Arc::new(RecordingAgent {
                    calls: Arc::clone(&agent_calls),
                }),
                complex_emotion: Arc::new(
                    crate::domain::complex_emotion::BuiltinKeywordComplexEmotionProvider,
                ),
                slots: None,
                merged_agent_directory_plugin_ids: Vec::new(),
            };
            Self {
                plugins,
                memory_calls,
                prompt_calls,
                emotion_calls,
                agent_calls,
                event_calls,
                llm,
            }
        }

        fn memory_calls(&self) -> usize {
            self.memory_calls.load(Ordering::SeqCst)
        }

        fn prompt_calls(&self) -> usize {
            self.prompt_calls.load(Ordering::SeqCst)
        }

        fn emotion_calls(&self) -> usize {
            self.emotion_calls.load(Ordering::SeqCst)
        }

        /// Agent and Event are outside the selected chain; their counters must stay at zero.
        fn assert_agent_and_event_untouched(&self) {
            assert_eq!(
                self.agent_calls.load(Ordering::SeqCst),
                0,
                "the selected consumer chain must not enter the Agent slot"
            );
            assert_eq!(
                self.event_calls.load(Ordering::SeqCst),
                0,
                "the selected consumer chain must not enter the Event slot"
            );
        }
    }

    fn v1_memories() -> Vec<Memory> {
        vec![
            // Highest weight, but it does not answer the query: the unselected marker below must
            // never reach the prompt through this chain.
            Memory {
                id: "v1-high-unmatched".into(),
                role_id: "v1-role".into(),
                content: "MARKER_UNSELECTED 今天星期三".into(),
                importance: 1.0,
                weight: 1.0,
                created_at: Utc::now(),
                scene_id: Some("v1-scene".into()),
                mention_count: 7,
                accessed_at: None,
            },
            Memory {
                id: "v1-low-matched".into(),
                role_id: "v1-role".into(),
                content: "MARKER_LOW 用户喜欢咖啡".into(),
                importance: 0.2,
                weight: 1.0,
                created_at: Utc::now(),
                scene_id: Some("v1-scene".into()),
                mention_count: 2,
                accessed_at: None,
            },
            Memory {
                id: "v1-mid-matched".into(),
                role_id: "v1-role".into(),
                content: "MARKER_MID 用户喜欢咖啡和茶".into(),
                importance: 0.6,
                weight: 1.0,
                created_at: Utc::now(),
                scene_id: Some("v1-scene".into()),
                mention_count: 3,
                accessed_at: None,
            },
        ]
    }

    /// The Prompt fixture. Its domain fields are prepared by hand for this test: the emotion label
    /// is a fixture string (it is **not** produced from the seven-dimension analyzer below), and the
    /// `Role` is an in-memory default — no role directory is loaded.
    struct V1PromptFixture {
        role: Role,
        personality: PersonalityVector,
        event_type: EventType,
        emotion_label: String,
        scene_label: String,
        scene_detail: String,
        worldview: String,
        life_context: String,
    }

    impl V1PromptFixture {
        fn new() -> Self {
            Self {
                role: Role::default(),
                personality: PersonalityVector {
                    stubbornness: 0.4,
                    clinginess: 0.6,
                    sensitivity: 0.7,
                    assertiveness: 0.5,
                    forgiveness: 0.6,
                    talkativeness: 0.6,
                    warmth: 0.8,
                },
                event_type: EventType::Praise,
                emotion_label: "v1-fixture-happy".to_string(),
                scene_label: "v1-场景".to_string(),
                scene_detail: "v1-场景细节".to_string(),
                worldview: "MARKER_WORLDVIEW 世界观片段".to_string(),
                life_context: String::new(),
            }
        }

        fn input<'a>(&'a self, memories: &'a [Memory], user_input: &'a str) -> PromptInput<'a> {
            PromptInput {
                role: &self.role,
                personality: &self.personality,
                memories,
                user_input,
                user_emotion: self.emotion_label.as_str(),
                user_relation_id: "v1-relation",
                relation_hint: "你们是朋友。",
                relation_before: "Friend",
                favorability_before: 55.0,
                relation_preview: "CloseFriend",
                favorability_preview: 60.0,
                event_type: &self.event_type,
                impact_factor: 0.7,
                scene_label: self.scene_label.as_str(),
                scene_detail: self.scene_detail.as_str(),
                topic_hint_line: "在「v1-场景」下，你们可能会多聊日常。",
                life_context_line: self.life_context.as_str(),
                worldview_snippet: self.worldview.as_str(),
                mutable_personality: "",
                ephemeral_personality: "",
                reply_quality_anchor: "",
                previous_complex_emotion_narrative_hint: "",
                user_identity_template: "",
                user_identity_id: "",
                host_prompt_overlay: "",
                host_state_expression_hint: "",
                relation_transition_hint: "",
                extra_sections: &[],
                persona_override: None,
                previous_assistant_reply: "",
            }
        }
    }

    fn v1_outcome(
        reply: &str,
        prompt_eval_ms: Option<u64>,
    ) -> oclive_kernel_contracts::LlmGenerateOutcome {
        oclive_kernel_contracts::LlmGenerateOutcome {
            reply: reply.to_string(),
            prompt_eval_ms,
        }
    }

    /// The memory block header the real prompt builder emits, written out here as an independent
    /// literal so the assertion does not depend on a production constant.
    const V1_MEMORY_HEADER: &str =
        "关于用户的记忆（已按相关性排序；请勿在回复中复述编号、括号或「重要性」等系统字样）:";

    /// V1-1: the rows the real consumer selects are handed to the real prompt entry unchanged, and
    /// the memory evidence region reflects exactly those rows.
    #[test]
    fn cp_b3_v1_memory_selection_hands_originals_to_the_real_prompt() {
        let h = V1Harness::new([]);
        let fixture = V1PromptFixture::new();
        let candidates = v1_memories();

        let ranked = SlotRunner::rank_memories(
            &h.plugins,
            MemoryRetrievalInput {
                memories: &candidates,
                user_query: "咖啡",
                scene_id: Some("v1-scene"),
                limit: 8,
            },
        )
        .expect("the single builtin consumer answers the query");

        // Selected ids and order, with the original fields intact.
        let ids: Vec<&str> = ranked.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, vec!["v1-mid-matched", "v1-low-matched"]);
        for row in &ranked {
            let original = candidates
                .iter()
                .find(|c| c.id == row.id)
                .expect("selected row comes from the candidate slice");
            assert_eq!(row.role_id, original.role_id);
            assert_eq!(row.importance, original.importance);
            assert_eq!(row.weight, original.weight);
            assert_eq!(row.created_at, original.created_at);
            assert_eq!(row.scene_id, original.scene_id);
            assert_eq!(row.mention_count, original.mention_count);
            assert_eq!(row.accessed_at, original.accessed_at);
            assert_eq!(row.content, original.content);
        }

        // The returned originals go to the real prompt entry as-is (no re-filtering here).
        let input = fixture.input(&ranked, "v1-用户输入");
        let prompt = SlotRunner::build_prompt(&h.plugins, &input).expect("the real prompt entry");

        assert!(prompt.contains(V1_MEMORY_HEADER), "{prompt}");
        assert!(
            prompt.contains("MARKER_MID"),
            "selected row missing: {prompt}"
        );
        assert!(
            prompt.contains("MARKER_LOW"),
            "selected row missing: {prompt}"
        );
        assert!(
            !prompt.contains("MARKER_UNSELECTED"),
            "an unselected original must not reach the prompt: {prompt}"
        );
        assert!(prompt.contains("v1-用户输入"), "{prompt}");

        assert_eq!(h.memory_calls(), 1);
        assert_eq!(h.prompt_calls(), 1);
        assert_eq!(h.llm.entry_count(), 0, "no generation was requested here");
        h.assert_agent_and_event_untouched();
    }

    /// V1-2: the Host fallback (no hit) and the empty-query convention both reach the real prompt as
    /// the full candidate set, the existing `limit` still applies, and nothing sneaks the original
    /// candidates into the prompt when the consumer returned none.
    #[test]
    fn cp_b3_v1_host_fallback_and_all_candidates_reach_the_prompt() {
        let h = V1Harness::new([]);
        let fixture = V1PromptFixture::new();
        let candidates = v1_memories();

        // No hit: the documented Host compatibility fallback is the existing all-candidate weighted
        // selection. It is not a Base retrieval hit, and this test does not call it one.
        let fallback = SlotRunner::rank_memories(
            &h.plugins,
            MemoryRetrievalInput {
                memories: &candidates,
                user_query: "完全无关的词 ZQ",
                scene_id: None,
                limit: 8,
            },
        )
        .expect("fallback");
        let ids: Vec<&str> = fallback.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["v1-high-unmatched", "v1-mid-matched", "v1-low-matched"]
        );
        let input = fixture.input(&fallback, "v1-用户输入");
        let prompt = SlotRunner::build_prompt(&h.plugins, &input).expect("prompt");
        assert!(prompt.contains(V1_MEMORY_HEADER), "{prompt}");
        assert!(prompt.contains("MARKER_UNSELECTED"), "{prompt}");
        assert!(
            prompt.contains("MARKER_MID") && prompt.contains("MARKER_LOW"),
            "{prompt}"
        );

        // Empty (trim-empty) query: every explicit candidate, same documented convention.
        let empty_query = SlotRunner::rank_memories(
            &h.plugins,
            MemoryRetrievalInput {
                memories: &candidates,
                user_query: "   ",
                scene_id: None,
                limit: 8,
            },
        )
        .expect("empty query");
        assert_eq!(
            empty_query
                .iter()
                .map(|m| m.id.as_str())
                .collect::<Vec<_>>(),
            ids
        );
        // CP-B3-V1-R1: this call's own returned slice goes to the real prompt entry — not the
        // candidate list and not the no-hit fallback slice. The header and all three independent
        // markers must therefore appear in the actual output.
        let empty_query_prompt =
            SlotRunner::build_prompt(&h.plugins, &fixture.input(&empty_query, "v1-用户输入"))
                .expect("prompt");
        assert!(
            empty_query_prompt.contains(V1_MEMORY_HEADER),
            "{empty_query_prompt}"
        );
        for marker in ["MARKER_UNSELECTED", "MARKER_MID", "MARKER_LOW"] {
            assert!(
                empty_query_prompt.contains(marker),
                "{marker} missing from the empty-query prompt: {empty_query_prompt}"
            );
        }

        // limit = 1 keeps only the top weighted row of the matched set.
        let limited = SlotRunner::rank_memories(
            &h.plugins,
            MemoryRetrievalInput {
                memories: &candidates,
                user_query: "咖啡",
                scene_id: None,
                limit: 1,
            },
        )
        .expect("limit 1");
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].id, "v1-mid-matched");
        let limited_prompt =
            SlotRunner::build_prompt(&h.plugins, &fixture.input(&limited, "v1-用户输入"))
                .expect("prompt");
        assert!(limited_prompt.contains("MARKER_MID"), "{limited_prompt}");
        assert!(!limited_prompt.contains("MARKER_LOW"), "{limited_prompt}");

        // limit = 0 selects nothing, and the prompt then carries no memory evidence at all: the
        // candidates are not re-injected by any test-side helper.
        let none = SlotRunner::rank_memories(
            &h.plugins,
            MemoryRetrievalInput {
                memories: &candidates,
                user_query: "咖啡",
                scene_id: None,
                limit: 0,
            },
        )
        .expect("limit 0");
        assert!(none.is_empty());
        let empty_prompt =
            SlotRunner::build_prompt(&h.plugins, &fixture.input(&none, "v1-用户输入"))
                .expect("prompt");
        assert!(!empty_prompt.contains(V1_MEMORY_HEADER), "{empty_prompt}");
        for marker in ["MARKER_UNSELECTED", "MARKER_MID", "MARKER_LOW"] {
            assert!(
                !empty_prompt.contains(marker),
                "{marker} must not appear when the consumer returned nothing"
            );
        }

        // Counts describe what this test actually executed: four memory consumer entries (no-hit
        // fallback, empty query, limit 1, limit 0) and four prompt assemblies, because every
        // returned slice above — including the empty-query slice and the empty `limit = 0` slice —
        // was handed to the real prompt entry. These are entry counts of this test, not a whole-turn
        // model-call budget.
        assert_eq!(h.memory_calls(), 4, "one consumer entry per rank call");
        assert_eq!(h.prompt_calls(), 4);
        assert_eq!(h.llm.entry_count(), 0);
        h.assert_agent_and_event_untouched();
    }

    /// V1-3: the real LLM consumer hands the whole prompt and the opts to the endpoint, returns the
    /// outcome unchanged, calls a single implementation exactly once, does not retry a typed
    /// failure, and keeps an empty reply as an empty reply.
    #[tokio::test]
    async fn cp_b3_v1_llm_consumer_receives_the_exact_prompt_and_opts() {
        let h = V1Harness::new([
            Ok(v1_outcome("v1-回复", Some(1234))),
            Ok(v1_outcome("", Some(77))),
            Err(crate::error::AppError::OllamaError(
                "v1-scripted-failure".into(),
            )),
        ]);
        let fixture = V1PromptFixture::new();
        let candidates = v1_memories();
        let ranked = SlotRunner::rank_memories(
            &h.plugins,
            MemoryRetrievalInput {
                memories: &candidates,
                user_query: "咖啡",
                scene_id: None,
                limit: 8,
            },
        )
        .expect("rank");
        let prompt =
            SlotRunner::build_prompt(&h.plugins, &fixture.input(&ranked, "v1-用户输入 甲"))
                .expect("prompt");

        // No opts: `None` stays `None` at this consumer call. CP-B3-V1-R1 wording: this is the call
        // argument handed to `LlmClient::generate_with_opts`, not an HTTP request body — no wire,
        // serialization or transport is observed here.
        let outcome = SlotRunner::generate_llm(&h.plugins, "v1-model", &prompt, None)
            .await
            .expect("generation");
        assert_eq!(outcome.reply, "v1-回复");
        assert_eq!(outcome.prompt_eval_ms, Some(1234));
        let entries = h.llm.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].model, "v1-model");
        assert_eq!(
            entries[0].prompt, prompt,
            "the whole prompt must be handed over verbatim"
        );
        assert!(
            entries[0].opts.is_none(),
            "None must not be replaced by opts"
        );

        // Explicit opts: every field arrives as provided.
        let opts = LlmGenerateOpts {
            keep_alive: Some("v1-keep".to_string()),
            want_metrics: true,
            temperature: Some(0.31),
            top_p: Some(0.82),
            max_output_tokens: Some(321),
            preferred_context_tokens: Some(2048),
        };
        let outcome = SlotRunner::generate_llm(&h.plugins, "v1-model", &prompt, Some(&opts))
            .await
            .expect("generation");
        assert_eq!(outcome.reply, "", "an empty reply stays empty");
        assert_eq!(outcome.prompt_eval_ms, Some(77));
        let entries = h.llm.entries();
        assert_eq!(entries.len(), 2);
        let recorded = entries[1].opts.as_ref().expect("opts were provided");
        assert_eq!(recorded.keep_alive.as_deref(), Some("v1-keep"));
        assert!(recorded.want_metrics);
        assert_eq!(recorded.temperature, Some(0.31));
        assert_eq!(recorded.top_p, Some(0.82));
        assert_eq!(recorded.max_output_tokens, Some(321));
        assert_eq!(recorded.preferred_context_tokens, Some(2048));
        assert_eq!(entries[1].prompt, prompt);

        // A typed failure is returned as it is, and this single-implementation consumer does not
        // retry it. (Nothing here says anything about a full turn's fallback behaviour.)
        let error = SlotRunner::generate_llm(&h.plugins, "v1-model", &prompt, None)
            .await
            .expect_err("the scripted failure must surface");
        match error {
            crate::error::AppError::OllamaError(message) => {
                assert_eq!(message, "v1-scripted-failure");
            }
            other => panic!("the typed error must be preserved: {other:?}"),
        }
        assert_eq!(
            h.llm.entry_count(),
            3,
            "one entry per call, no retry of the failing one"
        );
        assert_eq!(h.llm.generate_calls.load(Ordering::SeqCst), 0);
        assert_eq!(h.llm.tag_calls.load(Ordering::SeqCst), 0);
        assert_eq!(h.memory_calls(), 1);
        assert_eq!(h.prompt_calls(), 1);
        h.assert_agent_and_event_untouched();
    }

    /// V1-4: for one identical `PromptInput`, the two real Prompt entries keep their documented
    /// layout — `full()` is the connection of the two halves, `stable_len()` is the stable-prefix
    /// length, the guardrail stays in the per-turn tail, and the stable head carries no memory
    /// evidence. Byte equality of the two entries is **not** assumed: the two documented layouts
    /// differ, and the byte-level golden for the ordinary layout lives with that entry's own tests.
    #[tokio::test]
    async fn cp_b3_v1_segments_keep_the_documented_layout_for_the_same_input() {
        let h = V1Harness::new([Ok(v1_outcome("v1-segments-回复", None))]);
        let fixture = V1PromptFixture::new();
        let candidates = v1_memories();
        let ranked = SlotRunner::rank_memories(
            &h.plugins,
            MemoryRetrievalInput {
                memories: &candidates,
                user_query: "咖啡",
                scene_id: None,
                limit: 8,
            },
        )
        .expect("rank");
        let input = fixture.input(&ranked, "v1-用户输入 乙");

        let segments = SlotRunner::build_prompt_segments(&h.plugins, &input).expect("segments");
        assert_eq!(
            segments.full(),
            format!("{}{}", segments.stable_prefix, segments.dynamic_suffix)
        );
        assert_eq!(segments.stable_len(), segments.stable_prefix.len());

        // Boundaries, both sides: the stable head carries the worldview snippet and no memory
        // evidence; the per-turn tail carries the memory evidence and the always-appended
        // guardrails. CP-B3-V1-R1 adds the two negative halves, so "only in this segment" is
        // supported by a positive and a negative assertion on the same fixture and the same input
        // rather than by the positive side alone. These boundary facts belong to **this** fixture
        // and this input; they are not a universal claim about every custom template, backend or
        // `PromptInput`.
        assert!(
            segments.stable_prefix.contains("MARKER_WORLDVIEW"),
            "{}",
            segments.stable_prefix
        );
        assert!(
            !segments.dynamic_suffix.contains("MARKER_WORLDVIEW"),
            "the worldview snippet belongs to the stable head only: {}",
            segments.dynamic_suffix
        );
        assert!(!segments.stable_prefix.contains("MARKER_MID"));
        assert!(!segments.stable_prefix.contains(V1_MEMORY_HEADER));
        assert!(
            !segments.stable_prefix.contains("【对话硬约束】"),
            "the always-appended guardrails belong to the per-turn tail only: {}",
            segments.stable_prefix
        );
        assert!(segments.dynamic_suffix.contains(V1_MEMORY_HEADER));
        assert!(segments.dynamic_suffix.contains("MARKER_MID"));
        assert!(segments.dynamic_suffix.contains("【对话硬约束】"));
        assert!(segments.full().contains("【对话硬约束】"));
        assert!(segments.full().contains(V1_MEMORY_HEADER));
        assert!(segments.full().contains("v1-用户输入 乙"));

        // The ordinary entry for the same input keeps the same required content (its own byte
        // layout is covered by that entry's golden tests).
        let ordinary = SlotRunner::build_prompt(&h.plugins, &input).expect("ordinary entry");
        assert!(ordinary.contains("【对话硬约束】"));
        assert!(ordinary.contains(V1_MEMORY_HEADER));
        assert!(ordinary.contains("MARKER_WORLDVIEW"));
        assert!(ordinary.contains("v1-用户输入 乙"));

        // If the segment string is what the consumer sends, that whole string is what the endpoint
        // receives.
        let outcome = SlotRunner::generate_llm(&h.plugins, "v1-model", &segments.full(), None)
            .await
            .expect("generation");
        assert_eq!(outcome.reply, "v1-segments-回复");
        let entries = h.llm.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].prompt, segments.full());
        assert_eq!(h.memory_calls(), 1);
        assert_eq!(
            h.prompt_calls(),
            1,
            "`build_prompt_segments` is the builtin-only path and does not re-enter the assembler consumer"
        );
        h.assert_agent_and_event_untouched();
    }

    /// V1-5: the real Emotion consumer returns the builtin seven-dimension result for a lexicon hit
    /// and its compatible fallback for material with no hit.
    ///
    /// CP-B3-V1-R1 wording: the seven-dimension analysis and the tone label that reaches the prompt
    /// are **two separately tested facts**. The label below is prepared by the fixture; no
    /// seven-dimension→label conversion edge is exercised or verified here, and this test does not
    /// turn this consumer into a link of one fully connected product chain.
    #[test]
    fn cp_b3_v1_emotion_consumer_returns_builtin_seven_dimensions() {
        let h = V1Harness::new([]);
        let fixture = V1PromptFixture::new();

        let hit =
            SlotRunner::analyze_emotion(&h.plugins, "我很开心").expect("real builtin analysis");
        assert_eq!(
            [
                hit.joy,
                hit.sadness,
                hit.anger,
                hit.fear,
                hit.surprise,
                hit.disgust,
                hit.neutral
            ],
            [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
        );
        assert!(hit.extension.is_none());

        // No lexicon hit keeps the existing compatible fallback; this test does not argue that the
        // fallback is the better semantic choice, and the production value is unchanged.
        let no_hit =
            SlotRunner::analyze_emotion(&h.plugins, "今天星期三").expect("real builtin analysis");
        assert_eq!(
            [
                no_hit.joy,
                no_hit.sadness,
                no_hit.anger,
                no_hit.fear,
                no_hit.surprise,
                no_hit.disgust,
                no_hit.neutral
            ],
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]
        );
        assert!(no_hit.extension.is_none());
        assert_eq!(h.emotion_calls(), 2);

        // The prompt's tone line reflects the label the fixture supplies; the seven dimensions above
        // are not converted into that label by this chain.
        let prompt = SlotRunner::build_prompt(&h.plugins, &fixture.input(&[], "v1-用户输入 丙"))
            .expect("prompt");
        assert!(prompt.contains("【角色当前状态】"), "{prompt}");
        assert!(
            prompt.contains("用户语气线索：v1-fixture-happy。"),
            "the fixture-prepared label must reach the tone block: {prompt}"
        );
        assert_eq!(h.llm.entry_count(), 0);
        h.assert_agent_and_event_untouched();
    }

    /// V1-6: two **sequential** neighbouring consumer combinations keep their own values, and the
    /// counters separate the memory, prompt and generation entries.
    ///
    /// CP-B3-V1-R1 wording: what is local here is narrower than "each combination uses its own local
    /// query/material/model/opts". The query, model and `user_input` are string **literals**; the
    /// candidate rows are held by one local `Vec` that both combinations reuse; the assembled prompt
    /// and chain B's opts are local values. No cross-`Pending` borrowing, no lifetime evidence and no
    /// concurrency evidence is claimed — the two combinations are executed one after the other. The
    /// isolation assertions themselves stand: each combination's own inputs, selection and generation
    /// entry stay separate.
    ///
    /// This is two combinations of the same consumer methods, not a Kernel stage order and not a
    /// claim about how many model calls a whole turn makes: each combination here performs exactly
    /// one generation entry, and nothing is claimed beyond the calls this test makes. The counters
    /// below are entry counts of this test, not a whole-turn model-call budget.
    #[tokio::test]
    async fn cp_b3_v1_two_consumer_chains_do_not_share_values() {
        let h = V1Harness::new([
            Ok(v1_outcome("v1-回复 甲", Some(11))),
            Ok(v1_outcome("v1-回复 乙", Some(22))),
        ]);
        let fixture = V1PromptFixture::new();
        let candidates = v1_memories();

        let ranked_a = SlotRunner::rank_memories(
            &h.plugins,
            MemoryRetrievalInput {
                memories: &candidates,
                user_query: "咖啡",
                scene_id: None,
                limit: 8,
            },
        )
        .expect("chain A rank");
        let prompt_a =
            SlotRunner::build_prompt(&h.plugins, &fixture.input(&ranked_a, "v1-输入 甲"))
                .expect("chain A prompt");
        let outcome_a = SlotRunner::generate_llm(&h.plugins, "v1-model-A", &prompt_a, None)
            .await
            .expect("chain A generation");

        let ranked_b = SlotRunner::rank_memories(
            &h.plugins,
            MemoryRetrievalInput {
                memories: &candidates,
                user_query: "茶",
                scene_id: None,
                limit: 8,
            },
        )
        .expect("chain B rank");
        let prompt_b =
            SlotRunner::build_prompt(&h.plugins, &fixture.input(&ranked_b, "v1-输入 乙"))
                .expect("chain B prompt");
        let opts_b = LlmGenerateOpts {
            keep_alive: Some("v1-keep-B".to_string()),
            want_metrics: false,
            temperature: Some(0.11),
            top_p: None,
            max_output_tokens: Some(64),
            preferred_context_tokens: None,
        };
        let outcome_b =
            SlotRunner::generate_llm(&h.plugins, "v1-model-B", &prompt_b, Some(&opts_b))
                .await
                .expect("chain B generation");

        // Selection follows each chain's own query and stays inside that chain: the single-scalar
        // query "茶" selects only the row that actually contains it, while "咖啡" selects both
        // coffee rows.
        assert_eq!(
            ranked_a.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
            vec!["v1-mid-matched", "v1-low-matched"]
        );
        assert_eq!(
            ranked_b.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
            vec!["v1-mid-matched"]
        );
        assert!(prompt_a.contains("v1-输入 甲") && !prompt_a.contains("v1-输入 乙"));
        assert!(prompt_b.contains("v1-输入 乙") && !prompt_b.contains("v1-输入 甲"));
        assert_ne!(prompt_a, prompt_b, "each chain builds its own prompt");

        // The generation entries kept their own model, prompt and opts, in order.
        assert_eq!(outcome_a.reply, "v1-回复 甲");
        assert_eq!(outcome_a.prompt_eval_ms, Some(11));
        assert_eq!(outcome_b.reply, "v1-回复 乙");
        assert_eq!(outcome_b.prompt_eval_ms, Some(22));
        let entries = h.llm.entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].model, "v1-model-A");
        assert_eq!(entries[0].prompt, prompt_a);
        assert!(entries[0].opts.is_none());
        assert_eq!(entries[1].model, "v1-model-B");
        assert_eq!(entries[1].prompt, prompt_b);
        let recorded_b = entries[1].opts.as_ref().expect("chain B passed opts");
        assert_eq!(recorded_b.keep_alive.as_deref(), Some("v1-keep-B"));
        assert!(!recorded_b.want_metrics);
        assert_eq!(recorded_b.temperature, Some(0.11));
        assert_eq!(recorded_b.max_output_tokens, Some(64));

        // Counters separate the three consumer kinds: two memory selections, two prompt assemblies
        // and two generation entries — not one shared counter and not a per-turn budget.
        assert_eq!(h.memory_calls(), 2);
        assert_eq!(h.prompt_calls(), 2);
        assert_eq!(h.llm.entry_count(), 2);
        assert_eq!(h.llm.generate_calls.load(Ordering::SeqCst), 0);
        assert_eq!(h.llm.tag_calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            h.emotion_calls(),
            0,
            "neither chain asked for an emotion result"
        );
        h.assert_agent_and_event_untouched();
    }
}
