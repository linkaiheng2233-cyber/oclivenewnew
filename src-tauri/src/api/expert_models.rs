//! Module 9：Expert Models — 实现于 `oclive_kernel_runtime::domain::expert_models_admin`。

pub use oclive_kernel_runtime::domain::expert_models_admin::{
    ExpertModelsImportGgufRequest, ExpertModelsLocalGgufPathRequest,
    ExpertModelsRenameLocalGgufRequest, ExpertModelsSetGgufRepoMetaRequest, LocalModelFileDto,
};

use crate::models::dto::{
    ExpertModelsApplyResult, ExpertModelsApplyToSessionRequest,
    ExpertModelsClearRoleDefaultRequest, ExpertModelsClearRunsRequest,
    ExpertModelsClearSessionOverrideRequest, ExpertModelsEffectiveResponse,
    ExpertModelsGetEffectiveRequest, ExpertModelsGetRunDetailRequest,
    ExpertModelsGetRunDetailResponse, ExpertModelsListRunsResponse,
    ExpertModelsRollbackLastRunRequest, ExpertModelsRollbackToRunRequest,
    ExpertModelsSetRoleDefaultRequest, ExpertModelsSetRunPinnedRequest,
    ExpertModelsSetSessionOverrideRequest, ExpertWorkflowDto, ExpertWorkflowsDeleteRequest,
    ExpertWorkflowsGetRequest, ExpertWorkflowsListResponse, ExpertWorkflowsSaveRequest,
};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn expert_models_get_effective(
    req: ExpertModelsGetEffectiveRequest,
    state: State<'_, AppState>,
) -> Result<ExpertModelsEffectiveResponse, String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_get_effective(&state, &req)
        .await
}

#[tauri::command]
pub async fn expert_models_set_session_override(
    req: ExpertModelsSetSessionOverrideRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_set_session_override(
        &state, &req,
    )
    .await
}

#[tauri::command]
pub async fn expert_models_clear_session_override(
    req: ExpertModelsClearSessionOverrideRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_clear_session_override(
        &state, &req,
    )
    .await
}

#[tauri::command]
pub async fn expert_models_set_role_default(
    req: ExpertModelsSetRoleDefaultRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_set_role_default(&state, &req)
        .await
}

#[tauri::command]
pub async fn expert_models_clear_role_default(
    req: ExpertModelsClearRoleDefaultRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_clear_role_default(
        &state, &req,
    )
    .await
}

#[tauri::command]
pub async fn expert_models_apply_to_session(
    req: ExpertModelsApplyToSessionRequest,
    state: State<'_, AppState>,
) -> Result<ExpertModelsApplyResult, String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_apply_to_session(&state, &req)
        .await
}

#[tauri::command]
pub async fn expert_models_list_local_base_models(
    state: State<'_, AppState>,
) -> Result<Vec<LocalModelFileDto>, String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_list_local_base_models(&state)
        .await
}

#[tauri::command]
pub async fn expert_models_list_local_loras(
    state: State<'_, AppState>,
) -> Result<Vec<LocalModelFileDto>, String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_list_local_loras(&state).await
}

#[tauri::command]
pub async fn expert_models_import_base_gguf(
    req: ExpertModelsImportGgufRequest,
    state: State<'_, AppState>,
) -> Result<LocalModelFileDto, String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_import_base_gguf(&state, &req)
        .await
}

#[tauri::command]
pub async fn expert_models_import_lora_gguf(
    req: ExpertModelsImportGgufRequest,
    state: State<'_, AppState>,
) -> Result<LocalModelFileDto, String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_import_lora_gguf(&state, &req)
        .await
}

#[tauri::command]
pub async fn expert_models_delete_local_base_model(
    req: ExpertModelsLocalGgufPathRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_delete_local_base_model(
        &state, &req,
    )
    .await
}

#[tauri::command]
pub async fn expert_models_rename_local_base_model(
    req: ExpertModelsRenameLocalGgufRequest,
    state: State<'_, AppState>,
) -> Result<LocalModelFileDto, String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_rename_local_base_model(
        &state, &req,
    )
    .await
}

#[tauri::command]
pub async fn expert_models_set_gguf_repo_meta(
    req: ExpertModelsSetGgufRepoMetaRequest,
    state: State<'_, AppState>,
) -> Result<LocalModelFileDto, String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_set_gguf_repo_meta(
        &state, &req,
    )
    .await
}

#[tauri::command]
pub async fn expert_models_rollback_last_run(
    req: ExpertModelsRollbackLastRunRequest,
    state: State<'_, AppState>,
) -> Result<ExpertModelsApplyResult, String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_rollback_last_run(
        &state, &req,
    )
    .await
}

#[tauri::command]
pub async fn expert_models_list_runs(
    req: ExpertModelsGetEffectiveRequest,
    state: State<'_, AppState>,
) -> Result<ExpertModelsListRunsResponse, String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_list_runs(&state, &req).await
}

#[tauri::command]
pub async fn expert_models_get_run_detail(
    req: ExpertModelsGetRunDetailRequest,
    state: State<'_, AppState>,
) -> Result<ExpertModelsGetRunDetailResponse, String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_get_run_detail(&state, &req)
        .await
}

#[tauri::command]
pub async fn expert_models_clear_runs(
    req: ExpertModelsClearRunsRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_clear_runs(&state, &req).await
}

#[tauri::command]
pub async fn expert_models_set_run_pinned(
    req: ExpertModelsSetRunPinnedRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_set_run_pinned(&state, &req)
        .await
}

#[tauri::command]
pub async fn expert_models_rollback_to_run(
    req: ExpertModelsRollbackToRunRequest,
    state: State<'_, AppState>,
) -> Result<ExpertModelsApplyResult, String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_models_rollback_to_run(&state, &req)
        .await
}

#[tauri::command]
pub async fn expert_workflows_list(
    state: State<'_, AppState>,
) -> Result<ExpertWorkflowsListResponse, String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_workflows_list(&state).await
}

#[tauri::command]
pub async fn expert_workflows_get(
    req: ExpertWorkflowsGetRequest,
    state: State<'_, AppState>,
) -> Result<ExpertWorkflowDto, String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_workflows_get(&state, &req).await
}

#[tauri::command]
pub async fn expert_workflows_save(
    req: ExpertWorkflowsSaveRequest,
    state: State<'_, AppState>,
) -> Result<ExpertWorkflowDto, String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_workflows_save(&state, &req).await
}

#[tauri::command]
pub async fn expert_workflows_delete(
    req: ExpertWorkflowsDeleteRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    oclive_kernel_runtime::domain::expert_models_admin::expert_workflows_delete(&state, &req).await
}
