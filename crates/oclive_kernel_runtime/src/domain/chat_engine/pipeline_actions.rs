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
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::time::Instant;

/// 蓝图 `PARALLEL` 调度用的 I/O 语义标注（非文件系统只读；**WRITE** 表示会写库 / 改会话 / 调 LLM 等）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionIOType {
    ReadOnly,
    Write,
}

/// 各原子 `action` 的 I/O 类型（与 `ALLOWED_PIPELINE_BLUEPRINT_ACTIONS` 对齐；扩展原子时须同步）。
pub static ACTION_IO_TYPES: Lazy<HashMap<&'static str, ActionIOType>> = Lazy::new(|| {
    HashMap::from([
        ("init_turn", ActionIOType::Write),
        ("ensure_role_runtime", ActionIOType::Write),
        ("load_role", ActionIOType::Write),
        ("seed_interaction_mode", ActionIOType::Write),
        ("log_effective_plugin_backends", ActionIOType::ReadOnly),
        ("resolve_plugins", ActionIOType::Write),
        ("resolve_main_llm_model", ActionIOType::Write),
        ("run_agent", ActionIOType::Write),
        ("set_user_presence_scene", ActionIOType::Write),
        ("load_presence_routing", ActionIOType::Write),
        ("analyze_emotion_user", ActionIOType::Write),
        ("memory_retrieve_short_term", ActionIOType::ReadOnly),
        ("memory_retrieve_long_term", ActionIOType::ReadOnly),
        ("assemble_prompt", ActionIOType::ReadOnly),
        ("generate_response", ActionIOType::Write),
        ("expert_empathy_touch", ActionIOType::Write),
    ])
});

pub fn action_io_type(action: &str) -> Option<ActionIOType> {
    ACTION_IO_TYPES.get(action).copied()
}

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

/// 占位：短期记忆检索（**READ_ONLY**；示例与 `PARALLEL` 烟测用，无实际 DB 访问）。
pub async fn memory_retrieve_short_term(
    _state: &KernelAppState,
    _ctx: &mut TurnContext,
    _req: &SendMessageRequest,
) -> Result<()> {
    tracing::debug!(target: "oclive_pipeline", action = "memory_retrieve_short_term", "noop read-only");
    Ok(())
}

/// 占位：长期记忆检索（**READ_ONLY**；示例与 `PARALLEL` 烟测用）。
pub async fn memory_retrieve_long_term(
    _state: &KernelAppState,
    _ctx: &mut TurnContext,
    _req: &SendMessageRequest,
) -> Result<()> {
    tracing::debug!(target: "oclive_pipeline", action = "memory_retrieve_long_term", "noop read-only");
    Ok(())
}

/// 占位：Prompt 组装（**READ_ONLY**；真实组装仍在共景 / 远程路径内，蓝图层仅占位语义）。
pub async fn assemble_prompt(
    _state: &KernelAppState,
    _ctx: &mut TurnContext,
    _req: &SendMessageRequest,
) -> Result<()> {
    tracing::debug!(target: "oclive_pipeline", action = "assemble_prompt", "noop read-only");
    Ok(())
}

/// 占位：高共情路径上「专家模型 / 共情触发器」钩子（**WRITE**；当前仅审计日志，后续可接专家图）。
pub async fn expert_empathy_touch(
    _state: &KernelAppState,
    _ctx: &mut TurnContext,
    _req: &SendMessageRequest,
) -> Result<()> {
    tracing::info!(
        target: "oclive_pipeline",
        action = "expert_empathy_touch",
        "expert empathy trigger (placeholder)"
    );
    Ok(())
}

/// 与 `run_agent` 等价（**WRITE**），供蓝图以 `generate_response` 命名展示「生成」步骤。
pub async fn generate_response(
    state: &KernelAppState,
    ctx: &mut TurnContext,
    req: &SendMessageRequest,
) -> Result<()> {
    run_agent(state, ctx, req).await
}
