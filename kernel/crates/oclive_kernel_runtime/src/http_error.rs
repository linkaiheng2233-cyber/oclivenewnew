//! Map kernel HTTP error JSON to host [`AppError`] (SSOT for desktop attach + tools).

use crate::error::http_chat_codes;
use crate::KernelErrorBody;
use oclive_kernel_types::AppError;

#[derive(serde::Deserialize)]
struct ApiErrorEnvelope {
    error: KernelErrorBody,
}

/// Map kernel HTTP `{"error": KernelErrorBody}` (or bare body) to [`AppError`].
#[must_use]
pub fn app_error_from_http_response(status: u16, text: &str) -> AppError {
    let trimmed = text.trim();
    if let Ok(env) = serde_json::from_str::<ApiErrorEnvelope>(trimmed) {
        return app_error_from_kernel_body(&env.error);
    }
    if let Ok(body) = serde_json::from_str::<KernelErrorBody>(trimmed) {
        return app_error_from_kernel_body(&body);
    }
    if status == 401 {
        // Plain-text (or unmapped) 401 from the auth middleware: a stale kernel with a
        // mismatched token, never an LLM/back-end failure. The host rebuilds the kernel.
        return AppError::KernelAuthRequired(trimmed.to_string());
    }
    if status == 503 {
        return AppError::KernelOffline;
    }
    AppError::OllamaError(format!("HTTP {status}: {trimmed}"))
}

/// Map [`KernelErrorBody`] machine code to [`AppError`].
#[must_use]
pub fn app_error_from_kernel_body(body: &KernelErrorBody) -> AppError {
    match body.code.as_str() {
        http_chat_codes::EMPTY_MESSAGE => AppError::EmptyMessage,
        http_chat_codes::INVALID_ROLE_PATH => AppError::InvalidParameter(body.message.clone()),
        http_chat_codes::LOAD_ROLE_TASK_PANIC => AppError::Unknown(body.message.clone()),
        "ROLE_RUNTIME_NOT_READY" => AppError::RoleRuntimeNotReady,
        "KERNEL_OFFLINE" => AppError::KernelOffline,
        "ROLE_NOT_FOUND" => AppError::RoleNotFound(body.message.clone()),
        "INVALID_PARAMETER" => AppError::InvalidParameter(body.message.clone()),
        "DB_ERROR" => AppError::DatabaseError(body.message.clone()),
        "LLM_ERROR" => AppError::OllamaError(body.message.clone()),
        "KERNEL_AUTH_REQUIRED" => AppError::KernelAuthRequired(body.message.clone()),
        "STARTUP_HEALTH_FAILED" => AppError::StartupHealthFailed(body.message.clone()),
        "IO_ERROR" => AppError::Unknown(format!("IO_ERROR: {}", body.message)),
        "SERDE_ERROR" => AppError::Unknown(format!("SERDE_ERROR: {}", body.message)),
        "HIGH_RISK_CAPABILITY_NOT_GRANTED" => AppError::HighRiskCapabilityNotGranted {
            capability: body.message.clone(),
            id: String::new(),
        },
        "REMOTE_SERVICE_UNAVAILABLE" => AppError::RemoteServiceUnavailable(body.message.clone()),
        "ROLE_PACK_EXISTS" => AppError::RolePackExists(body.message.clone()),
        code if code.starts_with("API_") => AppError::Unknown(format!("{code}: {}", body.message)),
        _ => AppError::Unknown(body.message.clone()),
    }
}
