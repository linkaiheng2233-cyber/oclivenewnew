//! Host error bridge: core types live in `oclive_kernel_runtime`.

pub use oclive_kernel_runtime::error::*;

/// Map kernel [`AppError`] to Tauri invoke failure (orphan-safe helper).
#[must_use]
pub fn to_invoke_error(err: AppError) -> tauri::InvokeError {
    tauri::InvokeError::from(err.to_kernel_json())
}
