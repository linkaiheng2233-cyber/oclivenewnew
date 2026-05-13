//! 与 `oclive_kernel_core::error`（经 `oclive_kernel_runtime::error` 再导出）共用同一 `AppError` / `Result`。
//! Tauri 命令请使用 `map_err(|e: AppError| e.to_frontend_error())` 等到 `String`。

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
