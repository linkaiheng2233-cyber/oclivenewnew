//! 前后端契约（Tauri invoke）

use crate::models::plugin_backends::PluginBackends;
use crate::models::plugin_backends::PluginBackendsOverride;
use crate::models::plugin_backends::PluginBackendsSourceMap;
use crate::models::ui_config::UiConfig;
use crate::models::role::IdentityBinding;
use crate::models::role::LifeState;
use crate::models::role::PersonalitySource;
use serde::{Deserialize, Serialize};

pub const API_VERSION: u32 = 1;
pub const SCHEMA_VERSION: u32 = 11;

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub role_id: String,
    pub user_message: String,
    #[serde(default)]
    pub scene_id: Option<String>,
    /// 可选：区分同角色的多路会话（如 HTTP 试聊「新会话」）；与 `role_id` 组合为内部 DB 命名空间。
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EmotionDto {
    pub joy: f32,
    pub sadness: f32,
    pub anger: f32,
    pub fear: f32,
    pub surprise: f32,
    pub disgust: f32,
    pub neutral: f32,
}

#[derive(Debug, Serialize)]
pub struct DetectedEventDto {
    pub event_type: String,
    pub confidence: f32,
}

/// `send_message` 的共景 / 异地占位 / 异地心声 模式（供 UI 样式与调试）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceMode {
    CoPresent,
    RemoteStub,
    RemoteLife,
}

#[derive(Debug, Serialize)]
pub struct SendMessageResponse {
    pub api_version: u32,
    pub schema: u32,
    /// 共景 / 异地占位 / 异地心声
    pub presence_mode: PresenceMode,
    /// 本回合结束后的关系阶段（`role_runtime.relation_state`）
    pub relation_state: String,
    pub reply: String,
    /// 用户输入侧情绪分析（七维），供调试或 UI 高级展示
    pub emotion: EmotionDto,
    /// 本回合解析后的 bot 情绪标签（小写英文，与 `Emotion::Display` 一致）
    pub bot_emotion: String,
    /// 立绘用表情（LLM+人设+事件综合；`role_runtime.current_emotion` 与此一致）
    pub portrait_emotion: String,
    pub favorability_delta: f32,
    pub favorability_current: f32,
    pub events: Vec<DetectedEventDto>,
    pub scene_id: String,
    /// 用户表达了位移/前往意图时由后端置 true；实际切换仅通过 `switch_scene`。
    pub offer_destination_picker: bool,
    /// 规则/模型判定用户邀请角色「同行前往」时置 true；确认后应 `switch_scene`（`together: true`）。
    #[serde(default)]
    pub offer_together_travel: bool,
    /// 主对话 LLM 失败时是否使用了备用短回复（共景 / 异地心声均可能）。
    #[serde(default)]
    pub reply_is_fallback: bool,
    /// 本回合写入主/异地 Prompt 的检索到的知识块条数（0 表示未注入或未命中）。
    #[serde(default)]
    pub knowledge_chunks_in_prompt: u32,
    pub timestamp: i64,
}

// ----- WEEK3-004：角色 / 记忆 / 事件查询 -----

