//! 角色与 DTO：复用 `oclive_kernel_runtime::models`（单一真相源）。
//! `AppError` / `Result` 仍来自本 crate 的 `crate::error`（含 Tauri `InvokeError` 等）。

pub use oclive_kernel_runtime::models::{
    author_pack, chat, dto, emotion, event, expert_models, favorability, interaction_mode,
    knowledge, memory, oocp, personality, plugin_backends, role, role_manifest_disk,
    role_settings_disk, scene_disk, ui_config,
};

pub use crate::error::*;
pub use oclive_kernel_runtime::models::author_pack::{AuthorPackFile, AuthorRecommendedPlugin};
pub use oclive_kernel_runtime::models::chat::*;
pub use oclive_kernel_runtime::models::dto::*;
pub use oclive_kernel_runtime::models::emotion::*;
pub use oclive_kernel_runtime::models::event::*;
pub use oclive_kernel_runtime::models::expert_models::*;
pub use oclive_kernel_runtime::models::favorability::*;
pub use oclive_kernel_runtime::models::interaction_mode::InteractionMode;
pub use oclive_kernel_runtime::models::knowledge::{
    KnowledgeEventAugment, KnowledgeIndex, KnowledgePackConfigDisk,
};
pub use oclive_kernel_runtime::models::memory::*;
pub use oclive_kernel_runtime::models::personality::*;
pub use oclive_kernel_runtime::models::plugin_backends::*;
pub use oclive_kernel_runtime::models::role::*;
pub use oclive_kernel_runtime::models::role_manifest_disk::DiskRoleManifest;
pub use oclive_kernel_runtime::models::role_settings_disk::DiskRoleSettings;
pub use oclive_kernel_runtime::models::scene_disk::{DiskSceneConfig, DiskSceneTimeWindow};
pub use oclive_kernel_runtime::models::ui_config::{
    LayoutConfig, SlotConfig, ThemeConfig, UiConfig, UiSlots,
};
