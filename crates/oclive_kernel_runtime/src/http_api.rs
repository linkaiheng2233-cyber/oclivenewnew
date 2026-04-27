// Temporary shim: reuse existing implementation from `oclivenewnew-tauri`.
//
// Goal (next steps): move implementation here and make `src-tauri` depend on this crate
// (distribution -> kernel_runtime), not the other way around.

pub use oclivenewnew_tauri::http_api::*;

