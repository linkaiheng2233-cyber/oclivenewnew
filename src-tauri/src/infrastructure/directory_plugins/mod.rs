//! 目录式插件：主体在 `oclive_kernel_runtime::infrastructure::directory_plugins`。
//! 此处仅保留 Tauri 文件系统 watcher（`tauri-app`）。

pub use oclive_kernel_runtime::infrastructure::directory_plugins::{
    dependency_report, normalize_plugin_rel, normalize_ui_slot_appearance_id,
    parse_manifest_version, plugin_scan_container_roots, BridgeConfig, DirectoryPluginRuntime,
    HostPluginsFile, OclivePluginManifest, PluginProcessDebugInfo, PluginScanSummary, ShellSection,
    UiSchemaField, UiSchemaSection, UiSlotDecl,
};

#[cfg(feature = "tauri-app")]
mod watcher;
#[cfg(feature = "tauri-app")]
pub use watcher::start_plugin_fs_watcher;
