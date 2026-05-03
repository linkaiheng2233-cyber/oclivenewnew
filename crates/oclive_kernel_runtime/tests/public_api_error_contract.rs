//! 对外错误契约集成测试（P0）：与 `handoff/10_ERROR_CODE_DICTIONARY.md` §Common 对齐。
//! 纳入 `cargo test -p oclive_kernel_runtime` / `cargo test --workspace`。

use oclive_kernel_runtime::error::AppError;

fn assert_bracket_code(err: AppError, expected_code: &str) {
    let s = err.to_frontend_error();
    assert!(
        s.starts_with(&format!("[{}]", expected_code)),
        "expected prefix [{}] for {:?}, got {:?}",
        expected_code,
        err,
        s
    );
    assert_eq!(err.code(), expected_code);
}

#[test]
fn error_codes_match_dictionary_common() {
    assert_bracket_code(AppError::DatabaseError("e".into()), "DB_ERROR");
    assert_bracket_code(
        AppError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "x")),
        "IO_ERROR",
    );
    assert_bracket_code(AppError::OllamaError("x".into()), "LLM_ERROR");
    assert_bracket_code(AppError::RoleNotFound("r".into()), "ROLE_NOT_FOUND");
    assert_bracket_code(AppError::InvalidParameter("p".into()), "INVALID_PARAMETER");
    assert_bracket_code(
        AppError::SerializationError(
            serde_json::from_str::<serde_json::Value>("not-json").unwrap_err(),
        ),
        "SERDE_ERROR",
    );
    assert_bracket_code(AppError::RolePackExists("dup".into()), "ROLE_PACK_EXISTS");
    assert_bracket_code(
        AppError::PermissionDenied("nope".into()),
        "API_PERMISSION_DENIED",
    );
    assert_bracket_code(
        AppError::DirectoryPluginNotFound("p1".into()),
        "API_PLUGIN_NOT_FOUND",
    );
    assert_bracket_code(AppError::Unknown("u".into()), "UNKNOWN_ERROR");
    assert_bracket_code(
        AppError::ChatGenerationCancelled,
        "CHAT_GENERATION_CANCELLED",
    );
}

#[test]
fn transaction_error_uses_custom_code() {
    let err = AppError::TransactionError {
        code: "TXN_COMMIT_FAILED",
        message: "rollback".into(),
    };
    assert_eq!(err.code(), "TXN_COMMIT_FAILED");
    let s = err.to_frontend_error();
    assert!(s.starts_with("[TXN_COMMIT_FAILED]"));
}
