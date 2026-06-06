//! # Dual-core runtime scheduling (experimental core + stable core)
#![cfg(feature = "dual_core")]
//!
//! **Role**: when a role is loaded and [`Role::dual_core_gated`](crate::models::Role::dual_core_gated), run experimental steps from blueprint
//! `pipeline.experimental` first; on failure **gracefully degrade** to [`turn_pipeline::execute_turn`](crate::domain::chat_engine::turn_pipeline::execute_turn) ([`TurnMode::CoPresent`](crate::domain::chat_engine::turn_pipeline::TurnMode::CoPresent))
//! (stable core), without user-visible disruption.
//!
//! **Design**: experiment first, rollback-capable, degradable—experimental steps only mutate snapshot-able session in-memory state; on failure restore snapshot then take the stable path.
//!
//! **Feature freeze (2026-06)**: dual-core scheduling is compiled only with the `dual_core` Cargo feature.
//! Production stable pipeline execution for gated roles runs on **`oclivenewnew-tauri --features dual_core`**.
//! `oclive-kernel-server` and default host builds keep `dual_core` off—experimental steps are not scheduled there.
//! Allow-list SSOT: [`dual_pipeline_registry::EXPERIMENTAL_METHOD_SPECS`](super::dual_pipeline_registry::EXPERIMENTAL_METHOD_SPECS);
//! `oclive-cli explain DUAL_CORE` keeps a separate table—sync both when changing methods (see registry module docs).
//!
//! **Disambiguation**: `dual_pipeline` is the **runtime orchestrator** (this module). The blueprint JSON keys
//! `pipeline.experimental` / `pipeline.stable` are **config only** — not the on-disk blueprint file
//! `pipeline.ocblueprint`, and not a step-scheduling DSL.
//!
//! **Downstream**: [`process_message`](crate::domain::chat_engine::process_message),
//! [`ExperimentalStepCtx`](super::dual_pipeline_steps::ExperimentalStepCtx),
//! [`dual_pipeline_registry`](super::dual_pipeline_registry).
//!
//! **Key decision**: does not execute `pipeline.stable`; stable core is always hard-coded `co_present`.

use std::collections::{HashMap, HashSet};

use crate::domain::chat_engine::message_error::ProcessMessageError;
use crate::domain::chat_engine::turn_context::TurnContext;
use crate::domain::chat_engine::turn_pipeline::{execute_turn, TurnMode};
use crate::domain::dual_pipeline_steps::{ExperimentalStepCtx, StepOutcome};
use crate::models::dto::SendMessageResponse;
use crate::state::AppState;
use oclive_validation::{parse_pipeline_action_kind, PipelineActionKind, PipelineStep};

/// Experimental core failure (triggers graceful degradation).
#[derive(Debug, thiserror::Error)]
#[error("dual-core experimental: {0}")]
pub(crate) struct DualCoreError(pub String);

/// Session in-memory state captured before experimental core runs and restored on failure.
///
/// Only fields experimental steps may mutate and the stable core reuses (controls rollback cost and consistency):
/// - `narrative_hint`: complex emotion narrative cache;
/// - `emotion_state`: current emotion label in DB;
/// - `active_scene_id`: user narrative scene (`user_presence_scene`, may differ from role `current_scene`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnRollbackSnapshot {
    pub narrative_hint: Option<String>,
    pub emotion_state: Option<String>,
    pub active_scene_id: Option<String>,
}

pub struct DualPipelineRunner;

impl DualPipelineRunner {
    /// Call before any experimental step; used by [`rollback`](Self::rollback) to restore state.
    pub async fn take_snapshot(state: &AppState, srid: &str) -> TurnRollbackSnapshot {
        let hint = state
            .session_cache
            .stored_complex_emotion_narrative_hint(srid);
        let emotion_state = state
            .db_manager
            .get_current_emotion(srid)
            .await
            .ok()
            .flatten();
        let active_scene_id = state
            .db_manager
            .get_user_presence_scene(srid)
            .await
            .ok()
            .flatten();
        TurnRollbackSnapshot {
            narrative_hint: if hint.is_empty() { None } else { Some(hint) },
            emotion_state,
            active_scene_id,
        }
    }

    /// On experimental failure before degradation: restore the three session fields from [`take_snapshot`].
    pub async fn rollback(state: &AppState, srid: &str, snapshot: TurnRollbackSnapshot) {
        crate::domain::complex_emotion_store::persist_stored_narrative_hint(
            state,
            srid,
            snapshot.narrative_hint.unwrap_or_default(),
        )
        .await;
        if let Some(emotion) = snapshot.emotion_state {
            let _ = state
                .db_manager
                .set_current_emotion(srid, emotion.as_str())
                .await;
        }
        if let Some(scene) = snapshot.active_scene_id {
            let _ = state
                .db_manager
                .set_user_presence_scene(srid, scene.as_str())
                .await;
        }
    }

