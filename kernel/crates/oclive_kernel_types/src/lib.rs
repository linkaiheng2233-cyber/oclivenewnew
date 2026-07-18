//! # oclive_kernel_types — pure data-structure layer
//!
//! **Role**: kernel-shared **DTOs, error types, and config structs** (`Role`, `SendMessageRequest`, `AppError`, etc.); **contains no business logic or I/O**.
//!
//! **Upstream**: [`oclive_validation`](https://docs.rs/oclive_validation) (blueprint / manifest validation types such as `SlotRegistryEntry`).
//! **Downstream**: `oclive_kernel_contracts`, `oclive_kernel_runtime`, and `src-tauri` model re-exports.
//!
//! **Decoupling note**: `oclive_validation` is a path dependency today; some manifest/blueprint types are
//! re-exported for convenience. A future split may move validation-only types behind a narrower boundary
//! so `oclive_kernel_types` can version independently without pulling the full validation crate surface.
//!
//! **Key decision**: types and behavior are kept separate so the contract crate can be versioned independently; prefer importing from this crate root or from [`models`] / [`error`].
//!
//! ## Public-export audit (maintenance convention)
//!
//! | Allowed | Forbidden |
//! |------|------|
//! | `struct` / `enum` DTOs, `Default`, `From`/`Into` mappings | database, HTTP, subprocess, or directory I/O |
//! | **read-only derivations** on `impl Role` (gating booleans, config parsing, manifest round-trip) | orchestration ordering, plugin resolution, session side effects |
//! | [`AppError`] and [`KernelErrorBody`] | calling `PluginHost` or an LLM inside types |
//!
//! `Role::resolve_ollama_model` only resolves the **string priority** of manifest / environment variable / global default; it does not issue network requests.
//! The dual-core gate [`Role::dual_core_gated`](models::role::Role::dual_core_gated) only reads already-loaded blueprint fields.
//! For disk ↔ memory conversion see [`models::role_manifest_disk`] / [`models::role_settings_disk`].

pub mod agent;
pub mod complex_emotion;
pub mod emotion;
pub mod error;
pub mod event_impact;
pub mod kernel_error_codes;
pub mod local_plugin;
pub mod mcp;
pub mod memory_retrieval;
pub mod models;
pub mod policy;
pub mod prompt;
pub mod slot_extension;

pub use agent::{
    AgentDebugTrace, AgentInput, AgentOutput, AgentProcessRpcResult, AgentRoleConstraints,
    AgentRpcToolCall, AgentToolCallTrace, AgentToolResult, AgentToolSchema, AgentTurnContext,
    FunctionCall, ToolCall, ToolSchemaInput,
};
pub use complex_emotion::{ComplexEmotionInput, ComplexEmotionOutput};
pub use emotion::EmotionResult;
pub use error::{http_chat_codes, AppError, KernelErrorBody, Result};
pub use event_impact::EventImpactEstimate;
pub use local_plugin::{
    LocalPluginCapability, LocalPluginProviderDescriptor, LOCAL_PLUGIN_SCHEMA_VERSION,
};
pub use mcp::{McpServerInfo, McpToolInfo};
pub use memory_retrieval::MemoryRetrievalInput;
pub use policy::{EmotionPolicyConfig, MemoryPolicyConfig, PolicyConfig, PolicyContext};
pub use prompt::{PromptExtraSection, PromptInput};
pub use slot_extension::SlotExtension;

pub use oclive_validation::{
    MemorySeedEntry, MemorySeedFile, PortableLongTermMemoryEntry, PortableMemoryFile,
    PortablePersonaFile, SlotGroupEntry, SlotRegistryEntry, MEMORY_SEED_SCHEMA_VERSION,
    PORTABLE_MEMORY_SCHEMA_VERSION, PORTABLE_PERSONA_SCHEMA_VERSION,
};

pub use models::{
    author_pack::{AuthorPackFile, AuthorRecommendedPlugin},
    chat::{ChatRequest, ChatResponse},
    dto::{
        ClearAllSessionSlotOverridesRequest, ClearSessionSlotOverrideRequest, CreateEventRequest,
        CreateEventResponse, DetectedEventDto, EmotionDto, ExportChatLogsRequest,
        ExportChatLogsResponse, GenerateMonologueRequest, GenerateMonologueResponse,
        GetPluginResolutionDebugRequest, GetRoleInfoRequest, GetUserIdentityStateRequest,
        ImportProgress, JumpTimeRequest, JumpTimeResponse, LifeStateDto, MemoryItem,
        PluginResolutionDebugInfo, PortableStateExportResponse, PortableStateImportRequest,
        PortableStateImportResponse, PortableStateRequest, PresenceMode, QueryEventsRequest,
        QueryMemoriesRequest, RoleData, RoleInfo, RoleSummary, SaveRoleSlotRegistryRequest,
        SceneLabelEntry, SendMessageRequest, SendMessageResponse, SetEvolutionFactorRequest,
        SetRemoteLifeEnabledRequest, SetRoleInteractionModeRequest, SetSceneUserIdentityRequest,
        SetSceneUserRelationRequest, SetSessionPluginBackendRequest, SetSessionSlotOverrideRequest,
        SetUserIdentityRequest, SetUserPresenceSceneRequest, SetUserRelationRequest,
        SwitchSceneRequest, SwitchSceneResponse, TheaterCastRef, TheaterForkTemplate,
        TheaterSceneRequest, TheaterSceneResponse, TheaterScriptLine, TheaterTweak,
        TimeStateResponse, UserIdentityDto, UserIdentityStateResponse, UserRelationDto,
        API_VERSION, OCLIVE_DEFAULT_IDENTITY_SENTINEL, OCLIVE_DEFAULT_RELATION_SENTINEL,
        SCHEMA_VERSION,
    },
    emotion::Emotion,
    event::{Event, EventType},
    favorability::Favorability,
    interaction_mode::InteractionMode,
    kernel::{
        ActiveProfileSummary, AttachReason, DistroProfileRequirements, KernelHealthJson,
        ProfileCompat, ReplaceReason,
    },
    knowledge::{KnowledgeChunk, KnowledgeEventAugment, KnowledgeIndex, KnowledgePackConfigDisk},
    memory::{Memory, MemoryContext},
    personality::PersonalityVector,
    plugin_backends::*,
    role::{
        EvolutionBounds, EvolutionConfig, LifeState, MemoryConfig, PersonalityDefaults,
        PersonalitySource, Role, UserRelation,
    },
    role_manifest_disk::{disk_manifest_from_role, disk_manifest_to_role, DiskRoleManifest},
    role_pack_config::{
        RolePackTurnThinkingConfig, TurnThinkingAndGroup, TurnThinkingDeepWhen,
        TurnThinkingEphemeralArchiveConfig, TurnThinkingLatchConfig, TurnThinkingSignalRule,
    },
    role_settings_disk::{disk_role_settings_from_role, DiskRoleSettings},
    scene_disk::{DiskSceneConfig, DiskSceneTimeWindow},
    ui_config::{LayoutConfig, SlotConfig, ThemeConfig, UiConfig, UiSlots},
};
