//! Expert routing match and step execution (`slot.expert.invoke`).
//!
//! OPTIONAL (2026-07-24): still gated behind `dual_core` and not compiled by
//! default. The LoRA path is active only for role-predeclared, authorized LLM
//! directory plugins; other expert actions retain their existing behavior.

#![cfg(feature = "dual_core")]

use std::collections::{BTreeMap, HashMap};

use chrono::Timelike;
use oclive_validation::{
    parse_expert_step_action, select_expert_route, slot_registry_entry_in_zone, ExpertFallback,
    ExpertMatchContext, ExpertRoute, ExpertRouteStep, ExpertRoutingDoc, ExpertStepActionKind,
    PipelineStep, SlotRegistryEntry,
};

use crate::domain::chat_engine::message_error::ProcessMessageError;
use crate::domain::dual_pipeline::topological_sort_pipeline_steps;
use crate::domain::dual_pipeline_steps::{ExperimentalStepCtx, StepOutcome};
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::error::AppError;
use crate::models::PersonalityVector;
use crate::state::AppState;

fn map_db_err(e: AppError) -> ProcessMessageError {
    ProcessMessageError::dual_core(e)
}

/// Trigger miss: no route matched; skip silently (not an execution failure).
#[derive(Debug, Clone)]
pub struct ExpertTriggerMiss;

/// Predeclared directory-backed LLM instance selected by `slot.lora.apply`.
///
/// The plugin owns framework-specific LoRA loading (llama.cpp, vLLM, PEFT,
/// etc.); expert routing only selects an authorized, role-declared instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LoraLlmSelection {
    pub slot_key: String,
    pub plugin_id: String,
    pub entry: SlotRegistryEntry,
}

/// Resolve one LoRA plugin id to exactly one effective `llm` directory slot.
///
/// # Errors
///
/// Returns an error when the plugin is not declared by the role/session,
/// is not an LLM directory slot, or is declared ambiguously.
pub(crate) fn resolve_lora_llm_selection(
    registry: &BTreeMap<String, SlotRegistryEntry>,
    plugin_id: &str,
) -> Result<LoraLlmSelection, String> {
    let plugin_id = plugin_id.trim();
    if plugin_id.is_empty() {
        return Err("slot.lora.apply requires params.plugin_id".into());
    }

    let mut matches = registry
        .iter()
        .filter(|(_, entry)| {
            entry.slot_type.trim() == "llm"
                && entry.backend.trim() == "directory"
                && slot_registry_entry_in_zone(entry, "experimental")
                && !slot_registry_entry_in_zone(entry, "stable")
                && entry
                    .plugin
                    .as_deref()
                    .is_some_and(|id| id.trim() == plugin_id)
        })
        .map(|(slot_key, entry)| LoraLlmSelection {
            slot_key: slot_key.clone(),
            plugin_id: plugin_id.to_string(),
            entry: entry.clone(),
        });

    let selected = matches.next().ok_or_else(|| {
        format!(
            "LoRA plugin `{plugin_id}` must be predeclared as one experimental-only type=llm, backend=directory slot"
        )
    })?;
    if matches.next().is_some() {
        return Err(format!(
            "LoRA plugin `{plugin_id}` is declared by multiple llm slots; use one unique slot"
        ));
    }
    Ok(selected)
}

/// Snapshot before expert step execution (rollback on failure).
#[derive(Debug, Clone, Default)]
pub struct ExpertExecSnapshot {
    pub personality_before: Option<PersonalityVector>,
    pub injected_memory_ids: Vec<String>,
    pub lora_plugin_id: Option<String>,
    pub prompt_fragment_before: Option<String>,
}

impl ExpertExecSnapshot {
    pub async fn restore(&self, state: &AppState, srid: &str, role_id: &str) {
        if let Some(ref p) = self.personality_before {
            let _ = state
                .db_manager
                .save_personality_vector(srid, p, "expert_rollback")
                .await;
            state.invalidate_personality_cache_for_role(role_id);
        }
        for id in &self.injected_memory_ids {
            let _ = state.db_manager.delete_memory(id).await;
        }
        state.session_cache.clear_expert_injected_memories(srid);
        state
            .session_cache
            .set_expert_lora_plugin(srid, self.lora_plugin_id.clone());
        state.session_cache.set_expert_prompt_enhance(
            srid,
            self.prompt_fragment_before.clone().unwrap_or_default(),
        );
    }
}

