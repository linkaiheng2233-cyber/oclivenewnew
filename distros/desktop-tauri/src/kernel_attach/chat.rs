//! Remote chat and adult-stage HTTP endpoints.

use super::app_error_from_http_response;
use super::KernelHttpClient;
use crate::error::AppError;
use crate::kernel_lifecycle::KernelConnection;
use oclive_kernel_types::models::dto::{
    AdultStagedBeatDto, BeginAdultStageGenerationRequest, BeginAdultStageGenerationResponse,
    CancelAdultStageGenerationRequest, CommitAdultStagedBeatRequest, ListAdultStagedBeatsRequest,
    ListAdultStagedBeatsResponse, SendMessageRequest, SendMessageResponse, StageAdultBeatRequest,
    TheaterSceneRequest, TheaterSceneResponse,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use std::path::Path;

use super::parse_sse_block;
use futures_util::StreamExt;

impl KernelHttpClient {
    pub async fn send_message_via_http(
        conn: &KernelConnection,
        role_path: &Path,
        req: &SendMessageRequest,
    ) -> Result<SendMessageResponse, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let url = format!("{}/chat", conn.base_url);
        let body = serde_json::json!({
            "role_path": role_path.to_string_lossy(),
            "message": req.user_message,
            "session_id": req.session_id,
            "scene_id": req.scene_id,
            "include_raw_reply": req.include_raw_reply,
            "adult": req.adult,
        });
        let res = conn
            .http_client()
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "remote chat request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("remote chat body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("remote chat JSON: {e}")))
    }

    /// `POST /chat/stream` — SSE `event:token` + final `event:done` with `SendMessageResponse`.
    pub async fn send_message_stream_via_http(
        conn: &KernelConnection,
        role_path: &Path,
        req: &SendMessageRequest,
        mut on_token: impl FnMut(&str) + Send,
    ) -> Result<SendMessageResponse, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let url = format!("{}/chat/stream", conn.base_url);
        let body = serde_json::json!({
            "role_path": role_path.to_string_lossy(),
            "message": req.user_message,
            "session_id": req.session_id,
            "scene_id": req.scene_id,
            "include_raw_reply": req.include_raw_reply,
            "adult": req.adult,
        });
        let res = conn
            .http_client()
            .post(&url)
            .header("Accept", "text/event-stream")
            .json(&body)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "remote chat stream request", e))?;
        let status = res.status();
        if !status.is_success() {
            let text = res
                .text()
                .await
                .map_err(|e| AppError::OllamaError(format!("remote chat stream body: {e}")))?;
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }

        #[derive(Deserialize)]
        struct StreamDoneEnvelope {
            data: SendMessageResponse,
        }

        let mut buffer = String::new();
        let mut final_response: Option<SendMessageResponse> = None;
        let mut byte_stream = res.bytes_stream();
        while let Some(chunk) = byte_stream.next().await {
            let chunk = chunk
                .map_err(|e| AppError::OllamaError(format!("remote chat stream chunk: {e}")))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));
            while let Some(sep) = buffer.find("\n\n") {
                let block = buffer[..sep].to_string();
                buffer = buffer[sep + 2..].to_string();
                let (event_name, data) = parse_sse_block(&block);
                if data.is_empty() {
                    continue;
                }
                match event_name.as_str() {
                    "token" => {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&data) {
                            if let Some(t) = v.get("token").and_then(|x| x.as_str()) {
                                on_token(t);
                            }
                        }
                    }
                    "done" => {
                        final_response = serde_json::from_str::<StreamDoneEnvelope>(&data)
                            .ok()
                            .map(|w| w.data)
                            .or_else(|| serde_json::from_str::<SendMessageResponse>(&data).ok());
                    }
                    "error" => {
                        return Err(app_error_from_http_response(status.as_u16(), &data));
                    }
                    _ => {}
                }
            }
        }
        final_response.ok_or_else(|| {
            AppError::OllamaError("remote chat stream ended without done event".into())
        })
    }

    async fn post_adult_stage<Req, Res>(
        conn: &KernelConnection,
        route: &str,
        request: &Req,
    ) -> Result<Res, AppError>
    where
        Req: Serialize + ?Sized,
        Res: DeserializeOwned,
    {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let response = conn
            .http_client()
            .post(format!(
                "{}/chat/adult-stage/{route}",
                conn.base_url.trim_end_matches('/')
            ))
            .json(request)
            .send()
            .await
            .map_err(|error| Self::map_send_err(&conn.base_url, "adult stage request", error))?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|error| AppError::OllamaError(format!("adult stage body: {error}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|error| AppError::OllamaError(format!("adult stage JSON: {error}")))
    }

    pub async fn begin_adult_stage_via_http(
        conn: &KernelConnection,
        request: &BeginAdultStageGenerationRequest,
    ) -> Result<BeginAdultStageGenerationResponse, AppError> {
        Self::post_adult_stage(conn, "begin", request).await
    }

    pub async fn generate_adult_staged_beat_via_http(
        conn: &KernelConnection,
        request: &StageAdultBeatRequest,
    ) -> Result<AdultStagedBeatDto, AppError> {
        Self::post_adult_stage(conn, "beat", request).await
    }

    pub async fn commit_adult_staged_beat_via_http(
        conn: &KernelConnection,
        request: &CommitAdultStagedBeatRequest,
    ) -> Result<SendMessageResponse, AppError> {
        Self::post_adult_stage(conn, "commit", request).await
    }

    pub async fn cancel_adult_stage_via_http(
        conn: &KernelConnection,
        request: &CancelAdultStageGenerationRequest,
    ) -> Result<serde_json::Value, AppError> {
        Self::post_adult_stage(conn, "cancel", request).await
    }

    pub async fn list_adult_staged_beats_via_http(
        conn: &KernelConnection,
        request: &ListAdultStagedBeatsRequest,
    ) -> Result<ListAdultStagedBeatsResponse, AppError> {
        Self::post_adult_stage(conn, "list", request).await
    }

    pub async fn generate_theater_scene_via_http(
        conn: &KernelConnection,
        req: &TheaterSceneRequest,
    ) -> Result<TheaterSceneResponse, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let url = format!("{}/theater/scene", conn.base_url);
        let res = conn
            .http_client()
            .post(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "remote theater scene request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("remote theater scene body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("remote theater scene JSON: {e}")))
    }
}
