// Temporary shim: re-export runtime modules from `oclivenewnew-tauri`.
//
// Goal: progressively migrate code here and remove this dependency.

pub mod repositories_runtime;

pub use oclivenewnew_tauri::infrastructure::*;
