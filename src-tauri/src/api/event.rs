use crate::models::dto::{CreateEventRequest, CreateEventResponse, EventItem, QueryEventsRequest};
use crate::state::AppState;
use tauri::State;

pub async fn query_events_impl(
    state: &AppState,
    req: &QueryEventsRequest,
) -> Result<Vec<EventItem>, String> {
    oclive_kernel_runtime::domain::event_commands::query_events(state, req)
        .await
        .map_err(|e| e.to_frontend_error())
}

pub async fn create_event_impl(
    state: &AppState,
    req: &CreateEventRequest,
) -> Result<CreateEventResponse, String> {
    oclive_kernel_runtime::domain::event_commands::create_event(state, req)
        .await
        .map_err(|e| e.to_frontend_error())
}

#[tauri::command]
pub async fn query_events(
    req: QueryEventsRequest,
    state: State<'_, AppState>,
) -> Result<Vec<EventItem>, String> {
    query_events_impl(&state, &req).await
}

#[tauri::command]
pub async fn create_event(
    req: CreateEventRequest,
    state: State<'_, AppState>,
) -> Result<CreateEventResponse, String> {
    create_event_impl(&state, &req).await
}
