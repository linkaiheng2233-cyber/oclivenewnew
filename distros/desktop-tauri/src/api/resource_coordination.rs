//! Read-only host Resource Coordinator diagnostics.

#![allow(clippy::missing_errors_doc)]

use oclive_kernel_host::state::SharedAppState;
use oclive_kernel_types::ResourceCoordinationDiagnostics;
use tauri::State;

use crate::api::error::CommandError;

#[tauri::command]
pub async fn get_resource_coordination_diagnostics(
    state: State<'_, SharedAppState>,
) -> Result<ResourceCoordinationDiagnostics, CommandError> {
    Ok(oclive_kernel_host::service::get_resource_coordination_diagnostics_impl(&state).await)
}