async fn build_match_context(
    ctx: &ExperimentalStepCtx<'_>,
) -> Result<ExpertMatchContext, ProcessMessageError> {
    let user_emotion = ctx
        .state
        .db_manager
        .get_current_emotion(ctx.srid)
        .await
        .ok()
        .flatten();
    let user_relation = resolve_effective_user_relation_key(
        ctx.state,
        ctx.role,
        ctx.srid,
        Some(ctx.scene_id.as_str()),
    )
    .await
    .map_err(map_db_err)?;
    let virtual_time_ms = ctx
        .state
        .db_manager
        .get_virtual_time_ms(ctx.srid)
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    let now = chrono::Local::now();
    Ok(ExpertMatchContext {
        scene_id: ctx.scene_id.clone(),
        user_message: ctx.user_message.to_string(),
        user_emotion,
        user_relation: Some(user_relation),
        virtual_time_ms,
        wall_clock_hour_minute: (now.hour(), now.minute()),
    })
}

fn route_label(route: &ExpertRoute) -> String {
    route
        .id
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "expert-route".into())
}

fn log_trigger_miss(srid: &str, doc: &ExpertRoutingDoc) {
    tracing::debug!(
        target: "oclive_expert",
        session_ns = %srid,
        route_count = doc.routes.len(),
        "专家路由：无触发匹配，跳过专家流程"
    );
}

fn log_exec_fallback(route_id: &str, reason: &str, fallback: ExpertFallback) {
    tracing::warn!(
        target: "oclive_expert",
        route_id = %route_id,
        fallback = ?fallback,
        reason = %reason,
        fallback_hint = %format!("route={route_id}; reason={reason}"),
        "专家流程执行失败，应用降级"
    );
}

/// Run expert sub-pipeline; returns `TriggerMiss` when no route matches.
///
/// # Errors
///
/// Propagates when match context build fails or an expert step returns `ProcessMessageError`.
pub async fn execute_expert_route(
    step_ctx: &mut ExperimentalStepCtx<'_>,
    doc: &ExpertRoutingDoc,
) -> Result<Result<StepOutcome, ExpertTriggerMiss>, ProcessMessageError> {
    let match_ctx = build_match_context(step_ctx).await?;
    let Some(route) = select_expert_route(doc, &match_ctx) else {
        log_trigger_miss(step_ctx.srid, doc);
        return Ok(Err(ExpertTriggerMiss));
    };
    if route.steps.is_empty() {
        return Ok(Ok(StepOutcome::Continue));
    }

    let route_id = route_label(route);
    let pipeline_steps: Vec<PipelineStep> = route
        .steps
        .iter()
        .map(|s: &ExpertRouteStep| PipelineStep {
            action: s.action.clone(),
            depends_on: s.depends_on.clone(),
        })
        .collect();

    let ordered = match topological_sort_pipeline_steps(&pipeline_steps) {
        Ok(o) => o,
        Err(e) => {
            tracing::warn!(
                target: "oclive_expert",
                session_ns = %step_ctx.srid,
                route_id = %route_id,
                error = %e,
                "专家路由 steps 拓扑排序失败，跳过"
            );
            return Ok(Ok(StepOutcome::Continue));
        }
    };

    let step_by_action: HashMap<&str, &ExpertRouteStep> =
        route.steps.iter().map(|s| (s.action.as_str(), s)).collect();

    let fallback = doc.fallback_mode();
    let mut wants_stable_completion = false;
    let mut snapshot = ExpertExecSnapshot::default();

    for pstep in ordered {
        let Some(rstep) = step_by_action.get(pstep.action.as_str()) else {
            continue;
        };
        let outcome = run_expert_step(step_ctx, rstep, &mut snapshot).await;
        match outcome {
            Ok(StepOutcome::Continue) => {}
            Ok(StepOutcome::NeedsStableCompletion) => wants_stable_completion = true,
            Ok(StepOutcome::AgentComplete(resp)) => {
                return Ok(Ok(StepOutcome::AgentComplete(resp)))
            }
            Ok(StepOutcome::Failed(msg)) => {
                snapshot
                    .restore(step_ctx.state, step_ctx.srid, step_ctx.role.id.as_str())
                    .await;
                log_exec_fallback(&route_id, &msg, fallback);
                return Ok(Ok(apply_expert_fallback(
                    step_ctx, fallback, &route_id, &msg,
                )
                .await?));
            }
            Err(e) => {
                let msg = e.to_string();
                snapshot
                    .restore(step_ctx.state, step_ctx.srid, step_ctx.role.id.as_str())
                    .await;
                log_exec_fallback(&route_id, &msg, fallback);
                return Ok(Ok(apply_expert_fallback(
                    step_ctx, fallback, &route_id, &msg,
                )
                .await?));
            }
        }
    }

    Ok(Ok(if wants_stable_completion {
        StepOutcome::NeedsStableCompletion
    } else {
        StepOutcome::Continue
    }))
}

