//! 本地插件桥接抽象：当前保持与 `oclivenewnew-tauri` 一致，避免 plugin_host 的类型割裂。
//!
//! 待 `plugin_host` 一并下沉到 runtime 后，再把这里替换为 runtime 的单一真相源。

pub use oclivenewnew_tauri::domain::local_plugin_bridge::*;

