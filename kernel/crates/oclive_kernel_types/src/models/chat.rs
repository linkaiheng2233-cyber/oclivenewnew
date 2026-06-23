use serde::{Deserialize, Serialize};

/// Legacy/simple HTTP chat request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub role_id: String,
    pub user_input: String,
}

/// Legacy/simple HTTP chat response body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: String,
    pub emotion: String,
}
