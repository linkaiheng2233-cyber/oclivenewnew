//! Portable state transfer and memory/event query DTOs.

use serde::{Deserialize, Serialize};

/// Selects one role/persona runtime namespace for portable persona or memory transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableStateRequest {
    pub role_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

/// JSON payload returned for saving as `.ocpersona` or `.ocmemory`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableStateExportResponse {
    pub content: String,
    pub suggested_filename: String,
}

/// Imports a portable document into an installed role's runtime namespace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableStateImportRequest {
    pub role_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    pub content: String,
}

/// Import result. Seed memories remain read-only role-pack data and are not written to LTM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableStateImportResponse {
    pub imported_long_term: u32,
    pub skipped_memory_seed: u32,
    pub mutable_profile_restored: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryMemoriesRequest {
    pub role_id: String,
    pub limit: i32,
    pub offset: i32,
    /// Optional `ordinary` / `adult` filter; omitted returns both scopes.
    #[serde(default)]
    pub content_scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: String,
    pub role_id: String,
    pub content: String,
    /// Current store is long-term memory table only; fixed as `long_term`.
    pub memory_type: String,
    pub timestamp: String,
    pub importance: f64,
    pub content_scope: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryEventsRequest {
    pub role_id: String,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventItem {
    pub id: i64,
    pub role_id: String,
    pub event_type: String,
    pub user_emotion: Option<String>,
    pub bot_emotion: Option<String>,
    pub timestamp: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventRequest {
    pub role_id: String,
    pub event_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventResponse {
    pub id: i64,
    pub role_id: String,
    pub event_type: String,
    pub timestamp: String,
    pub description: Option<String>,
}