async fn run_expert_step(
    ctx: &mut ExperimentalStepCtx<'_>,
    step: &ExpertRouteStep,
    snap: &mut ExpertExecSnapshot,
) -> Result<StepOutcome, ProcessMessageError> {
    let kind = parse_expert_step_action(step.action.as_str())
        .map_err(ProcessMessageError::dual_core_invalid)?;
    match kind {
        ExpertStepActionKind::PersonalityAdjust => {
            run_personality_adjust(ctx, step, snap).await?;
            Ok(StepOutcome::Continue)
        }
        ExpertStepActionKind::PromptEnhanceApply => {
            run_prompt_enhance(ctx, step, snap)?;
            Ok(StepOutcome::Continue)
        }
        ExpertStepActionKind::MemoryInject => {
            run_memory_inject(ctx, step, snap).await?;
            Ok(StepOutcome::Continue)
        }
        ExpertStepActionKind::LoraApply => run_lora_apply(ctx, step, snap).await,
        ExpertStepActionKind::ExpertFallback => Ok(apply_expert_fallback(
            ctx,
            ExpertFallback::Skip,
            "expert-route",
            "slot.expert.fallback",
        )
        .await?),
        ExpertStepActionKind::Slot {
            registry_key,
            method,
        } => ctx.run_method(&registry_key, method.as_str()).await,
    }
}

async fn run_personality_adjust(
    ctx: &mut ExperimentalStepCtx<'_>,
    step: &ExpertRouteStep,
    snap: &mut ExpertExecSnapshot,
) -> Result<(), ProcessMessageError> {
    let params = step.params.as_ref().map(|p| &p.0);
    let (trait_name, delta) = parse_trait_delta(params)?;
    let personality = ctx
        .state
        .get_current_personality(ctx.srid, ctx.role)
        .await
        .map_err(map_db_err)?;
    if snap.personality_before.is_none() {
        snap.personality_before = Some(personality.clone());
    }
    let mut adjusted = personality.clone();
    apply_trait_delta(&mut adjusted, trait_name.as_str(), delta);
    adjusted.clamp(&ctx.role.evolution_bounds);
    ctx.state
        .db_manager
        .save_personality_vector(ctx.srid, &adjusted, "expert_personality_adjust")
        .await
        .map_err(map_db_err)?;
    ctx.state
        .invalidate_personality_cache_for_role(ctx.role.id.as_str());
    Ok(())
}

fn run_prompt_enhance(
    ctx: &mut ExperimentalStepCtx<'_>,
    step: &ExpertRouteStep,
    snap: &mut ExpertExecSnapshot,
) -> Result<(), ProcessMessageError> {
    let text = step
        .params
        .as_ref()
        .and_then(|p| p.0.get("text").and_then(|v| v.as_str()))
        .unwrap_or("")
        .trim()
        .to_string();
    if text.is_empty() {
        return Err(ProcessMessageError::dual_core_invalid(
            "slot.prompt_enhance.apply 需要 params.text",
        ));
    }
    if snap.prompt_fragment_before.is_none() {
        snap.prompt_fragment_before = Some(ctx.state.session_cache.expert_prompt_enhance(ctx.srid));
    }
    ctx.state
        .session_cache
        .set_expert_prompt_enhance(ctx.srid, text);
    Ok(())
}

