//! Unified error codes for directory-plugin and other Tauri commands; payload matches the kernel: **single-line `KernelErrorBody` JSON** (same as `AppError::to_kernel_json`).

use oclive_kernel_types::KernelErrorBody;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ApiError {
    PluginNotFound {
        plugin_id: String,
    },
    InvalidParameter {
        message: String,
    },
    PermissionDenied {
        message: String,
    },
    /// Same code as kernel [`oclive_kernel_types::AppError::HighRiskCapabilityNotGranted`]; used to map plain-text directory-plugin errors.
    HighRiskCapabilityNotGranted {
        message: String,
    },
    InvalidManifest {
        message: String,
    },
    Io {
        message: String,
    },
}

impl ApiError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            ApiError::PluginNotFound { .. } => "API_PLUGIN_NOT_FOUND",
            ApiError::InvalidParameter { .. } => "INVALID_PARAMETER",
            ApiError::PermissionDenied { .. } => "API_PERMISSION_DENIED",
            ApiError::HighRiskCapabilityNotGranted { .. } => "HIGH_RISK_CAPABILITY_NOT_GRANTED",
            ApiError::InvalidManifest { .. } => "API_INVALID_MANIFEST",
            ApiError::Io { .. } => "IO_ERROR",
        }
    }

    fn kernel_message(&self) -> String {
        match self {
            ApiError::PluginNotFound { plugin_id } => {
                format!("unknown plugin_id={}", plugin_id.trim())
            }
            ApiError::InvalidParameter { message }
            | ApiError::PermissionDenied { message }
            | ApiError::HighRiskCapabilityNotGranted { message }
            | ApiError::InvalidManifest { message }
            | ApiError::Io { message } => message.clone(),
        }
    }

    #[must_use]
    pub fn kernel_error_body(&self) -> KernelErrorBody {
        let message = self.kernel_message();
        let context = match self {
            Self::InvalidParameter { message: m } if m.contains("plugin_backends:") => {
                Some(serde_json::json!({ "kind": "plugin_backends_directory_slot" }))
            }
            Self::Io { message: m } if m.contains("host json") => {
                Some(serde_json::json!({ "kind": "host_json" }))
            }
            _ => None,
        };
        KernelErrorBody {
            code: self.code().to_string(),
            message,
            hint: None,
            context,
        }
    }

    #[must_use]
    pub fn to_kernel_json(&self) -> String {
        serde_json::to_string(&self.kernel_error_body()).unwrap_or_else(|_| {
            "{\"code\":\"UNKNOWN_ERROR\",\"message\":\"api error serialization failed\",\"hint\":null}"
                .to_string()
        })
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code(), self.kernel_message())
    }
}

impl From<ApiError> for String {
    fn from(e: ApiError) -> String {
        e.to_kernel_json()
    }
}

/// Tauri command error bridge (orphan-safe mapping to [`tauri::InvokeError`]).
#[derive(Debug)]
pub enum CommandError {
    App(crate::error::AppError),
    Api(ApiError),
}

impl CommandError {
    #[must_use]
    pub fn kernel_error_body(&self) -> KernelErrorBody {
        match self {
            Self::App(e) => e.kernel_error_body(),
            Self::Api(e) => e.kernel_error_body(),
        }
    }

    #[must_use]
    pub fn to_kernel_json(&self) -> String {
        match self {
            Self::App(e) => e.to_kernel_json(),
            Self::Api(e) => e.to_kernel_json(),
        }
    }
}

impl From<crate::error::AppError> for CommandError {
    fn from(e: crate::error::AppError) -> Self {
        Self::App(e)
    }
}

impl From<ApiError> for CommandError {
    fn from(e: ApiError) -> Self {
        Self::Api(e)
    }
}

#[cfg(feature = "tauri-commands")]
impl From<CommandError> for tauri::InvokeError {
    fn from(e: CommandError) -> Self {
        tauri::InvokeError::from(e.to_kernel_json())
    }
}

impl From<serde_json::Error> for CommandError {
    fn from(e: serde_json::Error) -> Self {
        Self::App(crate::error::AppError::SerializationError(e))
    }
}

impl From<String> for CommandError {
    fn from(s: String) -> Self {
        let trimmed = s.trim();
        if trimmed.starts_with('{') {
            if let Ok(body) = serde_json::from_str::<KernelErrorBody>(trimmed) {
                if body.code == "HIGH_RISK_CAPABILITY_NOT_GRANTED" {
                    return Self::Api(ApiError::HighRiskCapabilityNotGranted {
                        message: body.message,
                    });
                }
                if body.code == "API_PLUGIN_NOT_FOUND" {
                    return Self::Api(ApiError::PluginNotFound {
                        plugin_id: body.message,
                    });
                }
                if body.code == "API_PERMISSION_DENIED" {
                    return Self::Api(ApiError::PermissionDenied {
                        message: body.message,
                    });
                }
                if body.code == "API_INVALID_MANIFEST" {
                    return Self::Api(ApiError::InvalidManifest {
                        message: body.message,
                    });
                }
                if body.code == "IO_ERROR" {
                    return Self::Api(ApiError::Io {
                        message: body.message,
                    });
                }
                if body.code == "INVALID_PARAMETER" {
                    return Self::Api(ApiError::InvalidParameter {
                        message: body.message,
                    });
                }
            }
        }
        Self::App(crate::error::AppError::Unknown(s))
    }
}

