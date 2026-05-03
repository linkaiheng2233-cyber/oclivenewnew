//! 应用级设置（`app_settings`），供受控桥接更新。

use crate::state::AppState;
use serde_json::Value;

pub async fn update_settings_impl(state: &AppState, params: &Value) -> Result<Value, String> {
    oclive_kernel_runtime::domain::app_settings_commands::update_settings(state, params).await
}
