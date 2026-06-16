//! Theater scene director Tauri command (attach HTTP vs in-process).

use crate::api::error::CommandError;
use crate::kernel_attach::KernelHttpClient;
use crate::kernel_lifecycle::SharedKernelConnection;
use oclive_kernel_host::domain::theater::generate_scene;
use oclive_kernel_host::state::SharedAppState;
use oclive_kernel_types::models::dto::{TheaterSceneRequest, TheaterSceneResponse};
use tauri::{AppHandle, Manager, State};

/// Rewrites a full theater scene via the scene-director LLM path.
///
/// # Errors
///
/// Returns [`CommandError`] when validation or kernel HTTP fails.
#[tauri::command]
pub async fn generate_theater_scene(
    req: TheaterSceneRequest,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<TheaterSceneResponse, CommandError> {
    if let Some(conn) = app.try_state::<SharedKernelConnection>() {
        KernelHttpClient::generate_theater_scene_via_http(&conn, &req)
            .await
            .map_err(Into::into)
    } else {
        debug_assert!(
            cfg!(test),
            "desktop shell should route theater scene through kernel HTTP"
        );
        generate_scene(state.inner().as_ref(), &req)
            .await
            .map_err(Into::into)
    }
}
