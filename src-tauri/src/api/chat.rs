use crate::domain::chat_engine::process_message;
use crate::models::dto::{SendMessageRequest, SendMessageResponse};
use crate::state::AppState;
use tauri::State;
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn send_message(
    req: SendMessageRequest,
    state: State<'_, AppState>,
) -> Result<SendMessageResponse, crate::api::error::CommandError> {
    process_message(&state, &req).await.map_err(Into::into)
}
