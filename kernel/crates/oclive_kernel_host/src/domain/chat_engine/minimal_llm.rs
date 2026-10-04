//! Borrowed Base view of this Host's already composed text generation path.
//! No provider is rebuilt and no product turn is scheduled by this adapter.

use oclive_kernel_contracts::{BaseCallFuture, LlmBase};
use oclive_kernel_types::{AppError, BaseCallError, BaseCallErrorKind, LlmBaseRequest};

use crate::state::AppState;

pub(crate) struct HostTextGenerationBase<'a> {
    pub(crate) state: &'a AppState,
}

impl LlmBase for HostTextGenerationBase<'_> {
    fn generate<'a>(&'a self, request: LlmBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        Box::pin(async move {
            super::process_message::generate_minimal_text(self.state, request.input)
                .await
                .map_err(project_error)
        })
    }
}

// AppError has no typed timeout/cancellation source at this boundary. The original
// error remains available on the product path; Base diagnostics are not codes.
fn project_error(error: AppError) -> BaseCallError {
    let kind = match &error {
        AppError::HighRiskCapabilityNotGranted { .. } | AppError::RemoteServiceUnavailable(_) => {
            BaseCallErrorKind::Unavailable
        }
        _ => BaseCallErrorKind::Failed,
    };
    BaseCallError {
        kind,
        detail: Some(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_refusals_are_unavailable_with_original_diagnostics() {
        for error in [
            AppError::HighRiskCapabilityNotGranted {
                capability: "network:fixture".into(),
                id: "fixture".into(),
            },
            AppError::RemoteServiceUnavailable("fixture unavailable".into()),
        ] {
            let diagnostic = error.to_string();
            let projected = project_error(error);
            assert_eq!(projected.kind, BaseCallErrorKind::Unavailable);
            assert_eq!(projected.detail.as_deref(), Some(diagnostic.as_str()));
        }
    }

    #[test]
    fn failure_words_do_not_manufacture_timeout_cancellation_or_retry() {
        let error = AppError::OllamaError("timed out; cancelled; safe to retry".into());
        let diagnostic = error.to_string();
        let projected = project_error(error);
        assert_eq!(projected.kind, BaseCallErrorKind::Failed);
        assert_eq!(projected.detail.as_deref(), Some(diagnostic.as_str()));
    }
}
