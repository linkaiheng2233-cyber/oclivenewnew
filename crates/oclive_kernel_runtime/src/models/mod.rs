pub mod oocp;

// Temporary shim: re-export the rest from `oclivenewnew-tauri`.
//
// Goal: progressively migrate modules here and remove this dependency.
pub use oclivenewnew_tauri::models::{
    author_pack, chat, dto, emotion, error, event, favorability, interaction_mode, knowledge,
    memory, personality, plugin_backends, role, role_manifest_disk, role_settings_disk, scene_disk,
    ui_config,
};

pub use oclivenewnew_tauri::models::author_pack::{AuthorPackFile, AuthorRecommendedPlugin};
pub use oclivenewnew_tauri::models::chat::*;
pub use oclivenewnew_tauri::models::dto::*;
pub use oclivenewnew_tauri::models::emotion::*;
pub use oclivenewnew_tauri::models::error::*;
pub use oclivenewnew_tauri::models::event::*;
pub use oclivenewnew_tauri::models::favorability::*;
pub use oclivenewnew_tauri::models::interaction_mode::InteractionMode;
pub use oclivenewnew_tauri::models::knowledge::{
    KnowledgeEventAugment, KnowledgeIndex, KnowledgePackConfigDisk,
};
pub use oclivenewnew_tauri::models::memory::*;
pub use oclivenewnew_tauri::models::personality::*;
pub use oclivenewnew_tauri::models::plugin_backends::*;
pub use oclivenewnew_tauri::models::role::*;
pub use oclivenewnew_tauri::models::role_manifest_disk::DiskRoleManifest;
pub use oclivenewnew_tauri::models::role_settings_disk::DiskRoleSettings;
pub use oclivenewnew_tauri::models::scene_disk::{DiskSceneConfig, DiskSceneTimeWindow};
pub use oclivenewnew_tauri::models::ui_config::{
    LayoutConfig, SlotConfig, ThemeConfig, UiConfig, UiSlots,
};
