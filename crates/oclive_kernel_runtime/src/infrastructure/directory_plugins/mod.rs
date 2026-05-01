//! 目录式插件：扫描 `plugins/*/manifest.json`、懒启动子进程、缓存 JSON-RPC 根 URL。
//!
//! 契约见 `creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md`。
//! 文件系统热重载 watcher 仅在桌面壳（Tauri）中提供，内核 crate 不包含。

mod dependency;
mod manifest;
mod runtime;
mod version;

pub use dependency::dependency_report;
pub use manifest::{
    normalize_plugin_rel, normalize_ui_slot_appearance_id, BridgeConfig, OclivePluginManifest,
    ShellSection, UiSchemaField, UiSchemaSection, UiSlotDecl,
};
pub use runtime::{
    plugin_scan_container_roots, DirectoryPluginRuntime, HostPluginsFile, PluginProcessDebugInfo,
    PluginScanSummary,
};
pub use version::parse_manifest_version;