impl From<std::io::Error> for CommandError {
    fn from(e: std::io::Error) -> Self {
        Self::App(crate::error::AppError::from(e))
    }
}

/// Bridge for API helpers not yet migrated off `Result<_, String>`.
impl From<CommandError> for String {
    fn from(e: CommandError) -> Self {
        match e {
            CommandError::App(a) => a.to_frontend_error(),
            CommandError::Api(api) => api.to_string(),
        }
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::App(a) => write!(f, "{}", a.to_frontend_error()),
            Self::Api(api) => write!(f, "{api}"),
        }
    }
}

/// Maps plain-text failures from `DirectoryPluginRuntime::ensure_rpc_url` and similar to [`ApiError`].
#[must_use]
pub fn map_directory_rpc_url_error(plugin_id: &str, err: String) -> ApiError {
    let id = plugin_id.trim().to_string();
    if err.contains("unknown directory plugin_id=") {
        return ApiError::PluginNotFound { plugin_id: id };
    }
    if err.starts_with("plugin disabled:") {
        return ApiError::PermissionDenied { message: err };
    }
    if err.contains("directory plugin spawn not granted") {
        return ApiError::HighRiskCapabilityNotGranted { message: err };
    }
    if err.contains("directory plugin spawn not permitted") {
        return ApiError::HighRiskCapabilityNotGranted { message: err };
    }
    if err.contains(" has no process section") {
        return ApiError::InvalidManifest { message: err };
    }
    if err.contains("manifest.json")
        || err.contains("unsupported schema_version")
        || err.contains(": id empty")
        || err.contains(": version empty")
        || err.contains("shell.entry required")
    {
        return ApiError::InvalidManifest { message: err };
    }
    ApiError::Io { message: err }
}

#[cfg(test)]
mod tests {
    use super::{map_directory_rpc_url_error, ApiError, CommandError};
    use oclive_kernel_types::KernelErrorBody;

    fn invoke_code(err: CommandError) -> String {
        let j: KernelErrorBody = serde_json::from_str(&err.to_kernel_json()).expect("kernel json");
        j.code
    }

    #[test]
    fn api_error_variants_preserve_code_through_invoke() {
        let cases: Vec<(&str, ApiError)> = vec![
            (
                "API_PLUGIN_NOT_FOUND",
                ApiError::PluginNotFound {
                    plugin_id: "p1".into(),
                },
            ),
            (
                "INVALID_PARAMETER",
                ApiError::InvalidParameter {
                    message: "bad".into(),
                },
            ),
            (
                "API_PERMISSION_DENIED",
                ApiError::PermissionDenied {
                    message: "denied".into(),
                },
            ),
            (
                "HIGH_RISK_CAPABILITY_NOT_GRANTED",
                ApiError::HighRiskCapabilityNotGranted {
                    message: "spawn".into(),
                },
            ),
            (
                "API_INVALID_MANIFEST",
                ApiError::InvalidManifest {
                    message: "manifest".into(),
                },
            ),
            (
                "IO_ERROR",
                ApiError::Io {
                    message: "disk".into(),
                },
            ),
        ];
        for (expected_code, api_err) in cases {
            assert_eq!(invoke_code(CommandError::from(api_err)), expected_code);
        }
    }

    #[test]
    fn map_rpc_spawn_not_permitted_uses_high_risk_code() {
        let api = map_directory_rpc_url_error(
            "my_plug",
            "directory plugin spawn not permitted: plugin_id=my_plug missing process:spawn in manifest permissions"
                .into(),
        );
        assert_eq!(api.code(), "HIGH_RISK_CAPABILITY_NOT_GRANTED");
        assert_eq!(invoke_code(api.into()), "HIGH_RISK_CAPABILITY_NOT_GRANTED");
    }

    #[test]
    fn map_rpc_spawn_not_granted_uses_high_risk_code() {
        let api = map_directory_rpc_url_error(
            "my_plug",
            "directory plugin spawn not granted: plugin_id=my_plug".into(),
        );
        assert_eq!(api.code(), "HIGH_RISK_CAPABILITY_NOT_GRANTED");
        assert_eq!(invoke_code(api.into()), "HIGH_RISK_CAPABILITY_NOT_GRANTED");
    }

    #[test]
    fn map_rpc_unknown_plugin_is_kernel_json() {
        let api =
            map_directory_rpc_url_error("my_plug", "unknown directory plugin_id=my_plug".into());
        let j: KernelErrorBody = serde_json::from_str(&api.to_kernel_json()).expect("json");
        assert_eq!(j.code, "API_PLUGIN_NOT_FOUND");
        assert!(j.message.contains("my_plug"));
    }

    #[test]
    fn map_rpc_disabled_is_kernel_json() {
        let api = map_directory_rpc_url_error("x", "plugin disabled: x".into());
        let j: KernelErrorBody = serde_json::from_str(&api.to_kernel_json()).expect("json");
        assert_eq!(j.code, "API_PERMISSION_DENIED");
    }
}
