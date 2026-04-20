//! 目录式插件：扫描 `plugins/*/manifest.json`、懒启动子进程、缓存 JSON-RPC 根 URL。
//!
//! 契约见 `creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md`。

mod dependency;
mod manifest;
mod runtime;
mod version;
mod watcher;

pub use dependency::dependency_report;
pub use manifest::{
    normalize_plugin_rel, normalize_ui_slot_appearance_id, BridgeConfig, OclivePluginManifest,
    ShellSection, UiSlotDecl,
};
pub use runtime::{
    plugin_scan_container_roots, DirectoryPluginRuntime, HostPluginsFile, PluginProcessDebugInfo,
    PluginScanSummary,
};
pub use version::parse_manifest_version;
pub use watcher::start_plugin_fs_watcher;
