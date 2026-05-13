//! 对话会话列表（短期记忆命名空间聚合）。

use crate::state::AppState;
use serde_json::Value;

pub async fn get_conversation_list_impl(state: &AppState) -> Result<Value, String> {
    oclive_kernel_runtime::domain::conversation_query::get_conversation_list(state)
        .await
        .map_err(|e| e.to_frontend_error())
}
