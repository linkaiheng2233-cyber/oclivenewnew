use crate::models::dto::{ExportChatLogsRequest, ExportChatLogsResponse};
use crate::state::AppState;
use tauri::State;

pub use oclive_kernel_runtime::domain::export_chat_logs::sanitize_filename;

pub async fn export_chat_logs_impl(
    state: &AppState,
    req: &ExportChatLogsRequest,
) -> Result<ExportChatLogsResponse, String> {
    oclive_kernel_runtime::domain::export_chat_logs::export_chat_logs(
        state,
        req,
        env!("CARGO_PKG_VERSION"),
    )
    .await
    .map_err(|e| e.to_frontend_error())
}

#[tauri::command]
pub async fn export_chat_logs(
    req: ExportChatLogsRequest,
    state: State<'_, AppState>,
) -> Result<ExportChatLogsResponse, String> {
    export_chat_logs_impl(&state, &req).await
}
