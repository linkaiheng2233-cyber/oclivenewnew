use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Ollama error: {0}")]
    OllamaError(String),

    #[error("Role not found: {0}")]
    RoleNotFound(String),

    /// 尚未 `load_role` 或 `role_runtime` 行缺失时，避免用泛型 `INVALID_PARAMETER` 误导用户。
    #[error("Role runtime not initialized; call load_role first")]
    RoleRuntimeNotReady,

    #[error("Startup health failed: {0}")]
    StartupHealthFailed(String),

    #[error("Role already exists; overwrite required: {0}")]
    RolePackExists(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),

    #[error("Transaction failed ({code}): {message}")]
    TransactionError { code: &'static str, message: String },
}

pub type Result<T> = std::result::Result<T, AppError>;

impl AppError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            AppError::DatabaseError(_) => "DB_ERROR",
            AppError::IoError(_) => "IO_ERROR",
            AppError::OllamaError(_) => "LLM_ERROR",
            AppError::RoleNotFound(_) => "ROLE_NOT_FOUND",
            AppError::RoleRuntimeNotReady => "ROLE_RUNTIME_NOT_READY",
            AppError::StartupHealthFailed(_) => "STARTUP_HEALTH_FAILED",
            AppError::RolePackExists(_) => "ROLE_PACK_EXISTS",
            AppError::InvalidParameter(_) => "INVALID_PARAMETER",
            AppError::SerializationError(_) => "SERDE_ERROR",
            AppError::Unknown(_) => "UNKNOWN_ERROR",
            AppError::TransactionError { code, .. } => code,
        }
    }

    #[must_use]
    pub fn to_frontend_error(&self) -> String {
        format!("[{}] {}", self.code(), self)
    }
}

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

    #[test]
    fn role_runtime_not_ready_code() {
        assert_eq!(AppError::RoleRuntimeNotReady.code(), "ROLE_RUNTIME_NOT_READY");
    }

    #[test]
    fn startup_health_failed_code() {
        let e = AppError::StartupHealthFailed("db ping".into());
        assert_eq!(e.code(), "STARTUP_HEALTH_FAILED");
        assert!(e.to_frontend_error().contains("STARTUP_HEALTH_FAILED"));
    }
}
