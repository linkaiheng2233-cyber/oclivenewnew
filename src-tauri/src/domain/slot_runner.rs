//! # 蓝图 v2 多实例槽位执行器（`SlotRunner`）
//!
//! **角色**：当 `slot_registry` 中同一 `slot_type` 有多个实例时，按类型选择**合并策略**（串行 last-wins、记忆合并去重、LLM 串行等）并调用对应 `dyn` 实现。
//!
//! **上游**：[`SlotResolver`](../slot_resolver.rs) 产出 `ResolvedRoleSlots`；[`PluginHost`](../plugin_host.rs) 提供 `BackendRegistry`。
//! **下游**：`co_present` 各阶段（情绪、事件、记忆排序、Prompt、LLM）；**Agent 多目录插件合并**在 `PluginHost` / `SlotResolver::wrap_agent_if_merged`，不在本文件。
//!
//! **关键决策**：合并策略按槽位语义选择（见各 `*_last_wins` / `memory_merge_rank` 函数头注释）——例如记忆需**去重合并**，LLM 只需**最终回复**，避免一刀切并行导致上下文错乱。

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
use crate::error::Result;
use crate::domain::ports::LlmClient;
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
    ///
    /// **为何 last-wins**：情绪分析更新的是「当前用户情绪状态」，中间态无需保留；最后一次分析覆盖前序结果即可。
    /// **为何不用并行**：各分析器输入相同、输出互斥，并行只会浪费算力且增加合并歧义。
    /// **局限**：前序实例失败时仅打日志，仍可能无有效结果（见 `emotion_last_wins`）。
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

    /// 情绪链 **last-wins** 实现：按 `position` 顺序串行，保留最后一次成功结果。
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
            crate::domain::error_helpers::ollama_msg("emotion", "no slot produced a result")
        })
    }

    /// **complex_emotion last-wins**：与 emotion 相同——叙事提示取最后一次成功 `resolve_turn`。
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
                "complex_emotion",
                "no slot produced a result",
            )
        })
    }

    /// **event 串行 last-wins**：多事件检测器依次估计，保留最后一次成功结果。
    ///
    /// **解决的问题**：不同检测器可能对同一回合打出重复或冲突的事件标签。
    /// **为何 last-wins**：事件影响用于后续性格/记忆策略，只需**一个**归一化估计；中间态打 debug 日志。
    /// **局限**：前序检测器的补充信号不会合并，仅最后一路生效。
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
            crate::domain::error_helpers::ollama_msg("event", "no slot produced a result")
        })
    }

    /// **memory 串行合并去重**：多路检索结果按 `memory.id` 去重后按 importance×weight 排序截断。
    ///
    /// **解决的问题**：同一事件可能被多个记忆实例（不同 provider）重复召回。
    /// **为何串行而非并行**：后续实例可能依赖已写入的排序启发式；且合并逻辑需全局去重集。
    /// **为何不用 last-wins**：用户需要**并集**而非单一路径的 Top-K。
    /// **局限**：单实例失败会跳过该路，可能漏召回；`limit` 在合并后统一 truncate。
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

    /// **prompt top_topic last-wins**：多组装器的 `top_topic_hint` 串行，保留最后一个 `Some`。
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

    /// **prompt last-wins**：多组装器依次 `build_prompt`，最终字符串为最后一次成功输出。
    ///
    /// **解决的问题**：创作者可叠多个 Prompt 插件做实验，但发往 LLM 的只能有一份文本。
    /// **为何 last-wins**：Prompt 是**构建最终上下文**的流水线末端，后写覆盖前写符合「最后一道加工」直觉。
    /// **局限**：无法自动拼接多段 Prompt；需单实例内自行合并。
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
            crate::domain::error_helpers::ollama_msg("prompt", "no slot produced a result")
        })
    }

    /// **llm 串行 last-wins**：多 LLM 实例依次对**同一 prompt** 生成，仅保留最后一次成功回复。
    ///
    /// **解决的问题**：蓝图允许配置多个 LLM 槽（如主模型 + 备用），运行时只需**一条**用户可见回复。
    /// **为何串行**：各调用共享同一 prompt 上下文，无链式依赖时也避免并发打爆 GPU/配额。
    /// **为何 last-wins**：与「最终展示回复」语义一致；前序成功结果仅作日志对比。
    /// **局限**：不是 ensemble 投票；失败实例被跳过，若全部失败则返回错误。
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
            Err(crate::domain::error_helpers::ollama_msg(
                "llm",
                "no slot produced a reply",
            ))
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