async fn run_memory_inject(
    ctx: &mut ExperimentalStepCtx<'_>,
    step: &ExpertRouteStep,
    snap: &mut ExpertExecSnapshot,
) -> Result<(), ProcessMessageError> {
    let content = step
        .params
        .as_ref()
        .and_then(|p| p.0.get("content").and_then(|v| v.as_str()))
        .unwrap_or("")
        .trim()
        .to_string();
    if content.is_empty() {
        return Err(ProcessMessageError::dual_core_invalid(
            "slot.memory.inject 需要 params.content",
        ));
    }
    let importance = step
        .params
        .as_ref()
        .and_then(|p| p.0.get("importance").and_then(|v| v.as_f64()))
        .unwrap_or(0.85);
    let id = ctx
        .state
        .db_manager
        .save_memory_merged(
            ctx.srid,
            content.as_str(),
            importance,
            ctx.role.pack_memory_config.similarity_threshold,
            ctx.scene_id.as_str(),
        )
        .await
        .map_err(map_db_err)?;
    snap.injected_memory_ids.push(id.clone());
    ctx.state
        .session_cache
        .push_expert_injected_memory(ctx.srid, id);
    Ok(())
}

async fn run_lora_apply(
    ctx: &mut ExperimentalStepCtx<'_>,
    step: &ExpertRouteStep,
    snap: &mut ExpertExecSnapshot,
) -> Result<StepOutcome, ProcessMessageError> {
    let plugin_id = step
        .params
        .as_ref()
        .and_then(|p| p.0.get("plugin_id").and_then(|v| v.as_str()))
        .unwrap_or("")
        .trim()
        .to_string();
    if plugin_id.is_empty() {
        return Ok(StepOutcome::Failed(
            "slot.lora.apply 需要 params.plugin_id".into(),
        ));
    }
    let effective_registry = ctx
        .state
        .effective_slot_registry_for_session(ctx.role, ctx.srid)
        .ok_or_else(|| {
            ProcessMessageError::dual_core_invalid(
                "slot.lora.apply requires an effective slot_registry",
            )
        })?;
    let selection = match resolve_lora_llm_selection(&effective_registry, &plugin_id) {
        Ok(selection) => selection,
        Err(message) => return Ok(StepOutcome::Failed(message)),
    };
    if !ctx
        .state
        .directory_plugins
        .manifest_provides_capability(&selection.plugin_id, "llm")
    {
        return Ok(StepOutcome::Failed(format!(
            "LoRA plugin `{}` must declare provides=[\"llm\"]",
            selection.plugin_id
        )));
    }
    if let Err(message) = ctx
        .state
        .directory_plugins
        .ensure_rpc_url(&selection.plugin_id)
    {
        return Ok(StepOutcome::Failed(format!(
            "LoRA plugin `{}` unavailable: {message}",
            selection.plugin_id
        )));
    }
    if snap.lora_plugin_id.is_none() {
        snap.lora_plugin_id = ctx.state.session_cache.expert_lora_plugin_id(ctx.srid);
    }
    ctx.state
        .session_cache
        .set_expert_lora_plugin(ctx.srid, Some(plugin_id.clone()));
    tracing::info!(
        target: "oclive_expert",
        session_ns = %ctx.srid,
        plugin_id = %selection.plugin_id,
        slot_key = %selection.slot_key,
        "专家 LoRA 标记已应用（directory 插件）"
    );
    Ok(StepOutcome::Continue)
}

fn parse_trait_delta(
    params: Option<&serde_json::Value>,
) -> Result<(String, f64), ProcessMessageError> {
    let Some(p) = params else {
        return Err(ProcessMessageError::dual_core_invalid(
            "slot.personality.adjust 需要 params.trait 与 params.delta",
        ));
    };
    let trait_name = p
        .get("trait")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let delta = p.get("delta").and_then(|v| v.as_f64()).unwrap_or(0.0);
    if trait_name.is_empty() {
        return Err(ProcessMessageError::dual_core_invalid(
            "slot.personality.adjust: params.trait 不能为空",
        ));
    }
    Ok((trait_name, delta))
}

