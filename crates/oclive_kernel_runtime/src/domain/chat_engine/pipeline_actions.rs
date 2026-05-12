//! v0 原子操作：从 `process_message` 抽出，行为须与重构前一致（bit-exact）。
//!
//! 统一签名：`async fn(state, ctx, req) -> Result<()>`；同步逻辑亦包在 `async fn` 内以满足调用约定。

use super::context::validate_scene_id;
use super::presence::user_is_remote_from_character;
use super::turn_context::TurnContext;
use super::resolve_main_llm_model_for_generate;
use crate::domain::agent::AgentInput;
use crate::error::{AppError, Result};
use crate::models::dto::SendMessageRequest;
use crate::state::KernelAppState;
use std::sync::atomic::Ordering;
use std::time::Instant;

fn require_manifest_role_id<'a>(ctx: &'a TurnContext) -> Result<&'a str> {
    ctx.ids
        .manifest_role_id
        .as_deref()
        .ok_or_else(|| AppError::InvalidParameter("TurnContext.ids.manifest_role_id".into()))
}

fn require_session_namespace<'a>(ctx: &'a TurnContext) -> Result<&'a str> {
    ctx.ids
        .session_namespace
        .as_deref()
        .ok_or_else(|| AppError::InvalidParameter("TurnContext.ids.session_namespace".into()))
}

fn require_requested_scene_id(ctx: &TurnContext) -> Result<String> {
    ctx.request
        .requested_scene_id
        .clone()
        .ok_or_else(|| AppError::InvalidParameter("TurnContext.request.requested_scene_id".into()))
}

fn require_effective_scene_id(ctx: &TurnContext) -> Result<String> {
    ctx.scene
        .effective_scene_id
        .clone()
        .ok_or_else(|| AppError::InvalidParameter("TurnContext.scene.effective_scene_id".into()))
}

fn require_role(ctx: &TurnContext) -> Result<&std::sync::Arc<crate::models::Role>> {
    ctx.role
        .role
        .as_ref()
        .ok_or_else(|| AppError::InvalidParameter("TurnContext.role.role".into()))
}

/// 重置本轮生成取消标志（与 `process_message` 入口一致）。
pub async fn init_turn(state: &KernelAppState, _ctx: &mut TurnContext, _req: &SendMessageRequest) -> Result<()> {
    state.chat_generation_cancel.store(false, Ordering::Release);
    Ok(())
}

/// `ensure_role_runtime` + 与原先一致的 `tracing::debug`。
pub async fn ensure_role_runtime(state: &KernelAppState, ctx: &mut TurnContext, _req: &SendMessageRequest) -> Result<()> {
    let mrid = require_manifest_role_id(ctx)?;
    let srid = require_session_namespace(ctx)?;
    let io = Instant::now();
    state.db_manager.ensure_role_runtime(srid).await?;
    tracing::debug!(
        target: "oclive_chat_io",
        role_id = %mrid,
        session_ns = %srid,
        op = "ensure_role_runtime",
        elapsed_ms = io.elapsed().as_millis() as u64
    );
    Ok(())
}

/// 加载角色（`ensure_role_loaded`）+ 与原先一致的 `tracing::debug`。
pub async fn load_role(state: &KernelAppState, ctx: &mut TurnContext, _req: &SendMessageRequest) -> Result<()> {
    let mrid = require_manifest_role_id(ctx)?;
    let io = Instant::now();
    let role = state.load_role_cached(mrid)?;
    tracing::debug!(
        target: "oclive_chat_io",
        role_id = %mrid,
        op = "ensure_role_loaded",
        elapsed_ms = io.elapsed().as_millis() as u64
    );
    ctx.role.role = Some(role);
    Ok(())
}

/// `ensure_interaction_mode_seeded`（紧随 `load_role` 之后，与 `process_message` 顺序一致）。
pub async fn seed_interaction_mode(state: &KernelAppState, ctx: &mut TurnContext, _req: &SendMessageRequest) -> Result<()> {
    let srid = require_session_namespace(ctx)?;
    let role = require_role(ctx)?;
    state
        .db_manager
        .ensure_interaction_mode_seeded(srid, role.interaction_mode.as_deref())
        .await?;
    Ok(())
}

/// 与原先一致的 `effective_plugin_backends` 调试日志。
pub async fn log_effective_plugin_backends(
    state: &KernelAppState,
    ctx: &mut TurnContext,
    _req: &SendMessageRequest,
) -> Result<()> {
    let mrid = require_manifest_role_id(ctx)?;
    let srid = require_session_namespace(ctx)?;
    let scene_id = require_effective_scene_id(ctx)?;
    let role = require_role(ctx)?;
    let effective_backends = state.effective_plugin_backends_for_session(role.as_ref(), srid);
    let effective_sources = state.effective_plugin_backend_sources_for_session(srid);
    log::debug!(
        target: "oclive_chat",
        "send_message backends role_id={} scene_id={} session_ns={} {}",
        mrid,
        scene_id,
        srid,
        super::backend_resolution_summary(&effective_backends, &effective_sources)
    );
    Ok(())
}

