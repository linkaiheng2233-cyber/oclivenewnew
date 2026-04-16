//! 目录插件等 Tauri 命令的统一错误码（`Display` 含 `[CODE]` 供前端 `parseBackendError` 解析）。

use std::fmt;

#[derive(Debug, Clone)]
pub enum ApiError {
    PluginNotFound { plugin_id: String },
    InvalidParameter { message: String },
    PermissionDenied { message: String },
    InvalidManifest { message: String },
    Io { message: String },
}

impl ApiError {
    pub fn code(&self) -> &'static str {
        match self {
            ApiError::PluginNotFound { .. } => "API_PLUGIN_NOT_FOUND",
            ApiError::InvalidParameter { .. } => "INVALID_PARAMETER",
            ApiError::PermissionDenied { .. } => "API_PERMISSION_DENIED",
            ApiError::InvalidManifest { .. } => "API_INVALID_MANIFEST",
            ApiError::Io { .. } => "IO_ERROR",
        }
    }

    fn body(&self) -> String {
        match self {
            ApiError::PluginNotFound { plugin_id } => {
                format!("unknown plugin_id={}", plugin_id.trim())
            }
            ApiError::InvalidParameter { message }
            | ApiError::PermissionDenied { message }
            | ApiError::InvalidManifest { message }
            | ApiError::Io { message } => message.clone(),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code(), self.body())
    }
}

impl From<ApiError> for String {
    fn from(e: ApiError) -> String {
        e.to_string()
    }
}

impl From<ApiError> for tauri::InvokeError {
    fn from(e: ApiError) -> Self {
        Self::from(String::from(e))
    }
}

/// 将 `DirectoryPluginRuntime::ensure_rpc_url` 等返回的纯文本失败映射为带 `[CODE]` 的字符串，供前端解析。
pub fn map_directory_rpc_url_error(plugin_id: &str, err: String) -> String {
    let id = plugin_id.trim().to_string();
    if err.contains("unknown directory plugin_id=") {
        return ApiError::PluginNotFound { plugin_id: id }.to_string();
    }
    if err.starts_with("plugin disabled:") {
        return ApiError::PermissionDenied { message: err }.to_string();
    }
    if err.contains(" has no process section") {
        return ApiError::InvalidManifest { message: err }.to_string();
    }
    if err.contains("manifest.json")
        || err.contains("unsupported schema_version")
        || err.contains(": id empty")
        || err.contains(": version empty")
        || err.contains("shell.entry required")
    {
        return ApiError::InvalidManifest { message: err }.to_string();
    }
    ApiError::Io { message: err }.to_string()
}

#[cfg(test)]
mod tests {
    use super::map_directory_rpc_url_error;

    #[test]
    fn map_rpc_unknown_plugin() {
        let s =
            map_directory_rpc_url_error("my_plug", "unknown directory plugin_id=my_plug".into());
        assert!(s.starts_with("[API_PLUGIN_NOT_FOUND]"));
        assert!(s.contains("my_plug"));
    }

    #[test]
    fn map_rpc_disabled() {
        let s = map_directory_rpc_url_error("x", "plugin disabled: x".into());
        assert!(s.starts_with("[API_PERMISSION_DENIED]"));
    }
}
