//! # oclive_kernel_types
//!
//! Pure data structures for the oclive kernel.
//! All DTOs, error types, and configuration structs.
//!
//! Prefer importing from this crate root or [`models`] / [`error`]; avoid undocumented internal helpers.

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
