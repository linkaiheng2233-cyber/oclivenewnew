//! 创作者 profile 预览：实现于 `oclive_kernel_runtime::domain::profile_preview`。

pub use oclive_kernel_runtime::domain::profile_preview::{
    PreviewProfileFromPathRequest, ProfileBackendsDto, ProfilePermissionsDto, ProfilePluginSpecDto,
    ProfilePreviewDto,
};

#[tauri::command]
pub async fn preview_profile_from_path(
    req: PreviewProfileFromPathRequest,
) -> Result<ProfilePreviewDto, String> {
    oclive_kernel_runtime::domain::profile_preview::preview_profile_from_path(&req)
}
