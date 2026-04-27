// Temporary shim: keep runtime error type aligned with the current domain/runtime
// implementation still hosted in `oclivenewnew-tauri`.
//
// Once `domain/*` and `state/*` are migrated into `oclive_kernel_runtime`, we can move
// `AppError` here as the single source of truth.
pub use oclivenewnew_tauri::error::*;
