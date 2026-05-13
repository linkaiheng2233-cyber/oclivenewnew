//! 目录式插件：主体在 `oclive_kernel_runtime::infrastructure::directory_plugins`。
//! 此处仅保留 Tauri 文件系统 watcher（`tauri-app`）。

pub use oclive_kernel_runtime::infrastructure::directory_plugins::{
    build_directory_plugin_catalog, dependency_report, directory_plugin_bootstrap_dto,
    is_host_event_subscribed, normalize_plugin_rel, normalize_ui_slot_appearance_id,
    parse_manifest_version, plugin_scan_container_roots, read_plugin_asset_text_under_root,
    BridgeConfig, DirectoryPluginRuntime, HostPluginsFile, OclivePluginManifest,
    PluginProcessDebugInfo, PluginScanSummary, ShellSection, UiSchemaField, UiSchemaSection,
    UiSlotDecl, DEFAULT_DIRECTORY_PLUGIN_ASSET_BASE_URL,
};

#[cfg(feature = "tauri-app")]
mod watcher;
#[cfg(feature = "tauri-app")]
pub use watcher::start_plugin_fs_watcher;