/// 身份下拉里「跟随创作者 manifest 默认身份」选项提交该常量（非 manifest 键名）。
pub const OCLIVE_DEFAULT_RELATION_SENTINEL: &str = "__oclive_default__";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRelationDto {
    pub id: String,
    pub name: String,
    pub prompt_hint: String,
    pub favor_multiplier: f32,
    /// 角色包为该身份配置的初始好感度（0～100）；切换身份时会同步到当前好感。
    pub initial_favorability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleData {
    pub role_id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub personality_vector: Vec<f64>,
    pub current_favorability: f64,
    /// 占位；持久化 bot 情绪见后续里程碑
    pub current_emotion: String,
    pub memory_count: i32,
    pub event_count: i32,
    /// 角色包内定义的关系选项
    pub user_relations: Vec<UserRelationDto>,
    /// manifest 默认关系键
    pub default_relation: String,
    /// 关系阶段（好感度驱动的阶段，如 Stranger / Friend）
    pub relation_state: String,
    /// 当前运行时关系（解析后的 manifest 键）
    pub current_user_relation: String,
    pub use_manifest_default: bool,
    /// 异地时是否生成「生活轨迹与心声」（用户开关，持久化于 `role_runtime`）
    pub remote_life_enabled: bool,
    /// 角色包 `settings.json` 中 `remote_presence.default_enabled`（模式开关建议值），仅作 UI 默认提示
    pub remote_life_pack_default: Option<bool>,
    /// 有效事件影响系数（DB 覆盖 manifest 默认值）
    pub event_impact_factor: f64,
    /// `evolution.personality_source`：`vector` | `profile`
    #[serde(default)]
    pub personality_source: PersonalitySource,
    /// 本角色实际使用的 Ollama 模型（manifest → `OLLAMA_MODEL` → 全局默认）
    pub effective_ollama_model: String,
    /// 身份是否与场景绑定（manifest `identity_binding`）
    pub identity_binding: IdentityBinding,
    /// 当前交互模式（`role_runtime`）
    pub interaction_mode: String,
    /// 角色包 `settings.json` 建议默认（可选）
    pub interaction_mode_pack_default: Option<String>,
    /// 当前日程推断（无配置或未命中时段时为 `null`）
    #[serde(default)]
    pub current_life: Option<LifeStateDto>,
    /// `settings.json` → `plugin_backends`（与运行时 `PluginHost` 解析一致）
    #[serde(default)]
    pub plugin_backends: PluginBackends,
    /// 会话级覆盖（仅当前会话命名空间；无覆盖为 `null`）。
    #[serde(default)]
    pub plugin_backends_session_override: Option<PluginBackendsOverride>,
    /// 会话级覆盖叠加后的有效后端（供运行时面板展示与切换回显）。
    #[serde(default)]
    pub plugin_backends_effective: PluginBackends,
    /// 有效后端来源（pack/session/env）。
    #[serde(default)]
    pub plugin_backends_effective_sources: PluginBackendsSourceMap,
    /// 角色包根目录 `ui.json`（主题、布局、插槽等）。
    #[serde(default)]
    pub pack_ui_config: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneLabelEntry {
    pub id: String,
    pub label: String,
}

/// 虚拟时间 + manifest `life_schedule` 推断的当前活动（供 UI / 调试）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeStateDto {
    pub label: String,
    pub activity_key: String,
    pub busy_level: f32,
    pub preferred_scene_id: Option<String>,
}

impl From<&LifeState> for LifeStateDto {
    fn from(s: &LifeState) -> Self {
        Self {
            label: s.label.clone(),
            activity_key: s.activity_key.clone(),
            busy_level: s.busy_level,
            preferred_scene_id: s.optional_scene_hint.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleInfo {
    pub role_id: String,
    pub role_name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub current_favorability: f64,
    pub current_emotion: String,
    pub personality_vector: Vec<f64>,
    /// `evolution.personality_source`：`vector` | `profile`
    #[serde(default)]
    pub personality_source: PersonalitySource,
    pub last_interaction: Option<String>,
    /// 可用场景 id（manifest + `scenes/` 目录）
    pub scenes: Vec<String>,
    /// 与 `scenes` 顺序一致；`label` 来自 `scenes/{id}/scene.json` 的 `name` 或内置映射
    pub scene_labels: Vec<SceneLabelEntry>,
    pub current_scene: Option<String>,
    /// 用户叙事/发消息上下文场景（持久化）；与 `current_scene` 可不同。
    pub user_presence_scene: Option<String>,
    /// 虚拟世界时间（UTC 毫秒）；0 表示尚未通过 `get_time_state` 初始化
    pub virtual_time_ms: i64,
    pub user_relations: Vec<UserRelationDto>,
    pub default_relation: String,
    pub current_user_relation: String,
    /// 用户是否选择「默认身份」（跟随 manifest `default_relation`）；为 true 时下拉应选中 `OCLIVE_DEFAULT_RELATION_SENTINEL`。
    pub use_manifest_default: bool,
    /// 关系阶段（`role_runtime.relation_state`）
    pub relation_state: String,
    /// 异地心声开关
    pub remote_life_enabled: bool,
    /// 角色包建议默认是否开启异地心声（`settings.json` → `remote_presence.default_enabled`）
    pub remote_life_pack_default: Option<bool>,
    pub event_impact_factor: f64,
    /// 本角色实际使用的 Ollama 模型（manifest → `OLLAMA_MODEL` → 全局默认）
    pub effective_ollama_model: String,
    /// 身份是否与场景绑定（manifest `identity_binding`）
    pub identity_binding: IdentityBinding,
    /// 当前交互模式（`role_runtime`）
    pub interaction_mode: String,
    /// 角色包 `settings.json` 建议默认（可选）
    pub interaction_mode_pack_default: Option<String>,
    /// 当前日程推断（无配置或未命中时段时为 `null`）
    #[serde(default)]
    pub current_life: Option<LifeStateDto>,
    /// `settings.json` → `plugin_backends`（与 `load_role` / 编排层一致）
    #[serde(default)]
    pub plugin_backends: PluginBackends,
    /// 会话级覆盖（仅当前会话命名空间；无覆盖为 `null`）。
    #[serde(default)]
    pub plugin_backends_session_override: Option<PluginBackendsOverride>,
    /// 会话级覆盖叠加后的有效后端（供运行时面板展示与切换回显）。
    #[serde(default)]
    pub plugin_backends_effective: PluginBackends,
    /// 有效后端来源（pack/session/env）。
    #[serde(default)]
    pub plugin_backends_effective_sources: PluginBackendsSourceMap,
    /// 当前磁盘加载的角色是否含世界观知识索引（`knowledge_index` 已构建）
    #[serde(default)]
    pub knowledge_enabled: bool,
    /// `knowledge_index.chunks` 条数；未加载索引时为 0
    #[serde(default)]
    pub knowledge_chunk_count: i32,
    /// 角色包根目录 `ui.json`（主题、布局、插槽等）。
    #[serde(default)]
    pub pack_ui_config: UiConfig,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportProgress {
    pub percent: i32,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetUserRelationRequest {
    pub role_id: String,
    pub relation: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetSceneUserRelationRequest {
    pub role_id: String,
    pub scene_id: String,
    pub relation: String,
}

/// 移除当前场景的身份覆盖，使对话身份回退到全局有效身份（`use_manifest_default` / `user_relation`）。
#[derive(Debug, Clone, Deserialize)]
pub struct ClearSceneUserRelationRequest {
    pub role_id: String,
    pub scene_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetEvolutionFactorRequest {
    pub role_id: String,
    pub event_impact_factor: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetRemoteLifeEnabledRequest {
    pub role_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetSessionPluginBackendRequest {
    pub role_id: String,
    /// `memory` | `emotion` | `event` | `prompt` | `llm`
    pub module: String,
    /// 后端值（snake_case）三态：
    /// - 字段缺省：不修改该模块覆盖；
    /// - `null`：移除该模块会话覆盖并回退角色包默认；
    /// - `"xxx"`：设置为指定后端。
    #[serde(default)]
    pub backend: Option<Option<String>>,
    /// 仅当 `module = memory` 时生效：trim 后非空则设置本会话 `local_memory_provider_id`；
    /// 空串表示清除该字段的会话覆盖。字段缺省表示不修改该字段。
    #[serde(default)]
    pub local_memory_provider_id: Option<String>,
    /// 可选：HTTP 试聊等多会话场景下指定会话 id；缺省表示角色默认会话。
    #[serde(default)]
    pub session_id: Option<String>,
}

/// 查询运行时快照；`session_id` 与 `SendMessageRequest` 中同名字段同语义（多路试聊等）。
#[derive(Debug, Clone, Deserialize)]
pub struct GetRoleInfoRequest {
    pub role_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetPluginResolutionDebugRequest {
    pub role_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResolutionDebugInfo {
    pub app_version: String,
    pub api_version: u32,
    pub schema_version: u32,
    pub role_id: String,
    pub session_namespace: String,
    pub plugin_backends_pack_default: PluginBackends,
    #[serde(default)]
    pub plugin_backends_session_override: Option<PluginBackendsOverride>,
    pub plugin_backends_effective: PluginBackends,
    pub plugin_backends_effective_sources: PluginBackendsSourceMap,
    #[serde(default)]
    pub llm_env_override: Option<String>,
    pub remote_plugin_url_configured: bool,
    pub remote_llm_url_configured: bool,
    #[serde(default)]
    pub local_provider_ids: Vec<String>,
    pub local_provider_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeStateResponse {
    pub virtual_time_ms: i64,
    pub iso_datetime: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetRoleInteractionModeRequest {
    pub role_id: String,
    pub mode: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JumpTimeRequest {
    pub role_id: String,
    #[serde(default)]
    pub timestamp_ms: Option<i64>,
    #[serde(default)]
    pub preset: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JumpTimeResponse {
    pub virtual_time_ms: i64,
    pub iso_datetime: String,
    /// 时间跳转后生成的独白（通常 2 条，供前端插入聊天）
    pub monologues: Vec<String>,
    pub favorability_delta: f32,
    pub favorability_current: f32,
    /// 若 `autonomous_scene` 规则将角色 `current_scene` 从 `from` 切到 `to`
    #[serde(default)]
    pub autonomous_scene_from: Option<String>,
    #[serde(default)]
    pub autonomous_scene_to: Option<String>,
}

fn default_switch_together() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct SwitchSceneRequest {
    pub role_id: String,
    pub scene_id: String,
    /// `true`：写入 `current_scene` 并视为与角色同场景；`false`：仅更新 `user_presence_scene`（叙事独处）。
    #[serde(default = "default_switch_together")]
    pub together: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetUserPresenceSceneRequest {
    pub role_id: String,
    pub scene_id: String,
}

/// `.ocpak` 导入前预览（manifest）。
#[derive(Debug, Clone, Serialize)]
pub struct RolePackPeekResponse {
    pub id: String,
    pub name: String,
    pub version: String,
}

/// `switch_scene` 返回：角色信息与场景欢迎语（供前端插入聊天）。
#[derive(Debug, Clone, Serialize)]
pub struct SwitchSceneResponse {
    #[serde(flatten)]
    pub role: RoleInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scene_welcome: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateMonologueRequest {
    pub role_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateMonologueResponse {
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportChatLogsRequest {
    pub role_id: Option<String>,
    #[serde(default)]
    pub all_roles: bool,
    pub format: String,
    /// 可选：在导出内容中附加插件后端解析诊断（默认关闭；`all_roles=true` 时忽略）。
    #[serde(default)]
    pub include_plugin_resolution_debug: bool,
    /// 可选：诊断命名空间使用的会话 id（仅 `include_plugin_resolution_debug=true` 且单角色导出时生效）。
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportChatLogsResponse {
    pub content: String,
    pub suggested_filename: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryMemoriesRequest {
    pub role_id: String,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: String,
    pub role_id: String,
    pub content: String,
    /// 当前库仅为长期记忆表，固定为 `long_term`
    pub memory_type: String,
    pub timestamp: String,
    pub importance: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryEventsRequest {
    pub role_id: String,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventItem {
    pub id: i64,
    pub role_id: String,
    pub event_type: String,
    pub user_emotion: Option<String>,
    pub bot_emotion: Option<String>,
    pub timestamp: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateEventRequest {
    pub role_id: String,
    pub event_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventResponse {
    pub id: i64,
    pub role_id: String,
    pub event_type: String,
    pub timestamp: String,
    pub description: Option<String>,
}
