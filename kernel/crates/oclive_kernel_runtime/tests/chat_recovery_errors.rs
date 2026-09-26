use oclive_kernel_runtime::app_error_from_http_response;
use oclive_kernel_types::AppError;

#[test]
fn recovery_errors_keep_their_machine_codes_across_http() {
    for error in [
        AppError::ChatRequestConflict,
        AppError::ChatRequestUnconfirmed,
    ] {
        let body = serde_json::json!({"error":error.kernel_error_body()}).to_string();
        assert_eq!(
            app_error_from_http_response(409, &body).code(),
            error.code()
        );
    }
}
