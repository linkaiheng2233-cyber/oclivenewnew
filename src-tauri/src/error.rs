//! 与 `oclive_kernel_runtime::error` 共用同一 `AppError` / `Result`。
//! `AppError -> InvokeError` 在启用 `oclive_kernel_runtime/tauri_invoke` 时由内核 crate 实现。

pub use oclive_kernel_runtime::error::{AppError, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AppError::RoleNotFound("test".to_string());
        assert_eq!(err.to_string(), "Role not found: test");
    }

    #[test]
    fn test_result_type() {
        let _: Result<i32> = Err(AppError::Unknown("test".to_string()));
    }
}
