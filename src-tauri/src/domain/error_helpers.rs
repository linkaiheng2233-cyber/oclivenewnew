//! 领域层错误映射辅助：集中 `AppError` 构造与 `map_err` 模式。

use crate::error::AppError;

/// `serde_json` 编码/解码失败 → [`AppError::Unknown`]。
#[must_use]
pub fn serde_to_unknown(context: &str, e: serde_json::Error) -> AppError {
    AppError::Unknown(format!("{context}: {e}"))
}

/// `serde_json` 失败 → [`AppError::OllamaError`]（Remote LLM / 侧车 wire）。
#[must_use]
pub fn serde_to_ollama(context: &str, e: serde_json::Error) -> AppError {
    AppError::OllamaError(format!("{context}: {e}"))
}

/// 带上下文的 LLM/Remote 侧车错误文案。
#[must_use]
pub fn ollama_msg(context: &str, detail: impl std::fmt::Display) -> AppError {
    AppError::OllamaError(format!("{context}: {detail}"))
}

/// `Result` → [`AppError::OllamaError`]。
///
/// # Errors
///
/// Maps the inner error with [`ollama_msg`].
pub fn map_to_ollama<T, E: std::fmt::Display>(
    context: &str,
    r: std::result::Result<T, E>,
) -> std::result::Result<T, AppError> {
    r.map_err(|e| ollama_msg(context, e))
}

#[macro_export]
macro_rules! map_err_ollama {
    ($ctx:expr, $expr:expr) => {
        $expr.map_err(|e| $crate::domain::error_helpers::ollama_msg($ctx, e))
    };
}

#[macro_export]
macro_rules! map_err_unknown {
    ($ctx:expr, $expr:expr) => {
        $expr.map_err(|e| $crate::domain::error_helpers::serde_to_unknown($ctx, e))
    };
}
