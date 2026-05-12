//! 单次 `send_message` / `process_message` 回合内的共享可变上下文（蓝图调度引擎 v0）。
//!
//! - 生命周期：在 `process_message` 入口创建，返回前销毁；不跨请求缓存。
//! - 不包含 `KernelAppState`：由各原子操作以参数传入。
//! - 字段多为 `Option`：由入口 `bootstrap_from_request` 与原子操作按序填充；**共景 / 异地心声 / remote_life** 等分支专用字段仅在对应路径写入，其余时间保持 `None` 为正常状态。

use super::pipeline_loader::PipelineBlueprint;
use crate::domain::agent::AgentOutput;
use crate::domain::emotion_analyzer::EmotionResult;
use crate::domain::plugin_host::ResolvedRolePlugins;
use crate::models::dto::SendMessageRequest;
use crate::models::Role;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

/// 角色与会话标识（manifest id + SQLite 命名空间）。
#[derive(Default, Clone)]
pub struct TurnIds {
    /// 入口 `bootstrap_from_request` 即写入。
    pub manifest_role_id: Option<String>,
    /// 入口 `bootstrap_from_request` 即写入（由 `role_id` + `session_id` 派生命名空间）。
    pub session_namespace: Option<String>,
    /// 客户端会话 id；部分遥测或链式状态使用，主路径可不依赖。
    pub client_session_id: Option<String>,
}

/// 请求中与回合相关的只读镜像（便于原子操作少碰 `req`）。
#[derive(Default, Clone)]
pub struct TurnRequestView {
    /// 入口 `bootstrap_from_request` 即写入。
    pub user_message: Option<String>,
    /// 入口 `bootstrap_from_request` 即写入（缺省场景时为 `"default"`）。
    pub requested_scene_id: Option<String>,
}

/// 校验后的场景。
#[derive(Default, Clone)]
pub struct TurnScene {
    /// `validate_scene` 原子写入；其后主流程可 `expect`。
    pub effective_scene_id: Option<String>,
    /// `validate_scene` 写入可选场景列表。
    pub scene_id_list: Option<Vec<String>>,
}

#[derive(Default)]
pub struct TurnRole {
    /// `load_role` 写入；入口蓝图段在 `run_agent` 之前应已填充。
    pub role: Option<Arc<Role>>,
}

#[derive(Clone)]
pub struct TurnPlugins {
    /// `resolve_plugins` 写入；`analyze_emotion_user` / `run_agent` 等依赖。
    pub resolved: Option<ResolvedRolePlugins>,
}

impl Default for TurnPlugins {
    fn default() -> Self {
        Self { resolved: None }
    }
}

#[derive(Default, Clone)]
pub struct TurnLlmRouting {
    /// `resolve_main_llm_model` 写入。
    pub main_llm_model: Option<String>,
}

#[derive(Default)]
pub struct TurnAgent {
    /// `run_agent` / `generate_response` 写入。
    pub output: Option<AgentOutput>,
}

#[derive(Default)]
pub struct TurnEmotion {
    /// 共景 / Agent 早退分支中 `analyze_emotion_user` 写入；纯远程心声路径可能仍为 `None` 直至对应分析步骤。
    pub user_emotion: Option<EmotionResult>,
}

#[derive(Default, Clone)]
pub struct TurnPresence {
    /// `load_presence_routing` 从 DB 读取的当前场景；未走该原子前为 `None`。
    pub current_scene: Option<String>,
    /// `load_presence_routing` 写入。
    pub immersive: Option<bool>,
    /// `load_presence_routing` 写入（remote_life 开关）。
    pub remote_life_enabled: Option<bool>,
    /// `load_presence_routing` 根据场景与沉浸模式计算；共景路径在路由写入前可能未填充。
    pub is_remote: Option<bool>,
}

#[derive(Default)]
pub struct TurnFlags {
    /// `run_agent` 写入；`branch` 谓词 `agentHandled` 等使用。
    pub agent_handled: Option<bool>,
}

#[derive(Default)]
pub struct TurnTrace {
    /// `process_message` 在 `validate_scene` 之后立即设置，用于 `preflight_ms` 等计时。
    pub started_at: Option<Instant>,
    /// `load_presence_routing` 在存在 `started_at` 时写入预检耗时。
    pub preflight_ms: Option<u64>,
}

/// 本回合可选的 `pipeline.ocblueprint` 加载结果（v0）。
#[derive(Default, Clone)]
pub struct TurnPipeline {
    pub blueprint: Option<PipelineBlueprint>,
    pub loaded_from: Option<PathBuf>,
    /// 人类可读错误串（含 `[PIPELINE_*]` 前缀），供日志与 UI 诊断；加载成功时为 `None`。
    pub load_error: Option<String>,
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
    pub pipeline: TurnPipeline,
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
