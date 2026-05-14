//! 单条用户消息编排入口与分层错误类型。

use crate::domain::agent::AgentInput;
use crate::domain::chat_engine::co_present;
use crate::domain::chat_engine::{
    backend_resolution_summary, conversation_state_role_id, ensure_role_loaded,
    process_remote_life, process_remote_stub,
};
use crate::domain::chat_engine::{context::validate_scene_id, emotion_to_dto};
use crate::domain::chat_engine::presence::user_is_remote_from_character;
use crate::domain::startup_health;
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::error::{AppError, Result};
use crate::models::dto::{
    PresenceMode, SendMessageRequest, SendMessageResponse, API_VERSION, SCHEMA_VERSION,
};
use crate::state::AppState;
use std::time::Instant;
use thiserror::Error;

/// `process_message` 编排失败：按阶段标注，便于日志与排障。
#[derive(Debug, Error)]
pub enum ProcessMessageError {
    #[error("send_message[{stage}]: {source}")]
    Stage {
        stage: &'static str,
        #[source]
        source: AppError,
    },
    #[error(transparent)]
    CoPresent(#[from] co_present::CoPresentError),
}

impl ProcessMessageError {
    fn stage(stage: &'static str, source: AppError) -> Self {
        Self::Stage { stage, source }
    }
}

impl From<ProcessMessageError> for AppError {
    fn from(e: ProcessMessageError) -> Self {
        match e {
            ProcessMessageError::Stage { source, .. } => source,
            ProcessMessageError::CoPresent(c) => c.into(),
        }
    }
}

macro_rules! pm {
    ($e:expr, $stage:literal) => {
        $e.map_err(|source| ProcessMessageError::stage($stage, source))
    };
}

/// 处理一条用户消息：分析情绪 → 检测事件 → 演化性格 → 构建 Prompt → 调用 LLM → 持久化
pub async fn process_message(state: &AppState, req: &SendMessageRequest) -> Result<SendMessageResponse> {
    match run(state, req).await {
        Ok(v) => Ok(v),
        Err(e) => {
            log::error!(target: "oclive_chat", "{}", e);
            Err(e.into())
        }
    }
}

async fn run(
    state: &AppState,
    req: &SendMessageRequest,
) -> std::result::Result<SendMessageResponse, ProcessMessageError> {
    let mrid = req.role_id.as_str();
    let state_rid = conversation_state_role_id(mrid, req.session_id.as_deref());
    let srid = state_rid.as_str();
    let requested_scene_id = req
        .scene_id
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let (scene_id, scenes) = pm!(
        validate_scene_id(state, mrid, requested_scene_id),
        "validate_scene_id"
    )?;
    let t0 = Instant::now();
    log::debug!(
        target: "oclive_chat",
        "send_message start role_id={} scene_id={} session_ns={}",
        mrid,
        scene_id,
        srid
    );

    pm!(
        state.db_manager.ensure_role_runtime(srid).await,
        "ensure_role_runtime"
    )?;

    let role = pm!(ensure_role_loaded(state, mrid).await, "ensure_role_loaded")?;
    pm!(
        state
            .db_manager
            .ensure_interaction_mode_seeded(srid, role.interaction_mode.as_deref())
            .await,
        "ensure_interaction_mode_seeded"
    )?;
    let effective_backends = state.effective_plugin_backends_for_session(role.as_ref(), srid);
    let effective_sources = state.effective_plugin_backend_sources_for_session(srid);
    log::debug!(
        target: "oclive_chat",
        "send_message backends role_id={} scene_id={} session_ns={} {}",
        mrid,
        scene_id,
        srid,
        backend_resolution_summary(&effective_backends, &effective_sources)
    );

    pm!(
        startup_health::ensure_once(state, role.as_ref(), &effective_backends).await,
        "startup_health"
    )?;

    let pl = state.resolved_plugins_for_session(role.as_ref(), Some(srid));
    let agent_out = pm!(
        pl.agent
            .process(AgentInput {
                role_id: mrid.to_string(),
                session_namespace: srid.to_string(),
                message: req.user_message.clone(),
                model: role.resolve_ollama_model(state.ollama_model.as_str()),
            })
            .await,
        "agent_process"
    )?;
    if agent_out.handled {
        pm!(
            state
                .db_manager
                .set_user_presence_scene(srid, scene_id.as_str())
                .await,
            "set_user_presence_scene_agent"
        )?;
        let emotion_result = pm!(
            pl.emotion.analyze(req.user_message.as_str()),
            "agent_branch_emotion"
        )?;
        let user_relation_key = pm!(
            resolve_effective_user_relation_key(
                state,
                role.as_ref(),
                srid,
                Some(scene_id.as_str()),
            )
            .await,
            "agent_branch_resolve_user_relation"
        )?;
        let rel_id = pm!(
            state
                .db_manager
                .get_relation_state_for_identity(srid, user_relation_key.as_str())
                .await,
            "agent_branch_relation_identity"
        )?;
        let rel_global = pm!(
            state.db_manager.get_relation_state(srid).await,
            "agent_branch_relation_global"
        )?;
        let relation_state = rel_id
            .or(rel_global)
            .unwrap_or_else(|| "Stranger".to_string());
        let favor_current = pm!(
            state
                .db_manager
                .favorability_for_identity_with_runtime_fallback(srid, user_relation_key.as_str())
                .await,
            "agent_branch_favorability"
        )?;
        let portrait_emotion = pm!(
            state.db_manager.get_current_emotion(srid).await,
            "agent_branch_portrait_emotion"
        )?
        .unwrap_or_else(|| "neutral".to_string());
        return Ok(SendMessageResponse {
            api_version: API_VERSION,
            schema: SCHEMA_VERSION,
            presence_mode: PresenceMode::CoPresent,
            relation_state,
            reply: agent_out.reply,
            emotion: emotion_to_dto(&emotion_result),
            bot_emotion: portrait_emotion.clone(),
            portrait_emotion,
            favorability_delta: 0.0,
            favorability_current: favor_current as f32,
            events: vec![],
            scene_id,
            offer_destination_picker: false,
            offer_together_travel: false,
            reply_is_fallback: false,
            knowledge_chunks_in_prompt: 0,
            timestamp: chrono::Utc::now().timestamp_millis(),
        });
    }

    pm!(
        state
            .db_manager
            .set_user_presence_scene(srid, scene_id.as_str())
            .await,
        "set_user_presence_scene"
    )?;

    let current_scene = pm!(
        state.db_manager.get_current_scene(srid).await,
        "get_current_scene"
    )?;
    let immersive = pm!(
        state.db_manager.get_interaction_mode(srid).await,
        "get_interaction_mode"
    )?
    .is_immersive();
    let remote_life_enabled = pm!(
        state.db_manager.get_remote_life_enabled(srid).await,
        "get_remote_life_enabled"
    )?;
    let is_remote =
        immersive && user_is_remote_from_character(scene_id.as_str(), current_scene.as_deref());
    let preflight_ms = t0.elapsed().as_millis() as u64;
    if is_remote {
        if !remote_life_enabled {
            return Ok(pm!(
                process_remote_stub(
                    state,
                    req,
                    role.as_ref(),
                    scene_id.as_str(),
                    t0,
                    srid,
                    preflight_ms,
                )
                .await,
                "remote_stub"
            )?);
        }
        let char_scene = current_scene.as_deref().unwrap_or("default");
        return Ok(pm!(
            process_remote_life(
                state,
                req,
                role.as_ref(),
                scene_id.as_str(),
                char_scene,
                t0,
                mrid,
                srid,
                preflight_ms,
            )
            .await,
            "remote_life"
        )?);
    }

    Ok(co_present::process_co_present(
        state,
        req,
        role.as_ref(),
        scene_id,
        scenes,
        immersive,
        t0,
        mrid,
        srid,
        preflight_ms,
    )
    .await?)
}
