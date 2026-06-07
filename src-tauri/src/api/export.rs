use crate::api::error::CommandError;
use oclive_kernel_types::models::dto::{ExportChatLogsRequest, ExportChatLogsResponse};
use oclive_kernel_host::state::SharedAppState;
use tauri::State;

pub use oclive_kernel_host::service::export::export_chat_logs_impl;

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn export_chat_logs(
    req: ExportChatLogsRequest,
    state: State<'_, SharedAppState>,
) -> Result<ExportChatLogsResponse, CommandError> {
    export_chat_logs_impl(&state, &req).await
}