fn apply_trait_delta(p: &mut PersonalityVector, trait_name: &str, delta: f64) {
    match trait_name.to_ascii_lowercase().as_str() {
        "warmth" => p.warmth += delta,
        "stubbornness" => p.stubbornness += delta,
        "clinginess" => p.clinginess += delta,
        "sensitivity" => p.sensitivity += delta,
        "assertiveness" => p.assertiveness += delta,
        "forgiveness" => p.forgiveness += delta,
        "talkativeness" => p.talkativeness += delta,
        _ => {}
    }
}

async fn apply_expert_fallback(
    ctx: &mut ExperimentalStepCtx<'_>,
    fallback: ExpertFallback,
    route_id: &str,
    reason: &str,
) -> Result<StepOutcome, ProcessMessageError> {
    match fallback {
        ExpertFallback::Skip => Ok(StepOutcome::Continue),
        ExpertFallback::RetryWithDefault => {
            let key = ctx
                .role
                .slot_registry
                .as_ref()
                .and_then(|r| {
                    r.iter()
                        .find(|(_, e)| e.slot_type.trim() == "llm")
                        .map(|(k, _)| k.clone())
                })
                .unwrap_or_else(|| "llm".into());
            tracing::warn!(
                target: "oclive_expert",
                session_ns = %ctx.srid,
                route_id = %route_id,
                llm_key = %key,
                reason = %reason,
                fallback_hint = %format!("route={route_id}; reason={reason}"),
                "专家流程降级：默认 LLM generate"
            );
            match ctx.run_method(key.as_str(), "generate").await? {
                StepOutcome::NeedsStableCompletion | StepOutcome::Continue => {
                    Ok(StepOutcome::NeedsStableCompletion)
                }
                other => Ok(other),
            }
        }
    }
}

#[cfg(test)]
mod lora_tests {
    use super::*;

    fn entry(slot_type: &str, backend: &str, plugin: Option<&str>) -> SlotRegistryEntry {
        SlotRegistryEntry {
            slot_type: slot_type.into(),
            label: "test".into(),
            backend: backend.into(),
            position: 0,
            plugin: plugin.map(str::to_string),
            plugins: None,
            model: None,
            url: None,
            local_memory_provider_id: None,
            zone: (backend == "directory").then(|| serde_json::json!("experimental")),
            policy: None,
        }
    }

    #[test]
    fn lora_selection_requires_one_predeclared_directory_llm_slot() {
        let mut registry = BTreeMap::new();
        registry.insert(
            "lora_mumu".into(),
            entry("llm", "directory", Some("com.example.mumu-lora")),
        );

        let selected =
            resolve_lora_llm_selection(&registry, "com.example.mumu-lora").expect("selection");
        assert_eq!(selected.slot_key, "lora_mumu");
        assert_eq!(selected.plugin_id, "com.example.mumu-lora");
    }

    #[test]
    fn lora_selection_rejects_builtin_or_ambiguous_slots() {
        let mut registry = BTreeMap::new();
        registry.insert(
            "not_directory".into(),
            entry("llm", "ollama", Some("com.example.mumu-lora")),
        );
        assert!(resolve_lora_llm_selection(&registry, "com.example.mumu-lora").is_err());

        let mut dual_zone = entry("llm", "directory", Some("com.example.mumu-lora"));
        dual_zone.zone = Some(serde_json::json!(["stable", "experimental"]));
        registry.insert("not_isolated".into(), dual_zone);
        assert!(resolve_lora_llm_selection(&registry, "com.example.mumu-lora").is_err());

        registry.remove("not_isolated");
        registry.insert(
            "lora_a".into(),
            entry("llm", "directory", Some("com.example.mumu-lora")),
        );
        registry.insert(
            "lora_b".into(),
            entry("llm", "directory", Some("com.example.mumu-lora")),
        );
        assert!(resolve_lora_llm_selection(&registry, "com.example.mumu-lora").is_err());
    }
}
