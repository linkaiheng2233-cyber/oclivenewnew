//! # oclive_kernel_types — 纯数据结构层
//!
//! **角色**：内核共享的 **DTO、错误类型、配置结构**（`Role`、`SendMessageRequest`、`AppError` 等）；**不包含业务逻辑或 I/O**。
//!
//! **上游**：[`oclive_validation`](https://docs.rs/oclive_validation)（蓝图 / manifest 校验类型如 `SlotRegistryEntry`）。
//! **下游**：`oclive_kernel_contracts`、`oclive_kernel_runtime`、`src-tauri` 模型再导出。
//!
//! **关键决策**：类型与行为分离，保证契约 crate 可独立版本化；优先从本 crate 根或 [`models`] / [`error`] 导入。
//!
//! ## 公开导出审计（维护约定）
//!
//! | 允许 | 禁止 |
//! |------|------|
//! | `struct` / `enum` DTO、`Default`、`From`/`Into` 映射 | 数据库、HTTP、子进程、目录 I/O |
//! | `impl Role` 上的**只读派生**（门控布尔、配置解析、manifest 往返） | 编排顺序、插件解析、会话副作用 |
//! | [`AppError`] 与 [`KernelErrorBody`] | 在 types 内调用 `PluginHost` 或 LLM |
//!
//! `Role::resolve_ollama_model` 仅解析 manifest / 环境变量 / 全局默认的**字符串优先级**，不发起网络请求。
//! 双核门控 [`Role::dual_core_gated`](models::role::Role::dual_core_gated) 只读取已加载蓝图字段。
//! 磁盘 ↔ 内存转换见 [`models::role_manifest_disk`] / [`models::role_settings_disk`]。

pub mod agent;
pub mod complex_emotion;
pub mod emotion;
pub mod error;
pub mod event_impact;
pub mod local_plugin;
pub mod memory_retrieval;
pub mod models;
pub mod policy;
pub mod prompt;

pub use agent::{AgentInput, AgentOutput};
pub use complex_emotion::{ComplexEmotionInput, ComplexEmotionOutput};
pub use event_impact::EventImpactEstimate;
pub use emotion::EmotionResult;
pub use error::{
    http_chat_codes, AppError, KernelErrorBody, Result,
};
pub use local_plugin::{
    LocalPluginCapability, LocalPluginProviderDescriptor, LOCAL_PLUGIN_SCHEMA_VERSION,
};
pub use memory_retrieval::MemoryRetrievalInput;
pub use policy::{
    EmotionPolicyConfig, MemoryPolicyConfig, PolicyConfig, PolicyContext,
};
pub use prompt::PromptInput;

pub use oclive_validation::{SlotGroupEntry, SlotRegistryEntry};

pub use models::{
    author_pack::{AuthorPackFile, AuthorRecommendedPlugin},
    chat::{ChatRequest, ChatResponse},
    dto::{
        API_VERSION, OCLIVE_DEFAULT_RELATION_SENTINEL, SCHEMA_VERSION, ClearAllSessionSlotOverridesRequest,
        ClearSessionSlotOverrideRequest, CreateEventRequest, CreateEventResponse, DetectedEventDto,
        EmotionDto, ExportChatLogsRequest, ExportChatLogsResponse, GenerateMonologueRequest,
        GenerateMonologueResponse, GetPluginResolutionDebugRequest, GetRoleInfoRequest, ImportProgress,
        JumpTimeRequest, JumpTimeResponse, LifeStateDto, MemoryItem, PluginResolutionDebugInfo,
        PresenceMode, QueryEventsRequest, QueryMemoriesRequest, RoleData, RoleInfo, RoleSummary,
        SaveRoleSlotRegistryRequest, SceneLabelEntry, SendMessageRequest, SendMessageResponse,
        SetEvolutionFactorRequest, SetRemoteLifeEnabledRequest, SetRoleInteractionModeRequest,
        SetSceneUserRelationRequest, SetSessionPluginBackendRequest, SetSessionSlotOverrideRequest,
        SetUserPresenceSceneRequest, SetUserRelationRequest, SwitchSceneRequest, SwitchSceneResponse,
        TimeStateResponse, UserRelationDto,
    },
    emotion::Emotion,
    event::{Event, EventType},
    favorability::Favorability,
    interaction_mode::InteractionMode,
    knowledge::{KnowledgeChunk, KnowledgeEventAugment, KnowledgeIndex, KnowledgePackConfigDisk},
    memory::{Memory, MemoryContext},
    personality::PersonalityVector,
    plugin_backends::*,
    role::{
        EvolutionBounds, EvolutionConfig, LifeState, MemoryConfig, PersonalityDefaults,
        PersonalitySource, Role, UserRelation,
    },
    role_manifest_disk::{disk_manifest_from_role, disk_manifest_to_role, DiskRoleManifest},
    role_settings_disk::{disk_role_settings_from_role, DiskRoleSettings},
    scene_disk::{DiskSceneConfig, DiskSceneTimeWindow},
    ui_config::{LayoutConfig, SlotConfig, ThemeConfig, UiConfig, UiSlots},
};
