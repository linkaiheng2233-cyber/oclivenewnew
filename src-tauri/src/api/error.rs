//! 目录插件等 Tauri 命令的统一错误码；载荷与内核同源：**单行 `KernelErrorBody` JSON**（与 `AppError::to_kernel_json` 一致）。

use oclive_kernel_runtime::KernelErrorBody;
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
    /// 与内核 [`oclive_kernel_runtime::AppError::HighRiskCapabilityNotGranted`] 同码，供目录插件纯文本错误映射。
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
        KernelErrorBody {
            code: self.code().to_string(),
            message: self.kernel_message(),
            hint: None,
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

/// Tauri command error newtype (orphan-safe bridge to [`tauri::InvokeError`]).
#[derive(Debug)]
pub struct CommandError(pub crate::error::AppError);

impl From<crate::error::AppError> for CommandError {
    fn from(e: crate::error::AppError) -> Self {
        Self(e)
    }
}

impl From<CommandError> for tauri::InvokeError {
    fn from(e: CommandError) -> Self {
        tauri::InvokeError::from(e.0.to_kernel_json())
    }
}

impl From<serde_json::Error> for CommandError {
    fn from(e: serde_json::Error) -> Self {
        Self(crate::error::AppError::SerializationError(e))
    }
}

/// 将 `DirectoryPluginRuntime::ensure_rpc_url` 等返回的纯文本失败映射为 **`KernelErrorBody` JSON 单行**。
#[must_use]
pub fn map_directory_rpc_url_error(plugin_id: &str, err: String) -> String {
    let id = plugin_id.trim().to_string();
    if err.contains("unknown directory plugin_id=") {
        return ApiError::PluginNotFound { plugin_id: id }.to_kernel_json();
    }
    if err.starts_with("plugin disabled:") {
        return ApiError::PermissionDenied { message: err }.to_kernel_json();
    }
    if err.contains("directory plugin spawn not granted") {
        return ApiError::HighRiskCapabilityNotGranted { message: err }.to_kernel_json();
    }
    if err.contains(" has no process section") {
        return ApiError::InvalidManifest { message: err }.to_kernel_json();
    }
    if err.contains("manifest.json")
        || err.contains("unsupported schema_version")
        || err.contains(": id empty")
        || err.contains(": version empty")
        || err.contains("shell.entry required")
    {
        return ApiError::InvalidManifest { message: err }.to_kernel_json();
    }
    ApiError::Io { message: err }.to_kernel_json()
}

#[cfg(test)]
mod tests {
    use super::map_directory_rpc_url_error;
    use oclive_kernel_runtime::KernelErrorBody;

    #[test]
    fn map_rpc_spawn_not_granted_uses_high_risk_code() {
        let s = map_directory_rpc_url_error(
            "my_plug",
            "directory plugin spawn not granted: plugin_id=my_plug".into(),
        );
        let j: KernelErrorBody = serde_json::from_str(&s).expect("json");
        assert_eq!(j.code, "HIGH_RISK_CAPABILITY_NOT_GRANTED");
        assert!(j.message.contains("my_plug"));
    }

    #[test]
    fn map_rpc_unknown_plugin_is_kernel_json() {
        let s =
            map_directory_rpc_url_error("my_plug", "unknown directory plugin_id=my_plug".into());
        let j: KernelErrorBody = serde_json::from_str(&s).expect("json");
        assert_eq!(j.code, "API_PLUGIN_NOT_FOUND");
        assert!(j.message.contains("my_plug"));
    }

    #[test]
    fn map_rpc_disabled_is_kernel_json() {
        let s = map_directory_rpc_url_error("x", "plugin disabled: x".into());
        let j: KernelErrorBody = serde_json::from_str(&s).expect("json");
        assert_eq!(j.code, "API_PERMISSION_DENIED");
    }
}
