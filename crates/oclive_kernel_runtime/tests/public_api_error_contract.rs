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

/// 任务 C.3：`AppError::code()` 与 `to_frontend_error()` 信封一致（`[CODE]` 前缀），便于 i18n 外层解析。
#[test]
fn all_app_error_variants_expose_stable_bracket_envelope() {
    let cases: Vec<(AppError, &str)> = vec![
        (AppError::DatabaseError("e".into()), "DB_ERROR"),
        (AppError::IoError(std::io::Error::other("io")), "IO_ERROR"),
        (
            AppError::OllamaError("remote_plugin transport".into()),
            "LLM_ERROR",
        ),
        (AppError::RoleNotFound("x".into()), "ROLE_NOT_FOUND"),
        (AppError::RolePackExists("dup".into()), "ROLE_PACK_EXISTS"),
        (AppError::InvalidParameter("p".into()), "INVALID_PARAMETER"),
        (
            AppError::PermissionDenied("n".into()),
            "API_PERMISSION_DENIED",
        ),
        (
            AppError::DirectoryPluginNotFound("p".into()),
            "API_PLUGIN_NOT_FOUND",
        ),
        (
            AppError::SerializationError(
                serde_json::from_str::<serde_json::Value>("{").unwrap_err(),
            ),
            "SERDE_ERROR",
        ),
        (AppError::Unknown("u".into()), "UNKNOWN_ERROR"),
        (
            AppError::ChatGenerationCancelled,
            "CHAT_GENERATION_CANCELLED",
        ),
    ];
    let mut seen = std::collections::HashSet::new();
    for (err, code) in cases {
        assert!(seen.insert(code), "duplicate code in fixture: {}", code);
        let envelope = err.to_frontend_error();
        assert!(
            envelope.starts_with(&format!("[{}] ", code)),
            "envelope should start with [{}] , got {:?}",
            code,
            envelope
        );
        assert_eq!(err.code(), code);
    }
}

/// 关键路径错误载体使用 `OllamaError`（历史命名）：面向插件/LLM 的英文句式，便于日志与跨语言壳层映射。
#[test]
fn remote_transport_errors_use_english_llm_error_message() {
    let err = AppError::OllamaError(
        "remote_plugin transport kind=timeout method=memory.rank url=http://127.0.0.1:9/rpc err=timed out"
            .into(),
    );
    let s = err.to_string();
    assert!(
        s.is_ascii(),
        "expected ASCII technical message for logs, got {:?}",
        s
    );
    let fe = err.to_frontend_error();
    assert!(fe.starts_with("[LLM_ERROR]"));
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
