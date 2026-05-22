//! 双核运行时：实验核 `pipeline.experimental` + 稳定核 `co_present` 降级。
#![allow(clippy::missing_errors_doc)]

use std::collections::{HashMap, HashSet};

use crate::domain::chat_engine::co_present;
use crate::domain::chat_engine::message_error::ProcessMessageError;
use crate::domain::dual_pipeline_steps::{ExperimentalStepCtx, StepOutcome};
use crate::error::AppError;
use crate::models::dto::SendMessageRequest;
use crate::models::dto::SendMessageResponse;
use crate::models::Role;
use crate::state::AppState;
use oclive_validation::{parse_pipeline_action, PipelineStep};

/// 实验核失败（触发静默降级）。
#[derive(Debug, thiserror::Error)]
#[error("dual-core experimental: {0}")]
pub struct DualCoreError(pub String);

/// 回合开始前可回滚的会话内存态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnRollbackSnapshot {
    pub narrative_hint: Option<String>,
    /// `get_current_emotion` 快照；`None` 表示当时无记录。
    pub emotion_state: Option<String>,
    /// `get_user_presence_scene` 快照（用户叙事/发消息上下文场景）。
    pub active_scene_id: Option<String>,
}

pub struct DualPipelineRunner;

impl DualPipelineRunner {
    pub async fn take_snapshot(state: &AppState, srid: &str) -> TurnRollbackSnapshot {
        let hint = state.stored_complex_emotion_narrative_hint(srid);
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

    pub async fn rollback(state: &AppState, srid: &str, snapshot: TurnRollbackSnapshot) {
        state.set_stored_complex_emotion_narrative_hint(
            srid,
            snapshot.narrative_hint.unwrap_or_default(),
        );
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

    #[allow(clippy::too_many_arguments)]
    pub async fn run_stable(
        state: &AppState,
        req: &SendMessageRequest,
        role: &Role,
        scene_id: String,
        scenes: Vec<String>,
        immersive: bool,
        t0: std::time::Instant,
        mrid: &str,
        srid: &str,
        preflight_ms: u64,
    ) -> Result<SendMessageResponse, ProcessMessageError> {
        co_present::process_co_present(
            state,
            req,
            role,
            scene_id,
            scenes,
            immersive,
            t0,
            mrid,
            srid,
            preflight_ms,
        )
        .await
        .map_err(ProcessMessageError::from)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn run_experimental(
        state: &AppState,
        req: &SendMessageRequest,
        role: &Role,
        scene_id: &str,
        scenes: &[String],
        immersive: bool,
        t0: std::time::Instant,
        mrid: &str,
        srid: &str,
        preflight_ms: u64,
    ) -> Result<SendMessageResponse, ProcessMessageError> {
        let steps = role
            .pipeline_experimental
            .as_ref()
            .ok_or_else(|| experimental_err("missing pipeline.experimental"))?;
        if steps.is_empty() {
            return Err(experimental_err("empty pipeline.experimental"));
        }
        let ordered = topological_sort(steps).map_err(|e| ProcessMessageError::Stage {
            stage: "dual_core_experimental",
            source: AppError::InvalidParameter(e.0),
        })?;

        tracing::info!(
            target: "oclive_dual_core",
            session_ns = %srid,
            step_count = ordered.len(),
            "开始执行实验核"
        );

        let mut ctx =
            ExperimentalStepCtx::new(state, role, req, scene_id.to_string(), mrid, srid).await?;
        let mut wants_stable_completion = false;

        for (idx, step) in ordered.iter().enumerate() {
            let step_no = idx + 1;
            let (registry_key, method) = parse_pipeline_action(step.action.as_str()).map_err(|e| {
                tracing::warn!(
                    target: "oclive_dual_core",
                    session_ns = %srid,
                    step = step_no,
                    action = %step.action,
                    error = %e,
                    "实验核在第 {step_no} 步失败: {e}，正在降级到稳定核"
                );
                ProcessMessageError::Stage {
                    stage: "dual_core_experimental",
                    source: AppError::InvalidParameter(e),
                }
            })?;
            match ctx.run_method(&registry_key, method.as_str()).await {
                Ok(StepOutcome::Continue) => {}
                Ok(StepOutcome::NeedsStableCompletion) => wants_stable_completion = true,
                Ok(StepOutcome::AgentComplete(resp)) => {
                    tracing::info!(
                        target: "oclive_dual_core",
                        session_ns = %srid,
                        "实验核执行成功"
                    );
                    return Ok(resp);
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
                    return Err(experimental_process_err(msg));
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
            return Err(experimental_process_err(msg));
        }

        tracing::info!(
            target: "oclive_dual_core",
            session_ns = %srid,
            "实验核执行成功"
        );

        Self::run_stable(
            state,
            req,
            role,
            scene_id.to_string(),
            scenes.to_vec(),
            immersive,
            t0,
            mrid,
            srid,
            preflight_ms,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn run_with_fallback(
        state: &AppState,
        req: &SendMessageRequest,
        role: &Role,
        scene_id: String,
        scenes: Vec<String>,
        immersive: bool,
        t0: std::time::Instant,
        mrid: &str,
        srid: &str,
        preflight_ms: u64,
    ) -> Result<SendMessageResponse, ProcessMessageError> {
        let snapshot = Self::take_snapshot(state, srid).await;
        match Self::run_experimental(
            state,
            req,
            role,
            scene_id.as_str(),
            &scenes,
            immersive,
            t0,
            mrid,
            srid,
            preflight_ms,
        )
        .await
        {
            Ok(resp) => Ok(resp),
            Err(e) => {
                Self::rollback(state, srid, snapshot).await;
                let resp = Self::run_stable(
                    state,
                    req,
                    role,
                    scene_id,
                    scenes,
                    immersive,
                    t0,
                    mrid,
                    srid,
                    preflight_ms,
                )
                .await?;
                tracing::info!(
                    target: "oclive_dual_core",
                    session_ns = %srid,
                    degraded_from = "experimental",
                    prior_error = %e,
                    "稳定核执行完成（降级模式）"
                );
                Ok(resp)
            }
        }
    }
}

fn experimental_err(msg: impl Into<String>) -> ProcessMessageError {
    ProcessMessageError::Stage {
        stage: "dual_core_experimental",
        source: AppError::InvalidParameter(msg.into()),
    }
}

fn experimental_process_err(msg: impl Into<String>) -> ProcessMessageError {
    ProcessMessageError::Stage {
        stage: "dual_core_experimental",
        source: AppError::InvalidParameter(msg.into()),
    }
}

/// 按 `depends_on` 拓扑排序；环或未知依赖返回错误。
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
            Arc::new(crate::infrastructure::llm::MockLlmClient {
                reply: "ok".into(),
            }),
            tmp.path().to_path_buf(),
        )
        .await
        .unwrap();
        let srid = "role:demo:default";
        state
            .db_manager
            .ensure_role_runtime(srid)
            .await
            .unwrap();
        state.set_stored_complex_emotion_narrative_hint(srid, "hint-a".into());
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

        state.set_stored_complex_emotion_narrative_hint(srid, "hint-b".into());
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
            state.stored_complex_emotion_narrative_hint(srid),
            "hint-a"
        );
        assert_eq!(
            state.db_manager.get_current_emotion(srid).await.unwrap(),
            Some("happy".to_string())
        );
        assert_eq!(
            state.db_manager.get_user_presence_scene(srid).await.unwrap(),
            Some("park".to_string())
        );
    }
}
