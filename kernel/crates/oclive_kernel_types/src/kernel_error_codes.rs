//! Static kernel error codes (SSOT for docs, frontend enum, and dimension5 drift gate).
//!
//! Derived from [`crate::error::AppError::code`] samples + [`crate::error::http_chat_codes`].

use crate::error::{http_chat_codes, AppError};

/// Sample every [`AppError`] variant to collect its stable [`AppError::code`] string.
#[must_use]
pub fn app_error_codes_sampled() -> Vec<&'static str> {
    vec![
        AppError::DatabaseError(String::new()).code(),
        AppError::DbMigrationFailed(String::new()).code(),
        AppError::PluginManifestInvalid(String::new()).code(),
        AppError::IoError(std::io::Error::other("sample")).code(),
        AppError::OllamaError(String::new()).code(),
        AppError::RoleNotFound(String::new()).code(),
        AppError::RoleRuntimeNotReady.code(),
        AppError::StartupHealthFailed(String::new()).code(),
        AppError::RolePackExists(String::new()).code(),
        AppError::InvalidParameter(String::new()).code(),
        AppError::EmptyMessage.code(),
        AppError::HighRiskCapabilityNotGranted {
            capability: String::new(),
            id: String::new(),
        }
        .code(),
        AppError::RemoteServiceUnavailable(String::new()).code(),
        AppError::SerializationError(
            serde_json::from_str::<serde_json::Value>("not json").unwrap_err(),
        )
        .code(),
        AppError::KernelOffline.code(),
        AppError::KernelAuthRequired(String::new()).code(),
        AppError::Unknown(String::new()).code(),
    ]
}

/// HTTP `/chat` supplement codes without a dedicated [`AppError`] variant.
#[must_use]
pub const fn http_supplement_codes() -> [&'static str; 4] {
    [
        http_chat_codes::EMPTY_MESSAGE,
        http_chat_codes::INVALID_ROLE_PATH,
        http_chat_codes::LOAD_ROLE_TASK_PANIC,
        http_chat_codes::THEATER_SCENE_GEN_FAILED,
    ]
}

/// Sorted, deduplicated static codes documented in `creator-docs/getting-started/ERROR_CODES.md`.
#[must_use]
pub fn all_documented_kernel_codes() -> Vec<&'static str> {
    let mut codes = app_error_codes_sampled();
    codes.extend_from_slice(&http_supplement_codes());
    codes.sort_unstable();
    codes.dedup();
    codes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampled_codes_are_screaming_snake() {
        for code in all_documented_kernel_codes() {
            assert!(
                code.bytes().all(|b| b.is_ascii_uppercase() || b == b'_'),
                "{code} must be SCREAMING_SNAKE_CASE"
            );
        }
    }

    #[test]
    fn empty_message_not_duplicated_between_app_and_http() {
        let codes = all_documented_kernel_codes();
        assert_eq!(codes.iter().filter(|c| **c == "EMPTY_MESSAGE").count(), 1);
    }

    #[test]
    fn export_kernel_error_codes_json() {
        let codes: Vec<String> = all_documented_kernel_codes()
            .into_iter()
            .map(str::to_string)
            .collect();
        println!(
            "KERNEL_ERROR_CODES_JSON:{}",
            serde_json::to_string(&codes).expect("json")
        );
    }
}
