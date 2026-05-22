//! 内核纯数据结构：DTO、[`AppError`]、枚举与策略/插件描述符。
//!
//! 稳定 API：通过本 crate 根路径或子模块 [`models`]、[`error`] 引用；避免依赖未文档化的内部 helper。

pub mod complex_emotion;
pub mod emotion;
pub mod error;
pub mod local_plugin;
pub mod memory_retrieval;
pub mod models;
pub mod policy;
pub mod prompt;

pub use complex_emotion::{ComplexEmotionInput, ComplexEmotionOutput};
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

pub use oclive_validation::SlotRegistryEntry;

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
        EvolutionBounds, EvolutionConfig, LifeState, MemoryConfig, PersonalityDefaults, Role,
        UserRelation,
    },
    role_manifest_disk::{disk_manifest_from_role, disk_manifest_to_role, DiskRoleManifest},
    role_settings_disk::{disk_role_settings_from_role, DiskRoleSettings},
    scene_disk::{DiskSceneConfig, DiskSceneTimeWindow},
    ui_config::{LayoutConfig, SlotConfig, ThemeConfig, UiConfig, UiSlots},
};
