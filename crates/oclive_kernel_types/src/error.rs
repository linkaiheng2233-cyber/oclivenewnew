use serde::{Deserialize, Serialize};
use thiserror::Error;

/// **JSON error body** shared by the headless kernel and hosts (the Tauri `invoke` failure string and the HTTP `error` object share these fields).
///
/// - `code`: machine code consistent with [`AppError::code`] (`SCREAMING_SNAKE_CASE`), for shell-layer i18n and black-box assertions.
/// - `message`: the `Display` text of [`AppError`] (technical English by default); localization is mapped by the distribution via `code`.
/// - `hint`: optional "next step"; HTTP routes may attach it for cases such as trial chat, while the kernel defaults to `None`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KernelErrorBody {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// **`code`** values specific to `POST /chat` (and similar HTTP boundaries): they have no corresponding [`AppError`] variant, but follow the same naming rule as [`AppError::code`] (`SCREAMING_SNAKE_CASE`).
///
/// Hosts should reference the constants in this module when constructing a [`KernelErrorBody`] to avoid literal drift.
pub mod http_chat_codes {
    pub const EMPTY_MESSAGE: &str = "EMPTY_MESSAGE";
    pub const INVALID_ROLE_PATH: &str = "INVALID_ROLE_PATH";
    pub const LOAD_ROLE_TASK_PANIC: &str = "LOAD_ROLE_TASK_PANIC";
    pub const THEATER_SCENE_GEN_FAILED: &str = "THEATER_SCENE_GEN_FAILED";
}

/// Unified kernel error type mapped to [`KernelErrorBody`] and machine `code` strings.
///
/// The frontend should prefer mapping i18n via [`Self::code`] (`apiErrors` / `UNKNOWN_WITH_CODE`) rather than parsing the English `message`.
#[derive(Error, Debug)]
pub enum AppError {
    /// **When**: SQLx / migration / transaction failure. **Show**: retry or contact support. **User**: usually no config change needed.
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// **When**: failure reading/writing role packs, logs, or grant files. **Show**: check the path and disk permissions.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// **When**: an Ollama or directory LLM plugin call fails. **Show**: check whether the model is pulled and the service is running.
    #[error("Ollama error: {0}")]
    OllamaError(String),

    /// **When**: the `role_id` does not exist or has not been imported. **Show**: guide the user to select/import a role pack.
    #[error("Role not found: {0}")]
    RoleNotFound(String),

    /// **When**: `load_role` has not run yet, or the `role_runtime` row is missing. **Show**: prompt to load the role first or restart the session.
    #[error("Role runtime not initialized; call load_role first")]
    RoleRuntimeNotReady,

    /// **When**: the host's `startup_health` first-turn check fails (slots / DB / optional LLM). **Show**: the environment self-check on the settings page.
    #[error("Startup health failed: {0}")]
    StartupHealthFailed(String),

    /// **When**: the role-pack import target already exists and `overwrite` is not set. **Show**: confirm overwrite or choose another directory.
    #[error("Role already exists; overwrite required: {0}")]
    RolePackExists(String),

    /// **When**: validation of request params, blueprint, scene id, etc. fails. **Show**: a specific message; the creator fixes the pack and retries.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// **When**: chat message is empty or whitespace-only. **Show**: prompt user to enter visible characters.
    #[error("Message must not be empty or whitespace-only")]
    EmptyMessage,

    /// **When**: MCP / directory `process:spawn` / `network:*` is not granted. **Show**: the plugin-management permission dialog; **User**: must grant explicitly.
    #[error("High-risk capability not granted: {capability} (id={id})")]
    HighRiskCapabilityNotGranted { capability: String, id: String },

    /// **When**: the Remote backend is unavailable and auto-fallback is disabled. **Show**: network / URL / plugin logs; can switch back to builtin.
    #[error("Remote service unavailable: {0}")]
    RemoteServiceUnavailable(String),

    /// **When**: JSON/YAML parsing fails. **Show**: check the config or role-pack format.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// **When**: the shared kernel HTTP API is unreachable (desktop / VS Code attach mode).
    #[error("Kernel is offline")]
    KernelOffline,

    /// **When**: an unclassified internal error. **Show**: report with `code`; avoid exposing the stack.
    #[error("Unknown error: {0}")]
    Unknown(String),

