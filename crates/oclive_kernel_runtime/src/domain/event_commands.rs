//! 事件列表与手动创建（无 Tauri 依赖）。

use crate::error::{AppError, Result};
use crate::models::dto::{CreateEventRequest, CreateEventResponse, EventItem, QueryEventsRequest};
use crate::models::EventType;
use crate::state::KernelAppState;

fn parse_event_type(s: &str) -> Result<EventType> {
    match s {
        "Quarrel" => Ok(EventType::Quarrel),
        "Apology" => Ok(EventType::Apology),
        "Praise" => Ok(EventType::Praise),
        "Complaint" => Ok(EventType::Complaint),
        "Confession" => Ok(EventType::Confession),
        "Joke" => Ok(EventType::Joke),
        "Ignore" => Ok(EventType::Ignore),
        _ => Err(AppError::InvalidParameter(format!(
            "Invalid event_type: {}",
            s
        ))),
    }
}

pub async fn query_events(
    state: &KernelAppState,
    req: &QueryEventsRequest,
) -> Result<Vec<EventItem>> {
    if req.limit <= 0 || req.limit > 100 {
        return Err(AppError::InvalidParameter(
            "limit must be between 1 and 100".to_string(),
        ));
    }
    if req.offset < 0 {
        return Err(AppError::InvalidParameter(
            "offset must be >= 0".to_string(),
        ));
    }

    let rows = state
        .db_manager
        .list_events_paged(&req.role_id, req.limit, req.offset)
        .await?;

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

pub async fn create_event(
    state: &KernelAppState,
    req: &CreateEventRequest,
) -> Result<CreateEventResponse> {
    let event_type = parse_event_type(&req.event_type)?;
    state.db_manager.ensure_role_runtime(&req.role_id).await?;

    let (id, timestamp) = state
        .db_manager
        .insert_manual_event(
            &req.role_id,
            &event_type,
            "manual",
            "manual",
            req.description.as_deref(),
        )
        .await?;

    Ok(CreateEventResponse {
        id,
        role_id: req.role_id.clone(),
        event_type: format!("{:?}", event_type),
        timestamp,
        description: req.description.clone(),
    })
}
