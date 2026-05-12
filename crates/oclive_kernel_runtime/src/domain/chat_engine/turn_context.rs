//! 单次 `send_message` / `process_message` 回合内的共享可变上下文（蓝图调度引擎 v0 桩）。
//!
//! - 生命周期：在 `process_message` 入口创建，返回前销毁；不跨请求缓存。
//! - 不包含 `KernelAppState`：由各原子操作以参数传入。
//! - 首版字段多为 `Option`，由原子操作按顺序填充。

use crate::domain::agent::AgentOutput;
use crate::domain::emotion_analyzer::EmotionResult;
use crate::domain::plugin_host::ResolvedRolePlugins;
use crate::models::dto::SendMessageRequest;
use crate::models::Role;
use std::sync::Arc;
use std::time::Instant;

/// 角色与会话标识（manifest id + SQLite 命名空间）。
#[derive(Default, Clone)]
pub struct TurnIds {
    pub manifest_role_id: Option<String>,
    pub session_namespace: Option<String>,
    pub client_session_id: Option<String>,
}

/// 请求中与回合相关的只读镜像（便于原子操作少碰 `req`）。
#[derive(Default, Clone)]
pub struct TurnRequestView {
    pub user_message: Option<String>,
    pub requested_scene_id: Option<String>,
}

/// 校验后的场景。
#[derive(Default, Clone)]
pub struct TurnScene {
    pub effective_scene_id: Option<String>,
    pub scene_id_list: Option<Vec<String>>,
}

#[derive(Default)]
pub struct TurnRole {
    pub role: Option<Arc<Role>>,
}

#[derive(Clone)]
pub struct TurnPlugins {
    pub resolved: Option<ResolvedRolePlugins>,
}

impl Default for TurnPlugins {
    fn default() -> Self {
        Self { resolved: None }
    }
}

#[derive(Default, Clone)]
pub struct TurnLlmRouting {
    pub main_llm_model: Option<String>,
}

#[derive(Default)]
pub struct TurnAgent {
    pub output: Option<AgentOutput>,
}

#[derive(Default)]
pub struct TurnEmotion {
    pub user_emotion: Option<EmotionResult>,
}

#[derive(Default, Clone)]
pub struct TurnPresence {
    pub current_scene: Option<String>,
    pub immersive: Option<bool>,
    pub remote_life_enabled: Option<bool>,
    pub is_remote: Option<bool>,
}

#[derive(Default)]
pub struct TurnFlags {
    pub agent_handled: Option<bool>,
}

#[derive(Default)]
pub struct TurnTrace {
    pub started_at: Option<Instant>,
    pub preflight_ms: Option<u64>,
}

/// 单轮对话执行上下文（v0：仅承载 `process_message` 首段与 presence 路由所需状态）。
#[derive(Default)]
pub struct TurnContext {
    pub ids: TurnIds,
    pub request: TurnRequestView,
    pub scene: TurnScene,
    pub role: TurnRole,
    pub plugins: TurnPlugins,
    pub llm: TurnLlmRouting,
    pub agent: TurnAgent,
    pub emotion: TurnEmotion,
    pub presence: TurnPresence,
    pub flags: TurnFlags,
    pub trace: TurnTrace,
}

impl TurnContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// 从请求填充标识与请求镜像（须在 `validate_scene` 等之前调用）。
    pub fn bootstrap_from_request(&mut self, req: &SendMessageRequest) {
        let mrid = req.role_id.clone();
        self.ids.manifest_role_id = Some(mrid.clone());
        self.ids.session_namespace = Some(super::conversation_state_role_id(
            mrid.as_str(),
            req.session_id.as_deref(),
        ));
        self.ids.client_session_id = req.session_id.clone();
        self.request.user_message = Some(req.user_message.clone());
        self.request.requested_scene_id = Some(
            req.scene_id
                .clone()
                .unwrap_or_else(|| "default".to_string()),
        );
    }
}
