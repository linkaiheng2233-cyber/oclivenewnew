//! Read-only capability registry and execution-plan diagnostics.

#![allow(clippy::missing_errors_doc)]

use crate::api::error::CommandError;
use oclive_kernel_host::state::SharedAppState;
use oclive_kernel_types::{ExecutionPlanDiagnostics, GetExecutionPlanDiagnosticsRequest};
use tauri::State;

#[tauri::command]
pub async fn get_execution_plan_diagnostics(
    req: GetExecutionPlanDiagnosticsRequest,
    state: State<'_, SharedAppState>,
) -> Result<ExecutionPlanDiagnostics, CommandError> {
    oclive_kernel_host::service::get_execution_plan_diagnostics_impl(&state, &req).await
}
