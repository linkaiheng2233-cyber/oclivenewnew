//! 目录式插件：扫描 `plugins/*/manifest.json`、懒启动子进程、缓存 JSON-RPC 根 URL。
//!
//! 契约见 `creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md`。

mod manifest;
mod runtime;

pub use manifest::{
    BridgeConfig, OclivePluginManifest, ShellSection, UiSlotDecl, normalize_plugin_rel,
};
pub use runtime::{DirectoryPluginRuntime, HostPluginsFile, PluginScanSummary};
