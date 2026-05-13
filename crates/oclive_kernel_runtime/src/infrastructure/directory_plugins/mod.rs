//! 目录式插件：扫描 `plugins/*/manifest.json`、懒启动子进程、缓存 JSON-RPC 根 URL。
//!
//! 契约见 `creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md`。
//!
//! **留在宿主壳（Tauri）而不在本 crate 实现的能力**（见分期计划阶段 E）：
//! - `directory_plugin_invoke`：`DbManager` 权限/审计 + 阻塞 HTTP JSON-RPC。
//! - `spawn_blocking` / `remote_plugin` HTTP 客户端。
//! - `start_plugin_fs_watcher`（依赖 `tauri-app`）。
//!
//! 若未来无头 `kernel_server` 也需目录插件 RPC，再引入由宿主实现的 **`RpcInvoker` trait**，而非把 `reqwest` 拉进 domain。
//!
//! 文件系统热重载 watcher 仅在桌面壳（Tauri）中提供，内核 crate 不包含。

mod assets;
mod bootstrap;
mod catalog;
mod dependency;
mod install_meta;
mod manifest;
mod runtime;
mod version;

pub use assets::read_plugin_asset_text_under_root;
pub use bootstrap::{
    collect_subscribed_host_events, directory_plugin_bootstrap_dto, is_host_event_subscribed,
    merge_manifest_bridge_events, order_plugin_slots, pick_ui_slot_decl, shell_plugin_id_resolved,
    DEFAULT_DIRECTORY_PLUGIN_ASSET_BASE_URL, EMBEDDED_UI_SLOT_NAMES,
};
pub use catalog::build_directory_plugin_catalog;
pub use dependency::dependency_report;
pub use install_meta::{read_plugin_install_meta, write_plugin_install_meta};
pub use manifest::{
    normalize_plugin_rel, normalize_ui_slot_appearance_id, BridgeConfig, OclivePluginManifest,
    ShellSection, UiSchemaField, UiSchemaSection, UiSlotDecl,
};
pub use runtime::{
    plugin_scan_container_roots, DirectoryPluginRuntime, HostPluginsFile, PluginProcessDebugInfo,
    PluginScanSummary,
};
pub use version::parse_manifest_version;
