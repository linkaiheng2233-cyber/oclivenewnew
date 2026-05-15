use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 无头内核与宿主共用的 **JSON 错误体**（Tauri `invoke` 失败字符串、HTTP `error` 对象同源字段）。
///
/// - `code`：与 [`AppError::code`] 一致的机器码（`SCREAMING_SNAKE_CASE`），供壳层 i18n 与黑盒断言。
/// - `message`：[`AppError`] 的 `Display` 文本（默认英文技术句）；本地化由发行版用 `code` 映射。
/// - `hint`：可选「下一步」；HTTP 路由可为试聊等场景附加，内核默认 `None`。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KernelErrorBody {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// `POST /chat`（及同类 HTTP 边界）专有 **`code`**：无对应 [`AppError`] 变体，但命名规则与 [`AppError::code`] 相同（`SCREAMING_SNAKE_CASE`）。
///
/// 宿主在构造 [`KernelErrorBody`] 时应引用本模块常量，避免字面量漂移。
pub mod http_chat_codes {
    pub const EMPTY_MESSAGE: &str = "EMPTY_MESSAGE";
    pub const INVALID_ROLE_PATH: &str = "INVALID_ROLE_PATH";
    pub const LOAD_ROLE_TASK_PANIC: &str = "LOAD_ROLE_TASK_PANIC";
}

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

    /// 构造与 HTTP `error` 对象字段一致的 JSON 错误体（不含外层 `{ "error": … }`）。
    #[must_use]
    pub fn kernel_error_body(&self) -> KernelErrorBody {
        KernelErrorBody {
            code: self.code().to_string(),
            message: self.to_string(),
            hint: None,
        }
    }

    /// JSON 单行字符串，供 Tauri `invoke` 失败载荷与日志使用（与 HTTP 内层 `error` 同形）。
    #[must_use]
    pub fn to_kernel_json(&self) -> String {
        serde_json::to_string(&self.kernel_error_body()).unwrap_or_else(|_| {
            serde_json::to_string(&KernelErrorBody {
                code: "UNKNOWN_ERROR".into(),
                message: self.to_string(),
                hint: None,
            })
            .unwrap_or_else(|_| {
                "{\"code\":\"UNKNOWN_ERROR\",\"message\":\"serialization failed\",\"hint\":null}"
                    .into()
            })
        })
    }

    /// 与 [`Self::to_kernel_json`] 相同（历史命名：「前端」泛指任意宿主壳）。
    #[must_use]
    pub fn to_frontend_error(&self) -> String {
        self.to_kernel_json()
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
        let j: KernelErrorBody = serde_json::from_str(&e.to_kernel_json()).expect("json");
        assert_eq!(j.code, "STARTUP_HEALTH_FAILED");
        assert!(j.message.contains("db ping"));
    }

    #[test]
    fn to_kernel_json_roundtrip() {
        let e = AppError::RoleNotFound("x".into());
        let s = e.to_kernel_json();
        let j: KernelErrorBody = serde_json::from_str(&s).unwrap();
        assert_eq!(j.code, "ROLE_NOT_FOUND");
        assert!(j.message.contains('x'));
    }

    #[test]
    fn http_chat_codes_are_screaming_snake() {
        for c in [
            http_chat_codes::EMPTY_MESSAGE,
            http_chat_codes::INVALID_ROLE_PATH,
            http_chat_codes::LOAD_ROLE_TASK_PANIC,
        ] {
            assert!(
                c.bytes().all(|b| b.is_ascii_uppercase() || b == b'_'),
                "{c} must be SCREAMING_SNAKE_CASE"
            );
        }
    }
}