    /// # Errors
    ///
    /// Propagates [`ProcessMessageError`] from [`execute_turn`].
    pub async fn run_stable(
        ctx: &TurnContext<'_>,
    ) -> Result<SendMessageResponse, ProcessMessageError> {
        execute_turn(ctx, TurnMode::CoPresent)
            .await
            .map_err(ProcessMessageError::from)
    }

    /// # Errors
    ///
    /// Returns on blueprint DAG / action parse / experimental step / final `co_present` failure; caught by [`run_with_fallback`] for degradation.
    pub async fn run_experimental(
        turn: &TurnContext<'_>,
    ) -> Result<SendMessageResponse, ProcessMessageError> {
        let role = turn.role;
        let srid = turn.srid;
        let steps = role.pipeline_experimental.as_ref().ok_or_else(|| {
            ProcessMessageError::dual_core_invalid("missing pipeline.experimental")
        })?;
        if steps.is_empty() {
            return Err(ProcessMessageError::dual_core_invalid(
                "empty pipeline.experimental",
            ));
        }
        let ordered =
            topological_sort(steps).map_err(|e| ProcessMessageError::dual_core_invalid(e.0))?;

        tracing::info!(
            target: "oclive_dual_core",
            session_ns = %srid,
            step_count = ordered.len(),
            "开始执行实验核"
        );

        let mut step_ctx = ExperimentalStepCtx::new(
            turn.state,
            role,
            turn.req,
            turn.scene_id.to_string(),
            turn.mrid,
            srid,
        )
        .await?;
        let mut wants_stable_completion = false;

        for (idx, step) in ordered.iter().enumerate() {
            let step_no = idx + 1;
            let outcome = match parse_pipeline_action_kind(step.action.as_str()) {
                Ok(PipelineActionKind::ExpertInvoke) => step_ctx.run_expert_invoke().await,
                Ok(PipelineActionKind::Slot {
                    registry_key,
                    method,
                }) => step_ctx.run_method(&registry_key, method.as_str()).await,
                Err(e) => {
                    tracing::warn!(
                        target: "oclive_dual_core",
                        session_ns = %srid,
                        step = step_no,
                        action = %step.action,
                        error = %e,
                        "实验核在第 {step_no} 步失败: {e}，正在降级到稳定核"
                    );
                    return Err(ProcessMessageError::dual_core_invalid(e));
                }
            };
            match outcome {
                Ok(StepOutcome::Continue) => {}
                Ok(StepOutcome::NeedsStableCompletion) => wants_stable_completion = true,
                Ok(StepOutcome::AgentComplete(resp)) => {
                    tracing::info!(
                        target: "oclive_dual_core",
                        session_ns = %srid,
                        "实验核执行成功"
                    );
                    return Ok(*resp);
                }
                Ok(StepOutcome::Failed(msg)) => {
                    tracing::warn!(
                        target: "oclive_dual_core",
                        session_ns = %srid,
                        step = step_no,
                        action = %step.action,
                        error = %msg,
                        "实验核在第 {step_no} 步失败: {msg}，正在降级到稳定核"
                    );
                    return Err(ProcessMessageError::dual_core_invalid(msg));
                }
                Err(e) => {
                    tracing::warn!(
                        target: "oclive_dual_core",
                        session_ns = %srid,
                        step = step_no,
                        action = %step.action,
                        error = %e,
                        "实验核在第 {step_no} 步失败: {e}，正在降级到稳定核"
                    );
                    return Err(e);
                }
            }
        }

        if !wants_stable_completion {
            let msg = "experimental pipeline 须包含至少一步 slot.<llm_key>.generate 或 agent.process 短路";
            tracing::warn!(
                target: "oclive_dual_core",
                session_ns = %srid,
                error = %msg,
                "实验核校验失败: {msg}，正在降级到稳定核"
            );
            return Err(ProcessMessageError::dual_core_invalid(msg));
        }

        tracing::info!(
            target: "oclive_dual_core",
            session_ns = %srid,
            "实验核执行成功"
        );

        Self::run_stable(turn).await
    }

    /// # Errors
    ///
    /// After experimental failure with snapshot rollback; returns error to caller only if stable core `co_present` also fails.
    pub async fn run_with_fallback(
        turn: &TurnContext<'_>,
    ) -> Result<SendMessageResponse, ProcessMessageError> {
        let snapshot = Self::take_snapshot(turn.state, turn.srid).await;
        match Self::run_experimental(turn).await {
            Ok(resp) => Ok(resp),
            Err(e) => {
                Self::rollback(turn.state, turn.srid, snapshot).await;
                let resp = Self::run_stable(turn).await?;
                tracing::info!(
                    target: "oclive_dual_core",
                    session_ns = %turn.srid,
                    degraded_from = "experimental",
                    prior_error = %e,
                    "稳定核执行完成（降级模式）"
                );
                Ok(resp)
            }
        }
    }
}

