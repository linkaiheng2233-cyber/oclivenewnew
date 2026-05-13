//! 与 `src-tauri/src/api/error.rs` 中 `ApiError` 的 `Display` 一致，供 Module 9 等命令在无 Tauri 宿主下返回相同 `[CODE]` 前缀。

use std::fmt;

#[derive(Debug, Clone)]
pub enum BridgeApiError {
    InvalidParameter { message: String },
    Io { message: String },
}

impl BridgeApiError {
    pub fn code(&self) -> &'static str {
        match self {
            BridgeApiError::InvalidParameter { .. } => "INVALID_PARAMETER",
            BridgeApiError::Io { .. } => "IO_ERROR",
        }
    }

    fn body(&self) -> String {
        match self {
            BridgeApiError::InvalidParameter { message } | BridgeApiError::Io { message } => {
                message.clone()
            }
        }
    }
}

impl fmt::Display for BridgeApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code(), self.body())
    }
}

impl From<BridgeApiError> for String {
    fn from(e: BridgeApiError) -> String {
        e.to_string()
    }
}
