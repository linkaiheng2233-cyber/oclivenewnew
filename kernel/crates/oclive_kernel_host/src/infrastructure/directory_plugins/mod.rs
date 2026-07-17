//! Directory plugins: scan `plugins/*/manifest.json`, lazy-start child processes, cache JSON-RPC base URL.
//!
//! Contract: `creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md`.

pub mod bootstrap_dto;
mod dependency;
mod manifest;
mod runtime;
mod version;
pub use dependency::dependency_report;
pub(crate) use manifest::validate_plugin_id;
pub use manifest::{
    normalize_plugin_rel, normalize_ui_slot_appearance_id, BridgeConfig, OclivePluginManifest,
    ShellSection, UiSchemaField, UiSchemaSection, UiSlotDecl,
};
pub use runtime::{
    find_plugin_asset_path, plugin_scan_container_roots, DirectoryPluginRuntime, HostPluginsFile,
    PluginProcessDebugInfo, PluginRootEntry, PluginScanSummary,
};
pub use version::parse_manifest_version;

#[must_use]
pub(crate) fn rpc_url_is_loopback(url: &str) -> bool {
    runtime::rpc_url_is_loopback(url)
}
