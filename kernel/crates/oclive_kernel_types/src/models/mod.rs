pub mod author_pack;
pub mod chat;
pub mod dto;
pub mod emotion;
pub mod event;
pub mod favorability;
pub mod interaction_mode;
pub mod kernel;
pub mod knowledge;
pub mod memory;
pub mod meta_action_templates_config;
pub mod personality;
pub mod plugin_backends;
pub mod portrait_catalog_config;
pub mod reply_post_processor_config;
pub mod role;
pub mod role_manifest_disk;
pub mod role_pack_config;
pub mod role_settings_disk;
pub mod role_time_config;
pub mod scene_disk;
pub mod ui_config;
pub mod user_identity;
pub mod visual_presentation_config;

pub use author_pack::{AuthorPackFile, AuthorRecommendedPlugin};
pub use chat::*;
pub use dto::*;
pub use emotion::*;
pub use event::*;
pub use favorability::*;
pub use interaction_mode::InteractionMode;
pub use kernel::{
    ActiveProfileSummary, AttachReason, DistroProfileRequirements, KernelHealthJson, ProfileCompat,
    ReplaceReason,
};
pub use knowledge::{KnowledgeEventAugment, KnowledgeIndex, KnowledgePackConfigDisk};
pub use memory::*;
pub use meta_action_templates_config::{
    MetaActionTemplateEntry, RolePackMetaActionTemplatesConfig,
};
pub use personality::*;
pub use plugin_backends::*;
pub use portrait_catalog_config::{
    PortraitAssetKind, PortraitAssetResources, PortraitCatalogAsset, PortraitCatalogFile,
    PortraitCatalogToggle, SIMPLE_PORTRAIT_SLOT_IDS,
};
pub use reply_post_processor_config::{
    ReplyPostProcessorBackendKind, RolePackBuiltinReplyPostProcessorConfig,
    RolePackDirectoryReplyPostProcessorConfig, RolePackRemoteReplyPostProcessorConfig,
    RolePackReplyPostProcessorConfig,
};
pub use role::*;
pub use role_manifest_disk::DiskRoleManifest;
pub use role_pack_config::{
    RolePackChatStorageConfig, RolePackConfigFile, RolePackEvolutionConfig, RolePackMemoryConfig,
    RolePackRelationConfig, RolePackTurnThinkingConfig, TurnThinkingAndGroup,
    TurnThinkingDeepWhen, TurnThinkingEphemeralArchiveConfig, TurnThinkingLatchConfig,
    TurnThinkingSignalRule,
};
pub use role_settings_disk::{disk_role_settings_from_role, DiskRoleSettings};
pub use role_time_config::{RoleTimeConfig, DEFAULT_REAL_TO_VIRTUAL_RATIO};
pub use scene_disk::{DiskSceneConfig, DiskSceneTimeWindow};
pub use ui_config::{LayoutConfig, SlotConfig, ThemeConfig, UiConfig, UiSlots};
pub use user_identity::{
    UserIdentityCatalog, UserIdentityCatalogEntry, UserIdentityIndex, UserIdentityIndexEntry,
};
pub use visual_presentation_config::{
    PerformanceDirective, RolePackVisualPresentationConfig, VisualPresentationBackendKind,
};