/// 解析 `ResolvedRolePlugins`（单次回合内复用）。
pub async fn resolve_plugins(state: &KernelAppState, ctx: &mut TurnContext, _req: &SendMessageRequest) -> Result<()> {
    let srid = require_session_namespace(ctx)?;
    let role = require_role(ctx)?;
    ctx.plugins.resolved = Some(state.resolved_plugins_for_session(role.as_ref(), Some(srid)));
    Ok(())
}

/// 解析主 LLM `model` 参数（与 `resolve_main_llm_model_for_generate` 一致）。
pub async fn resolve_main_llm_model(
    state: &KernelAppState,
    ctx: &mut TurnContext,
    _req: &SendMessageRequest,
) -> Result<()> {
    let srid = require_session_namespace(ctx)?;
    let role = require_role(ctx)?;
    let agent_llm_model = resolve_main_llm_model_for_generate(state, role.as_ref(), srid).await?;
    ctx.llm.main_llm_model = Some(agent_llm_model);
    Ok(())
}

/// 场景校验与回退（`validate_scene_id`）。
pub async fn validate_scene(state: &KernelAppState, ctx: &mut TurnContext, _req: &SendMessageRequest) -> Result<()> {
    let mrid = require_manifest_role_id(ctx)?;
    let requested_scene_id = require_requested_scene_id(ctx)?;
    let (scene_id, scenes) = validate_scene_id(state, mrid, requested_scene_id)?;
    ctx.scene.effective_scene_id = Some(scene_id);
    ctx.scene.scene_id_list = Some(scenes);
    Ok(())
}

/// 写入用户 presence 场景（`set_user_presence_scene`）。
pub async fn set_user_presence_scene(state: &KernelAppState, ctx: &mut TurnContext, _req: &SendMessageRequest) -> Result<()> {
    let srid = require_session_namespace(ctx)?;
    let scene_id = require_effective_scene_id(ctx)?;
    state
        .db_manager
        .set_user_presence_scene(srid, scene_id.as_str())
        .await?;
    Ok(())
}

/// 加载异地/沉浸相关路由信息，并写入 `preflight_ms`（与 `process_message` 中 `t0.elapsed()` 语义一致）。
pub async fn load_presence_routing(state: &KernelAppState, ctx: &mut TurnContext, _req: &SendMessageRequest) -> Result<()> {
    let srid = require_session_namespace(ctx)?;
    let scene_id = require_effective_scene_id(ctx)?;
    let current_scene = state.db_manager.get_current_scene(srid).await?;
    let immersive = state
        .db_manager
        .get_interaction_mode(srid)
        .await?
        .is_immersive();
    let remote_life_enabled = state.db_manager.get_remote_life_enabled(srid).await?;
    let is_remote =
        immersive && user_is_remote_from_character(scene_id.as_str(), current_scene.as_deref());
    ctx.presence.current_scene = current_scene;
    ctx.presence.immersive = Some(immersive);
    ctx.presence.remote_life_enabled = Some(remote_life_enabled);
    ctx.presence.is_remote = Some(is_remote);
    if let Some(start) = ctx.trace.started_at {
        ctx.trace.preflight_ms = Some(start.elapsed().as_millis() as u64);
    }
    Ok(())
}

/// 用户句情绪分析（`pl.emotion.analyze`）。
pub async fn analyze_emotion_user(_state: &KernelAppState, ctx: &mut TurnContext, _req: &SendMessageRequest) -> Result<()> {
    let pl = ctx
        .plugins
        .resolved
        .as_ref()
        .ok_or_else(|| AppError::InvalidParameter("TurnContext.plugins.resolved".into()))?;
    let user_message = ctx
        .request
        .user_message
        .as_deref()
        .ok_or_else(|| AppError::InvalidParameter("TurnContext.request.user_message".into()))?;
    let analyzed = pl.emotion.analyze(user_message)?;
    let emotion_result: crate::domain::emotion_analyzer::EmotionResult = analyzed;
    ctx.emotion.user_emotion = Some(emotion_result);
    Ok(())
}

/// 运行 Agent（写入 `ctx.agent.output`）；调用方根据 `handled` 分支。
pub async fn run_agent(_state: &KernelAppState, ctx: &mut TurnContext, req: &SendMessageRequest) -> Result<()> {
    let mrid = require_manifest_role_id(ctx)?;
    let srid = require_session_namespace(ctx)?;
    let pl = ctx
        .plugins
        .resolved
        .as_ref()
        .ok_or_else(|| AppError::InvalidParameter("TurnContext.plugins.resolved".into()))?;
    let agent_llm_model = ctx
        .llm
        .main_llm_model
        .as_deref()
        .ok_or_else(|| AppError::InvalidParameter("TurnContext.llm.main_llm_model".into()))?;
    let agent_out = pl
        .agent
        .process(AgentInput {
            role_id: mrid.to_string(),
            session_namespace: srid.to_string(),
            message: req.user_message.clone(),
            model: agent_llm_model.to_string(),
        })
        .await?;
    ctx.flags.agent_handled = Some(agent_out.handled);
    ctx.agent.output = Some(agent_out);
    Ok(())
}
