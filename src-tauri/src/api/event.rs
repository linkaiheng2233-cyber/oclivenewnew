use crate::error::AppError;
use crate::models::dto::{CreateEventRequest, CreateEventResponse, EventItem, QueryEventsRequest};
use crate::models::EventType;
use crate::state::AppState;
use tauri::State;

fn parse_event_type(s: &str) -> Result<EventType, String> {
    match s {
        "Quarrel" => Ok(EventType::Quarrel),
        "Apology" => Ok(EventType::Apology),
        "Praise" => Ok(EventType::Praise),
        "Complaint" => Ok(EventType::Complaint),
        "Confession" => Ok(EventType::Confession),
        "Joke" => Ok(EventType::Joke),
        "Ignore" => Ok(EventType::Ignore),
        _ => Err(
            AppError::InvalidParameter(format!("Invalid event_type: {}", s)).to_frontend_error(),
        ),
    }
}

pub async fn query_events_impl(
    state: &AppState,
    req: &QueryEventsRequest,
) -> Result<Vec<EventItem>, String> {
    if req.limit <= 0 || req.limit > 100 {
        return Err(
            AppError::InvalidParameter("limit must be between 1 and 100".to_string())
                .to_frontend_error(),
        );
    }
    if req.offset < 0 {
        return Err(
            AppError::InvalidParameter("offset must be >= 0".to_string()).to_frontend_error(),
        );
    }

    let rows = state
        .db_manager
        .list_events_paged(&req.role_id, req.limit, req.offset)
        .await
        .map_err(|e| e.to_frontend_error())?;

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

pub async fn create_event_impl(
    state: &AppState,
    req: &CreateEventRequest,
) -> Result<CreateEventResponse, String> {
    let event_type = parse_event_type(&req.event_type)?;
    state
        .db_manager
        .ensure_role_runtime(&req.role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;

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
        .map_err(|e| e.to_frontend_error())?;

    Ok(CreateEventResponse {
        id,
        role_id: req.role_id.clone(),
        event_type: format!("{:?}", event_type),
        timestamp,
        description: req.description.clone(),
    })
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