/// 共景编排对槽位合并器的端口；[`crate::domain::chat_engine::co_present`] 仅通过本 trait 调用。
#[async_trait::async_trait]
pub trait CoPresentSlotRunner: Send + Sync {
    fn analyze_emotion(&self, pl: &ResolvedRolePlugins, text: &str) -> Result<EmotionResult>;
    fn resolve_complex_emotion(
        &self,
        pl: &ResolvedRolePlugins,
        input: &ComplexEmotionInput,
    ) -> Result<ComplexEmotionOutput>;
    async fn estimate_event(
        &self,
        pl: &ResolvedRolePlugins,
        ollama_model: &str,
        user_message: &str,
        user_emotion: &Emotion,
        personality: &PersonalityVector,
        personality_source: PersonalitySource,
        recent_turns: &[(String, String)],
        recent_events: &[Event],
        knowledge_augment: Option<&KnowledgeEventAugment>,
    ) -> Result<EventImpactEstimate>;
    fn rank_memories(
        &self,
        pl: &ResolvedRolePlugins,
        input: MemoryRetrievalInput<'_>,
    ) -> Result<Vec<Memory>>;
    fn top_topic_hint(&self, pl: &ResolvedRolePlugins, role: &Role, scene_id: &str) -> Option<String>;
    fn build_prompt(&self, pl: &ResolvedRolePlugins, input: &PromptInput<'_>) -> Result<String>;
    async fn generate_llm(&self, pl: &ResolvedRolePlugins, model: &str, prompt: &str)
        -> Result<String>;
    fn primary_llm(&self, pl: &ResolvedRolePlugins) -> Arc<dyn LlmClient>;
}

#[async_trait::async_trait]
impl CoPresentSlotRunner for SlotRunner {
    fn analyze_emotion(&self, pl: &ResolvedRolePlugins, text: &str) -> Result<EmotionResult> {
        SlotRunner::analyze_emotion(pl, text)
    }

    fn resolve_complex_emotion(
        &self,
        pl: &ResolvedRolePlugins,
        input: &ComplexEmotionInput,
    ) -> Result<ComplexEmotionOutput> {
        SlotRunner::resolve_complex_emotion(pl, input)
    }

    async fn estimate_event(
        &self,
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
        SlotRunner::estimate_event(
            pl,
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

    fn rank_memories(
        &self,
        pl: &ResolvedRolePlugins,
        input: MemoryRetrievalInput<'_>,
    ) -> Result<Vec<Memory>> {
        SlotRunner::rank_memories(pl, input)
    }

    fn top_topic_hint(&self, pl: &ResolvedRolePlugins, role: &Role, scene_id: &str) -> Option<String> {
        SlotRunner::top_topic_hint(pl, role, scene_id)
    }

    fn build_prompt(&self, pl: &ResolvedRolePlugins, input: &PromptInput<'_>) -> Result<String> {
        SlotRunner::build_prompt(pl, input)
    }

    async fn generate_llm(
        &self,
        pl: &ResolvedRolePlugins,
        model: &str,
        prompt: &str,
    ) -> Result<String> {
        SlotRunner::generate_llm(pl, model, prompt).await
    }

    fn primary_llm(&self, pl: &ResolvedRolePlugins) -> Arc<dyn LlmClient> {
        SlotRunner::primary_llm(pl)
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
