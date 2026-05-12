//! 蓝图解释器：按 `PipelineBlueprint.steps` 顺序调用 `pipeline_actions` 原子；支持 `branch` 递归子列表。

use super::pipeline_actions;
use super::pipeline_loader::{OnFailurePolicy, PipelineBlueprint, PipelineStepSpec};
use super::turn_context::TurnContext;
use crate::error::{AppError, Result};
use crate::models::dto::SendMessageRequest;
use crate::state::KernelAppState;
use std::future::Future;
use std::pin::Pin;
use std::time::Instant;

type StepsFut<'a> = Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;

/// 与 `process_message` 历史默认入口序列一致（`validate_scene` 已在蓝图外执行）。
pub async fn run_default_entry_pipeline(
    state: &KernelAppState,
    ctx: &mut TurnContext,
    req: &SendMessageRequest,
) -> Result<()> {
    pipeline_actions::init_turn(state, ctx, req).await?;
    pipeline_actions::ensure_role_runtime(state, ctx, req).await?;
    pipeline_actions::load_role(state, ctx, req).await?;
    pipeline_actions::seed_interaction_mode(state, ctx, req).await?;
    pipeline_actions::log_effective_plugin_backends(state, ctx, req).await?;
    pipeline_actions::resolve_plugins(state, ctx, req).await?;
    pipeline_actions::resolve_main_llm_model(state, ctx, req).await?;
    pipeline_actions::run_agent(state, ctx, req).await?;
    Ok(())
}

/// 顺序执行蓝图中的每一步；`onFailure` 为 `DEGRADE` 时在单步失败后继续。
pub async fn execute_pipeline(
    state: &KernelAppState,
    ctx: &mut TurnContext,
    req: &SendMessageRequest,
    blueprint: &PipelineBlueprint,
) -> Result<()> {
    run_steps(state, ctx, req, &blueprint.steps, blueprint.on_failure).await
}

fn run_steps<'a>(
    state: &'a KernelAppState,
    ctx: &'a mut TurnContext,
    req: &'a SendMessageRequest,
    steps: &'a [PipelineStepSpec],
    on_failure: OnFailurePolicy,
) -> StepsFut<'a> {
    Box::pin(async move {
        for (idx, step) in steps.iter().enumerate() {
            if let Some(b) = &step.branch {
                let take_true = b.predicate.eval(ctx);
                tracing::debug!(
                    target: "oclive_pipeline",
                    step_index = idx,
                    predicate = ?b.predicate,
                    branch = if take_true { "onTrue" } else { "onFalse" },
                    "pipeline branch"
                );
                let arm = if take_true {
                    b.on_true.as_slice()
                } else {
                    b.on_false.as_slice()
                };
                run_steps(state, ctx, req, arm, on_failure).await?;
                continue;
            }
            if step.parallel.is_some() {
                return Err(AppError::InvalidParameter(
                    "parallel steps are not supported by this interpreter build".into(),
                ));
            }
            let action = step.action.as_deref().ok_or_else(|| {
                AppError::InvalidParameter("pipeline step missing action".into())
            })?;
            let t0 = Instant::now();
            let res = dispatch_action(state, ctx, req, action).await;
            let elapsed_ms = t0.elapsed().as_millis() as u64;
            let step_id = step.id.as_deref().unwrap_or("-");
            match &res {
                Ok(()) => tracing::debug!(
                    target: "oclive_pipeline",
                    step_index = idx,
                    step_id,
                    action = %action,
                    elapsed_ms,
                    ok = true,
                    "pipeline step"
                ),
                Err(e) => tracing::warn!(
                    target: "oclive_pipeline",
                    step_index = idx,
                    step_id,
                    action = %action,
                    elapsed_ms,
                    ok = false,
                    error = %e,
                    "pipeline step"
                ),
            }
            if let Err(e) = res {
                match on_failure {
                    OnFailurePolicy::Halt => return Err(e),
                    OnFailurePolicy::Degrade => {
                        tracing::warn!(
                            target: "oclive_pipeline",
                            step_index = idx,
                            step_id,
                            action = %action,
                            "pipeline step failed; onFailure=DEGRADE, continuing"
                        );
                    }
                }
            }
        }
        Ok(())
    })
}

async fn dispatch_action(
    state: &KernelAppState,
    ctx: &mut TurnContext,
    req: &SendMessageRequest,
    action: &str,
) -> Result<()> {
    match action {
        "init_turn" => pipeline_actions::init_turn(state, ctx, req).await,
        "ensure_role_runtime" => pipeline_actions::ensure_role_runtime(state, ctx, req).await,
        "load_role" => pipeline_actions::load_role(state, ctx, req).await,
        "seed_interaction_mode" => pipeline_actions::seed_interaction_mode(state, ctx, req).await,
        "log_effective_plugin_backends" => {
            pipeline_actions::log_effective_plugin_backends(state, ctx, req).await
        }
        "resolve_plugins" => pipeline_actions::resolve_plugins(state, ctx, req).await,
        "resolve_main_llm_model" => {
            pipeline_actions::resolve_main_llm_model(state, ctx, req).await
        }
        "run_agent" => pipeline_actions::run_agent(state, ctx, req).await,
        "set_user_presence_scene" => {
            pipeline_actions::set_user_presence_scene(state, ctx, req).await
        }
        "load_presence_routing" => {
            pipeline_actions::load_presence_routing(state, ctx, req).await
        }
        "analyze_emotion_user" => {
            pipeline_actions::analyze_emotion_user(state, ctx, req).await
        }
        _ => Err(AppError::InvalidParameter(format!(
            "unknown pipeline action (interpreter): {action}"
        ))),
    }
}
