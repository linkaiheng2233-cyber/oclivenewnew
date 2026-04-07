use crate::domain::chat_engine::process_message;
use crate::error::AppError;
use crate::models::dto::{SendMessageRequest, SendMessageResponse};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn send_message(
    req: SendMessageRequest,
    state: State<'_, AppState>,
) -> Result<SendMessageResponse, String> {
    process_message(&state, &req)
        .await
        .map_err(|e: AppError| e.to_frontend_error())
}
