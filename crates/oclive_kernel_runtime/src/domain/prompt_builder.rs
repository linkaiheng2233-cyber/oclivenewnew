//! 提示词构建：当前保持与 `oclivenewnew-tauri` 一致，避免 plugin_host/remote_plugin 的 trait 族撕裂。
//!
//! 待 `plugin_host` + `remote_plugin` + 相关 trait 一并下沉到 runtime 后，再把这里变成 runtime 的单一真相源。

pub use oclivenewnew_tauri::domain::prompt_builder::*;
