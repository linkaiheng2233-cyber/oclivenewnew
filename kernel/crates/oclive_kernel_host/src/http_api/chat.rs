use super::{api_error, kernel_http_error, ApiError, ChatApiRequest, ChatApiResponse};
use crate::domain::chat_engine::{process_message, process_message_stream};
use crate::error::{http_chat_codes, AppError};
use crate::infrastructure::chat_storage::{SessionMeta, StoredMessage};
use crate::models::dto::{
    AdultStagedBeatDto, BeginAdultStageGenerationRequest, BeginAdultStageGenerationResponse,
    CancelAdultStageGenerationRequest, CommitAdultStagedBeatRequest, ListAdultStagedBeatsRequest,
    ListAdultStagedBeatsResponse, StageAdultBeatRequest,
};
use crate::models::dto::{SendMessageRequest, SendMessageResponse};
use crate::service::{execute_chat_storage_proxy, ChatStorageProxyOp};
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use futures_util::stream;
use oclive_kernel_contracts::LlmTokenSink;
use serde::Deserialize;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::spawn_blocking;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt as _;

/// Inside `spawn_blocking`: `load_role_from_dir` and directory probing are both blocking I/O; do not call them directly on an async thread.
enum ChatRoleLoadError {
    NotDirectory(String),
    Load(crate::error::AppError),
}

fn adult_stage_api_error(error: AppError) -> ApiError {
    let status = if matches!(error, AppError::InvalidParameter(_)) {
        axum::http::StatusCode::BAD_REQUEST
    } else {
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    };
    api_error(status, error.kernel_error_body())
}

pub(crate) async fn begin_adult_stage_route(
    State(state): State<Arc<AppState>>,
    Json(request): Json<BeginAdultStageGenerationRequest>,
) -> Result<Json<BeginAdultStageGenerationResponse>, ApiError> {
    crate::domain::adult_stage::begin_adult_stage_generation(state.as_ref(), request)
        .await
        .map(Json)
        .map_err(adult_stage_api_error)
}

pub(crate) async fn generate_adult_staged_beat_route(
    State(state): State<Arc<AppState>>,
    Json(request): Json<StageAdultBeatRequest>,
) -> Result<Json<AdultStagedBeatDto>, ApiError> {
    crate::domain::adult_stage::generate_adult_staged_beat(state.as_ref(), request)
        .await
        .map(Json)
        .map_err(adult_stage_api_error)
}

pub(crate) async fn commit_adult_staged_beat_route(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CommitAdultStagedBeatRequest>,
) -> Result<Json<SendMessageResponse>, ApiError> {
    crate::domain::adult_stage::commit_adult_staged_beat(state.as_ref(), request)
        .await
        .map(Json)
        .map_err(adult_stage_api_error)
}

pub(crate) async fn cancel_adult_stage_route(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CancelAdultStageGenerationRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    crate::domain::adult_stage::cancel_adult_stage_generation(state.as_ref(), request)
        .await
        .map(|()| Json(serde_json::json!({ "ok": true })))
        .map_err(adult_stage_api_error)
}

