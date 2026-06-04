//! Directory plugins: scan `plugins/*/manifest.json`, lazy-start child processes, cache JSON-RPC base URL.
//!
//! Contract: `creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md`.

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
    plugin_scan_container_roots, resolve_plugin_asset_path, DirectoryPluginRuntime, HostPluginsFile,
    PluginProcessDebugInfo, PluginRootEntry, PluginScanSummary,
};
pub use version::parse_manifest_version;
