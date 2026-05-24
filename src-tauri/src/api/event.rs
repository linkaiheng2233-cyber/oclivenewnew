use crate::error::AppError;
use crate::models::dto::{CreateEventRequest, CreateEventResponse, EventItem, QueryEventsRequest};
use crate::models::EventType;
use crate::state::AppState;
use tauri::State;
use crate::api::error::CommandError;

fn parse_event_type(s: &str) -> Result<EventType, CommandError> {
    match s {
        "Quarrel" => Ok(EventType::Quarrel),
        "Apology" => Ok(EventType::Apology),
        "Praise" => Ok(EventType::Praise),
        "Complaint" => Ok(EventType::Complaint),
        "Confession" => Ok(EventType::Confession),
        "Joke" => Ok(EventType::Joke),
        "Ignore" => Ok(EventType::Ignore),
        _ => Err(AppError::InvalidParameter(format!("Invalid event_type: {}", s)).into()),
    }
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn query_events_impl(
    state: &AppState,
    req: &QueryEventsRequest,
) -> Result<Vec<EventItem>, CommandError> {
    if req.limit <= 0 || req.limit > 100 {
        return Err(AppError::InvalidParameter("limit must be between 1 and 100".to_string()).into());
    }
    if req.offset < 0 {
        return Err(AppError::InvalidParameter("offset must be >= 0".to_string()).into());
    }

    let rows = state
        .db_manager
        .list_events_paged(&req.role_id, req.limit, req.offset)
        .await
        ?;

    Ok(rows
        .into_iter()
        .map(|r| EventItem {
            id: r.id,
            role_id: r.role_id,
            event_type: r.event_type,
            user_emotion: r.user_emotion,
            bot_emotion: r.bot_emotion,
            timestamp: r.created_at,
            description: r.resolution,
        })
        .collect())
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn create_event_impl(
    state: &AppState,
    req: &CreateEventRequest,
) -> Result<CreateEventResponse, CommandError> {
    let event_type = parse_event_type(&req.event_type)?;
    state
        .db_manager
        .ensure_role_runtime(&req.role_id)
        .await
        ?;

    let (id, timestamp) = state
        .db_manager
        .insert_manual_event(
            &req.role_id,
            &event_type,
            "manual",
            "manual",
            req.description.as_deref(),
        )
        .await
        ?;

    Ok(CreateEventResponse {
        id,
        role_id: req.role_id.clone(),
        event_type: format!("{:?}", event_type),
        timestamp,
        description: req.description.clone(),
    })
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn query_events(
    req: QueryEventsRequest,
    state: State<'_, AppState>,
) -> Result<Vec<EventItem>, CommandError> {
    query_events_impl(&state, &req).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn create_event(
    req: CreateEventRequest,
    state: State<'_, AppState>,
) -> Result<CreateEventResponse, CommandError> {
    create_event_impl(&state, &req).await
}