pub(crate) async fn list_adult_staged_beats_route(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ListAdultStagedBeatsRequest>,
) -> Result<Json<ListAdultStagedBeatsResponse>, ApiError> {
    crate::domain::adult_stage::list_adult_staged_beats(state.as_ref(), request)
        .await
        .map(Json)
        .map_err(adult_stage_api_error)
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatSessionsQuery {
    role_id: String,
    scene_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatMessagesQuery {
    session_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
}

pub(crate) async fn chat(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ChatApiRequest>,
) -> Result<Json<ChatApiResponse>, ApiError> {
    let session_echo = body.session_id.clone();
    let user_message = body.message.trim().to_string();
    if user_message.is_empty() {
        return Err(api_error(
            axum::http::StatusCode::BAD_REQUEST,
            kernel_http_error(
                http_chat_codes::EMPTY_MESSAGE,
                "message must not be empty or whitespace-only",
                Some("请至少输入 1 个可见字符".into()),
            ),
        ));
    }
    let path = PathBuf::from(body.role_path.trim());
    let storage = state.storage.clone();
    let blocked = spawn_blocking(move || {
        if !path.is_dir() {
            return Err(ChatRoleLoadError::NotDirectory(path.display().to_string()));
        }
        storage
            .load_role_from_dir(&path)
            .map_err(ChatRoleLoadError::Load)
    })
    .await
    .map_err(|e| {
        api_error(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            kernel_http_error(
                http_chat_codes::LOAD_ROLE_TASK_PANIC,
                format!("load_role task panicked: {e}"),
                None,
            ),
        )
    })?;

    let role = match blocked {
        Err(ChatRoleLoadError::NotDirectory(display)) => {
            return Err(api_error(
                axum::http::StatusCode::BAD_REQUEST,
                kernel_http_error(
                    http_chat_codes::INVALID_ROLE_PATH,
                    format!("role_path is not a directory: {display}"),
                    Some("请传入包含 manifest.json 的角色目录绝对路径".into()),
                ),
            ));
        }
        Err(ChatRoleLoadError::Load(e)) => {
            let mut k = e.kernel_error_body();
            k.hint = Some("请检查角色目录结构与 manifest/settings 是否完整".into());
            return Err(api_error(axum::http::StatusCode::BAD_REQUEST, k));
        }
        Ok(r) => r,
    };

    let personality_source = role.evolution_config.personality_source;
    let role = Arc::new(role);

    state.invalidate_personality_cache_for_role(role.id.as_str());

    state.insert_http_api_role(role.id.clone(), Arc::clone(&role));

    let req = SendMessageRequest {
        role_id: role.id.clone(),
        user_message,
        scene_id: body.scene_id,
        session_id: body.session_id,
        include_raw_reply: body.include_raw_reply,
        adult: body.adult,
    };

    let res: SendMessageResponse = process_message(&state, &req).await.map_err(|e: AppError| {
        let mut k = e.kernel_error_body();
        k.hint = Some("请查看 oclive 日志（target: oclive_chat / oclive_plugin）".into());
        api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, k)
    })?;

    Ok(Json(ChatApiResponse {
        data: res,
        personality_source,
        session_id: session_echo,
    }))
}

pub(crate) async fn chat_stream(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ChatApiRequest>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let session_echo = body.session_id.clone();
    let user_message = body.message.trim().to_string();
    if user_message.is_empty() {
        return Err(api_error(
            axum::http::StatusCode::BAD_REQUEST,
            kernel_http_error(
                http_chat_codes::EMPTY_MESSAGE,
                "message must not be empty or whitespace-only",
                Some("请至少输入 1 个可见字符".into()),
            ),
        ));
    }
    let path = PathBuf::from(body.role_path.trim());
    let storage = state.storage.clone();
    let blocked = spawn_blocking(move || {
        if !path.is_dir() {
            return Err(ChatRoleLoadError::NotDirectory(path.display().to_string()));
        }
        storage
            .load_role_from_dir(&path)
            .map_err(ChatRoleLoadError::Load)
    })
    .await
    .map_err(|e| {
        api_error(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            kernel_http_error(
                http_chat_codes::LOAD_ROLE_TASK_PANIC,
                format!("load_role task panicked: {e}"),
                None,
            ),
        )
    })?;

    let role = match blocked {
        Err(ChatRoleLoadError::NotDirectory(display)) => {
            return Err(api_error(
                axum::http::StatusCode::BAD_REQUEST,
                kernel_http_error(
                    http_chat_codes::INVALID_ROLE_PATH,
                    format!("role_path is not a directory: {display}"),
                    Some("请传入包含 manifest.json 的角色目录绝对路径".into()),
                ),
            ));
        }
        Err(ChatRoleLoadError::Load(e)) => {
            let mut k = e.kernel_error_body();
            k.hint = Some("请检查角色目录结构与 manifest/settings 是否完整".into());
            return Err(api_error(axum::http::StatusCode::BAD_REQUEST, k));
        }
        Ok(r) => r,
    };

    let personality_source = role.evolution_config.personality_source;
    let role = Arc::new(role);

    state.invalidate_personality_cache_for_role(role.id.as_str());
    state.insert_http_api_role(role.id.clone(), Arc::clone(&role));

    let req = SendMessageRequest {
        role_id: role.id.clone(),
        user_message,
        scene_id: body.scene_id,
        session_id: body.session_id,
        include_raw_reply: body.include_raw_reply,
        adult: body.adult,
    };

    let (tx, rx) = mpsc::unbounded_channel();
    let state_bg = Arc::clone(&state);
    tokio::spawn(async move {
        let (token_tx, mut token_rx) = mpsc::unbounded_channel::<String>();
        let tx_tokens = tx.clone();
        let forward = tokio::spawn(async move {
            while let Some(token) = token_rx.recv().await {
                let payload = serde_json::json!({ "token": token }).to_string();
                let _ = tx_tokens.send(Ok(Event::default().event("token").data(payload)));
            }
        });

        let on_token: LlmTokenSink = Arc::new(move |t: &str| {
            let _ = token_tx.send(t.to_string());
        });

        let result = process_message_stream(state_bg.as_ref(), &req, on_token).await;
        let _ = forward.await;

        match result {
            Ok(res) => {
                let api = ChatApiResponse {
                    data: res,
                    personality_source,
                    session_id: session_echo,
                };
                match serde_json::to_string(&api) {
                    Ok(json) => {
                        let _ = tx.send(Ok(Event::default().event("done").data(json)));
                    }
                    Err(e) => {
                        let _ = tx.send(Ok(Event::default()
                            .event("error")
                            .data(format!("encode failed: {e}"))));
                    }
                }
            }
            Err(e) => {
                let k = e.kernel_error_body();
                let _ = tx.send(Ok(Event::default()
                    .event("error")
                    .data(serde_json::json!({ "error": k }).to_string())));
            }
        }
    });

    let sse_stream = UnboundedReceiverStream::new(rx)
        .chain(stream::once(async { Ok(Event::default().comment("end")) }));

    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default()))
}

pub(crate) async fn chat_sessions_route(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ChatSessionsQuery>,
) -> Result<Json<Vec<SessionMeta>>, ApiError> {
    state
        .conversation_store
        .list_sessions(
            q.role_id.trim(),
            q.scene_id.trim(),
            q.limit.unwrap_or(50),
            q.offset.unwrap_or(0),
        )
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, k)
        })
}

pub(crate) async fn chat_messages_route(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ChatMessagesQuery>,
) -> Result<Json<Vec<StoredMessage>>, ApiError> {
    state
        .conversation_store
        .fetch_messages(
            q.session_id.trim(),
            q.limit.unwrap_or(500),
            q.offset.unwrap_or(0),
        )
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, k)
        })
}

pub(crate) async fn chat_storage_proxy_route(
    State(state): State<Arc<AppState>>,
    Json(op): Json<ChatStorageProxyOp>,
) -> Result<Json<serde_json::Value>, ApiError> {
    execute_chat_storage_proxy(state.as_ref(), op)
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, k)
        })
}
