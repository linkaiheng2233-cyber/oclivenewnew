use crate::api::error::CommandError;
use oclive_kernel_host::service::{
    export_portable_memory_impl, export_portable_persona_impl, import_portable_memory_impl,
    import_portable_persona_impl,
};
use oclive_kernel_host::state::SharedAppState;
use oclive_kernel_types::{
    PortableStateExportResponse, PortableStateImportRequest, PortableStateImportResponse,
    PortableStateRequest,
};
use tauri::State;

#[tauri::command]
pub async fn export_portable_persona(
    req: PortableStateRequest,
    state: State<'_, SharedAppState>,
) -> Result<PortableStateExportResponse, CommandError> {
    export_portable_persona_impl(&state, &req).await
}

#[tauri::command]
pub async fn import_portable_persona(
    req: PortableStateImportRequest,
    state: State<'_, SharedAppState>,
) -> Result<PortableStateImportResponse, CommandError> {
    import_portable_persona_impl(&state, &req).await
}

#[tauri::command]
pub async fn export_portable_memory(
    req: PortableStateRequest,
    state: State<'_, SharedAppState>,
) -> Result<PortableStateExportResponse, CommandError> {
    export_portable_memory_impl(&state, &req).await
}

#[tauri::command]
pub async fn import_portable_memory(
    req: PortableStateImportRequest,
    state: State<'_, SharedAppState>,
) -> Result<PortableStateImportResponse, CommandError> {
    import_portable_memory_impl(&state, &req).await
}
