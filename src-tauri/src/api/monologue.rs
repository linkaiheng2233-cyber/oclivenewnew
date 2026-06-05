use crate::models::dto::{GenerateMonologueRequest, GenerateMonologueResponse};
use crate::state::SharedAppState;
use oclive_kernel_host::service::generate_monologue_impl;
use tauri::State;
use crate::api::error::CommandError;

pub use oclive_kernel_host::service::time::generate_monologue_lines;

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn generate_monologue(
    req: GenerateMonologueRequest,
    state: State<'_, SharedAppState>,
) -> Result<GenerateMonologueResponse, CommandError> {
    generate_monologue_impl(&state, &req.role_id).await
}
