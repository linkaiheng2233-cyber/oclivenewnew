//! 蓝图 v2 多实例槽位在共景阶段表内的串行合并（P4，RFC §4.2）。

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
use crate::domain::prompt_assembler::PromptAssembler;
use crate::domain::prompt_builder::PromptInput;
use crate::domain::slot_resolver::ResolvedRoleSlots;
use crate::domain::user_emotion_analyzer::UserEmotionAnalyzer;
use crate::error::{AppError, Result};
use crate::infrastructure::llm::LlmClient;
use crate::models::knowledge::KnowledgeEventAugment;
use crate::models::{Emotion, Event, Memory, PersonalitySource, PersonalityVector, Role};
use std::collections::HashSet;
use std::sync::Arc;

pub struct SlotRunner;

impl SlotRunner {
    /// 折叠六槽 LLM，或 registry 中 `position` 最大的 `llm` 实例。
    #[must_use]
    pub fn primary_llm(pl: &ResolvedRolePlugins) -> Arc<dyn LlmClient> {
        pl.slots
            .as_ref()
            .and_then(|s| s.llm.last().map(|(_, l)| Arc::clone(l)))
            .unwrap_or_else(|| Arc::clone(&pl.llm))
    }

    /// `emotion`：串行调用，**last-wins**（≥2 实例）；单实例用 registry 条目。
    pub fn analyze_emotion(pl: &ResolvedRolePlugins, text: &str) -> Result<EmotionResult> {
        if let Some(instances) = registry_instances(&pl.slots, |s| &s.emotion) {
            if instances.len() >= 2 {
                return Self::emotion_last_wins(instances, text);
            }
            return instances[0].1.analyze(text);
        }
        pl.emotion.analyze(text)
    }

    /// `complex_emotion`：串行，**last-wins**。
    pub fn resolve_complex_emotion(
        pl: &ResolvedRolePlugins,
        input: &ComplexEmotionInput,
    ) -> Result<ComplexEmotionOutput> {
        if let Some(instances) = registry_instances(&pl.slots, |s| &s.complex_emotion) {
            if instances.len() >= 2 {
                return Self::complex_emotion_last_wins(instances, input);
            }
            return instances[0].1.resolve_turn(input);
        }
        pl.complex_emotion.resolve_turn(input)
    }

    /// `event`：串行估计，**last-wins**（中间实例打 debug 日志）。
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
        if let Some(instances) = registry_instances(&pl.slots, |s| &s.event) {
            if instances.len() >= 2 {
                return Self::event_last_wins(
                    instances,
                    &llm,
                    ollama_model,
                    user_message,
                    user_emotion,
                    personality,
                    personality_source,
                    recent_turns,
                    recent_events,
                    knowledge_augment,
                )
                .await;
            }
            return instances[0]
                .1
                .estimate(
                    &llm,
                    ollama_model,
                    user_message,
                    user_emotion,
                    personality,
                    personality_source,
                    recent_turns,
                    recent_events,
                    knowledge_augment,
                )
                .await;
        }
        pl.event
            .estimate(
                &llm,
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
    }

    /// `memory`：串行 rank → 按 id 去重 → 按 `importance * weight` 统一排序。
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
        if let Some(instances) = registry_instances(&pl.slots, |s| &s.prompt) {
            if instances.len() >= 2 {
                return Self::prompt_top_topic_last_wins(instances, role, scene_id);
            }
            return instances[0].1.top_topic_hint(role, scene_id);
        }
        pl.prompt.top_topic_hint(role, scene_id)
    }

    /// `prompt`：`build_prompt` **last-wins**。
    pub fn build_prompt(pl: &ResolvedRolePlugins, input: &PromptInput<'_>) -> Result<String> {
        if let Some(instances) = registry_instances(&pl.slots, |s| &s.prompt) {
            if instances.len() >= 2 {
                return Self::prompt_build_last_wins(instances, input);
            }
            return instances[0].1.build_prompt(input);
        }
        pl.prompt.build_prompt(input)
    }

    /// `llm`：串行 **全部调用**（打日志），**last-wins** 作为最终回复。
    pub async fn generate_llm(
        pl: &ResolvedRolePlugins,
        ollama_model: &str,
        prompt: &str,
    ) -> Result<String> {
        if let Some(instances) = registry_instances(&pl.slots, |s| &s.llm) {
            if instances.len() >= 2 {
                return Self::llm_serial_last_wins(instances, ollama_model, prompt).await;
            }
            return instances[0].1.generate(ollama_model, prompt).await;
        }
        pl.llm.generate(ollama_model, prompt).await
    }

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
        last.ok_or_else(|| AppError::OllamaError("no emotion slot produced a result".into()))
    }

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
            AppError::OllamaError("no complex_emotion slot produced a result".into())
        })
    }

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
        last.ok_or_else(|| AppError::OllamaError("no event slot produced a result".into()))
    }

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
        last.ok_or_else(|| AppError::OllamaError("no prompt slot produced a result".into()))
    }

    async fn llm_serial_last_wins(
        instances: &[(String, Arc<dyn LlmClient>)],
        ollama_model: &str,
        prompt: &str,
    ) -> Result<String> {
        if instances.len() == 1 {
            return instances[0].1.generate(ollama_model, prompt).await;
        }
        let mut last = String::new();
        let mut any_ok = false;
        for (key, llm) in instances {
            match llm.generate(ollama_model, prompt).await {
                Ok(reply) => {
                    tracing::info!(
                        target: "oclive_plugin",
                        slot_key = %key,
                        reply_len = reply.len(),
                        "llm_generate slot (serial; last-wins)"
                    );
                    last = reply;
                    any_ok = true;
                }
                Err(e) => {
                    tracing::warn!(
                        target: "oclive_plugin",
                        slot_key = %key,
                        err = %e,
                        "llm_generate slot failed"
                    );
                }
            }
        }
        if any_ok {
            Ok(last)
        } else {
            Err(AppError::OllamaError("no llm slot produced a reply".into()))
        }
    }
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
    use crate::domain::memory_retrieval::{BuiltinMemoryRetrieval, BuiltinMemoryRetrievalV2};
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
            },
            Memory {
                id: "b".into(),
                role_id: "r".into(),
                content: "beta".into(),
                importance: 0.5,
                weight: 1.0,
                created_at: Utc::now(),
                scene_id: None,
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
                Arc::new(BuiltinMemoryRetrievalV2) as Arc<dyn MemoryRetrieval>,
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