    /// **When**: a multi-table atomic write fails (carries a stable `code`). **Show**: map by `code`; sending the message can be retried.
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
            AppError::EmptyMessage => http_chat_codes::EMPTY_MESSAGE,
            AppError::HighRiskCapabilityNotGranted { .. } => "HIGH_RISK_CAPABILITY_NOT_GRANTED",
            AppError::RemoteServiceUnavailable(_) => "REMOTE_SERVICE_UNAVAILABLE",
            AppError::SerializationError(_) => "SERDE_ERROR",
            AppError::KernelOffline => "KERNEL_OFFLINE",
            AppError::Unknown(_) => "UNKNOWN_ERROR",
            AppError::TransactionError { code, .. } => code,
        }
    }

    /// Build a JSON error body whose fields match the HTTP `error` object (without the outer `{ "error": … }` wrapper).
    #[must_use]
    pub fn kernel_error_body(&self) -> KernelErrorBody {
        KernelErrorBody {
            code: self.code().to_string(),
            message: self.to_string(),
            hint: None,
        }
    }

    /// Single-line JSON string for Tauri `invoke` failure payloads and logs (same shape as the HTTP inner `error`).
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

    /// Same as [`Self::to_kernel_json`] (legacy name: "frontend" here refers to any host shell).
    #[must_use]
    pub fn to_frontend_error(&self) -> String {
        self.to_kernel_json()
    }

    /// Prefix error message with `send_message[{stage}]` while preserving machine `code`.
    #[must_use]
    pub fn with_chat_stage(self, stage: &'static str) -> Self {
        match self {
            Self::DatabaseError(m) => Self::DatabaseError(format!("send_message[{stage}]: {m}")),
            Self::IoError(e) => Self::IoError(e),
            Self::OllamaError(m) => Self::OllamaError(format!("send_message[{stage}]: {m}")),
            Self::RoleNotFound(m) => Self::RoleNotFound(format!("send_message[{stage}]: {m}")),
            Self::RoleRuntimeNotReady => Self::RoleRuntimeNotReady,
            Self::StartupHealthFailed(m) => {
                Self::StartupHealthFailed(format!("send_message[{stage}]: {m}"))
            }
            Self::RolePackExists(m) => Self::RolePackExists(format!("send_message[{stage}]: {m}")),
            Self::InvalidParameter(m) => Self::InvalidParameter(format!("send_message[{stage}]: {m}")),
            Self::EmptyMessage => Self::EmptyMessage,
            Self::HighRiskCapabilityNotGranted { capability, id } => {
                Self::HighRiskCapabilityNotGranted {
                    capability: format!("send_message[{stage}]: {capability}"),
                    id,
                }
            }
            Self::RemoteServiceUnavailable(m) => {
                Self::RemoteServiceUnavailable(format!("send_message[{stage}]: {m}"))
            }
            Self::SerializationError(e) => Self::SerializationError(e),
            Self::KernelOffline => Self::KernelOffline,
            Self::Unknown(m) => Self::Unknown(format!("send_message[{stage}]: {m}")),
            Self::TransactionError { code, message } => Self::TransactionError {
                code,
                message: format!("send_message[{stage}]: {message}"),
            },
        }
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
        assert_eq!(
            AppError::RoleRuntimeNotReady.code(),
            "ROLE_RUNTIME_NOT_READY"
        );
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
    fn high_risk_capability_not_granted_code() {
        let e = AppError::HighRiskCapabilityNotGranted {
            capability: "mcp_http".into(),
            id: "weather".into(),
        };
        assert_eq!(e.code(), "HIGH_RISK_CAPABILITY_NOT_GRANTED");
        let j: KernelErrorBody = serde_json::from_str(&e.to_kernel_json()).expect("json");
        assert_eq!(j.code, "HIGH_RISK_CAPABILITY_NOT_GRANTED");
        assert!(j.message.contains("weather"));
    }

    #[test]
    fn remote_service_unavailable_code() {
        let e = AppError::RemoteServiceUnavailable("emotion.analyze timeout".into());
        assert_eq!(e.code(), "REMOTE_SERVICE_UNAVAILABLE");
        let j: KernelErrorBody = serde_json::from_str(&e.to_kernel_json()).expect("json");
        assert_eq!(j.code, "REMOTE_SERVICE_UNAVAILABLE");
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
            http_chat_codes::THEATER_SCENE_GEN_FAILED,
        ] {
            assert!(
                c.bytes().all(|b| b.is_ascii_uppercase() || b == b'_'),
                "{c} must be SCREAMING_SNAKE_CASE"
            );
        }
    }
}