/// Topological sort by `depends_on`; cycle or unknown dependency returns error (reused by expert sub-flow).
pub(crate) fn topological_sort_pipeline_steps(
    steps: &[PipelineStep],
) -> Result<Vec<&PipelineStep>, String> {
    topological_sort(steps).map_err(|e| e.0)
}

/// Topological sort by `depends_on`; cycle or unknown dependency returns error.
fn topological_sort(steps: &[PipelineStep]) -> Result<Vec<&PipelineStep>, DualCoreError> {
    let actions: HashSet<&str> = steps.iter().map(|s| s.action.as_str()).collect();
    let mut indegree: HashMap<&str, usize> = HashMap::new();
    let mut edges: HashMap<&str, Vec<&str>> = HashMap::new();

    for step in steps {
        indegree.entry(step.action.as_str()).or_insert(0);
        for dep in &step.depends_on {
            if !actions.contains(dep.as_str()) {
                return Err(DualCoreError(format!(
                    "depends_on「{dep}」未在同一 experimental pipeline 中声明"
                )));
            }
            edges
                .entry(dep.as_str())
                .or_default()
                .push(step.action.as_str());
            *indegree.entry(step.action.as_str()).or_insert(0) += 1;
        }
    }

    let mut queue: Vec<&str> = indegree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(a, _)| *a)
        .collect();
    queue.sort_unstable();

    let mut out = Vec::with_capacity(steps.len());
    let step_by_action: HashMap<&str, &PipelineStep> =
        steps.iter().map(|s| (s.action.as_str(), s)).collect();

    while let Some(action) = queue.first().copied() {
        queue.remove(0);
        if let Some(step) = step_by_action.get(action) {
            out.push(*step);
        }
        if let Some(nexts) = edges.get(action) {
            for next in nexts {
                if let Some(d) = indegree.get_mut(next) {
                    *d = d.saturating_sub(1);
                    if *d == 0 {
                        queue.push(next);
                    }
                }
            }
        }
    }

    if out.len() != steps.len() {
        return Err(DualCoreError("depends_on 存在环".into()));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use std::sync::Arc;

    #[test]
    fn topological_sort_respects_deps() {
        let steps = vec![
            PipelineStep {
                action: "slot.a.analyze".into(),
                depends_on: vec![],
            },
            PipelineStep {
                action: "slot.b.generate".into(),
                depends_on: vec!["slot.a.analyze".into()],
            },
        ];
        let sorted = topological_sort(&steps).unwrap();
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].action, "slot.a.analyze");
        assert_eq!(sorted[1].action, "slot.b.generate");
    }

    #[tokio::test]
    async fn rollback_restores_narrative_hint_emotion_and_scene() {
        let tmp = tempfile::tempdir().unwrap();
        let state = AppState::new_in_memory_with_llm(
            Arc::new(crate::infrastructure::llm::MockLlmClient { reply: "ok".into() }),
            tmp.path().to_path_buf(),
        )
        .await
        .unwrap();
        let srid = "role:demo:default";
        state.db_manager.ensure_role_runtime(srid).await.unwrap();
        state
            .session_cache
            .set_stored_complex_emotion_narrative_hint(srid, "hint-a".into());
        state
            .db_manager
            .set_current_emotion(srid, "happy")
            .await
            .unwrap();
        state
            .db_manager
            .set_user_presence_scene(srid, "park")
            .await
            .unwrap();

        let snap = DualPipelineRunner::take_snapshot(&state, srid).await;

        state
            .session_cache
            .set_stored_complex_emotion_narrative_hint(srid, "hint-b".into());
        state
            .db_manager
            .set_current_emotion(srid, "sad")
            .await
            .unwrap();
        state
            .db_manager
            .set_user_presence_scene(srid, "home")
            .await
            .unwrap();

        DualPipelineRunner::rollback(&state, srid, snap).await;

        assert_eq!(
            state
                .session_cache
                .stored_complex_emotion_narrative_hint(srid),
            "hint-a"
        );
        assert_eq!(
            state.db_manager.get_current_emotion(srid).await.unwrap(),
            Some("happy".to_string())
        );
        assert_eq!(
            state
                .db_manager
                .get_user_presence_scene(srid)
                .await
                .unwrap(),
            Some("park".to_string())
        );
    }
}
