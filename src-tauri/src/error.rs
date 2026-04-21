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

    #[error("角色已存在，需确认是否覆盖：{0}")]
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

impl From<AppError> for tauri::ipc::InvokeError {
    fn from(err: AppError) -> Self {
        tauri::ipc::InvokeError::from(err.to_string())
    }
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::DatabaseError(_) => "DB_ERROR",
            AppError::IoError(_) => "IO_ERROR",
            AppError::OllamaError(_) => "LLM_ERROR",
            AppError::RoleNotFound(_) => "ROLE_NOT_FOUND",
            AppError::RolePackExists(_) => "ROLE_PACK_EXISTS",
            AppError::InvalidParameter(_) => "INVALID_PARAMETER",
            AppError::SerializationError(_) => "SERDE_ERROR",
            AppError::Unknown(_) => "UNKNOWN_ERROR",
            AppError::TransactionError { code, .. } => code,
        }
    }

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
}
