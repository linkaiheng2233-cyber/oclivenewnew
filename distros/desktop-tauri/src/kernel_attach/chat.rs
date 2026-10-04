//! Remote chat and adult-stage HTTP endpoints.

use super::app_error_from_http_response;
use super::KernelHttpClient;
use crate::error::AppError;
use crate::kernel_lifecycle::KernelConnection;
use oclive_kernel_types::models::dto::{
    AdultStagedBeatDto, BeginAdultStageGenerationRequest, BeginAdultStageGenerationResponse,
    CancelAdultStageGenerationRequest, CommitAdultStagedBeatRequest, ListAdultStagedBeatsRequest,
    ListAdultStagedBeatsResponse, MinimalRoleLocalMessageRequest, MinimalRoleMessageResponse,
    SendMessageRequest, SendMessageResponse, StageAdultBeatRequest, TheaterSceneRequest,
    TheaterSceneResponse,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use std::path::Path;

use super::parse_sse_block;
use futures_util::StreamExt;

/// Upper bound for one un-delimited SSE frame; protects the decoder from an endless partial block.
const MAX_SSE_BLOCK_BYTES: usize = 4 * 1024 * 1024;

#[cfg(test)]
mod minimal_transport_tests {
    use oclive_kernel_types::models::dto::{
        MinimalRoleLocalMessageRequest, MinimalRoleMessageResponse,
        MinimalRoleProductExtensionStatus,
    };
    use serde_json::json;

    #[test]
    fn renderer_source_and_basic_result_use_the_shared_types_without_rich_defaults() {
        let body = json!({
            "source": {"role_id":"converter-owned-id", "asset_root":"E:/fixture-assets",
                "definition_reference":"chosen-content.json"},
            "message": {"user_message":"  current material\r\n", "requirements":""}
        });
        let request: MinimalRoleLocalMessageRequest = serde_json::from_value(body.clone()).unwrap();
        assert_eq!(serde_json::to_value(&request).unwrap(), body);
        let wire = json!({"role_id":request.source.role_id,"reply":"",
            "product_extensions":"unavailable"});
        let response: MinimalRoleMessageResponse = serde_json::from_value(wire.clone()).unwrap();
        assert!(response.reply.is_empty());
        assert_eq!(
            response.product_extensions,
            MinimalRoleProductExtensionStatus::Unavailable
        );
        assert_eq!(serde_json::to_value(response).unwrap(), wire);
        let rich = json!({"reply":"pretend rich success","favorability_current":0});
        assert!(serde_json::from_value::<MinimalRoleMessageResponse>(rich).is_err());
    }

    #[test]
    fn minimum_rejections_use_existing_auth_input_and_model_error_mapping() {
        for (status, code) in [
            (401, "KERNEL_AUTH_REQUIRED"),
            (400, "INVALID_PARAMETER"),
            (500, "LLM_ERROR"),
        ] {
            let wire = json!({"error":{"code":code,"message":"synthetic failure"}}).to_string();
            assert_eq!(
                super::app_error_from_http_response(status, &wire).code(),
                code
            );
        }
    }
}

/// First index of `needle` inside `haystack`, byte-wise (the delimiter is ASCII, so byte search is
/// exactly what a UTF-8 stream needs).
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Incremental SSE decoder for `POST /chat/stream`.
///
/// Kept separate from the transport so the byte-level contract is unit-testable: a chunk may split a
/// multi-byte UTF-8 character **and** the `\n\n` block delimiter, tokens must be emitted in wire
/// order, the terminal `error` must win over any partial prefix, and a stream that never delivers
/// `done` is a failure rather than a partial success.
#[derive(Default)]
pub(crate) struct ChatStreamDecoder {
    pending: Vec<u8>,
    done: Option<SendMessageResponse>,
    done_unusable: bool,
    failure: Option<AppError>,
}

#[derive(Deserialize)]
struct StreamDoneEnvelope {
    data: SendMessageResponse,
}

impl ChatStreamDecoder {
    /// Feed one HTTP chunk; complete blocks are consumed and tokens emitted immediately.
    ///
    /// # Errors
    ///
    /// Returns the mapped kernel error as soon as the wire carries an `error` block, and rejects an
    /// unbounded frame that never terminates with a block delimiter.
    pub(crate) fn push(
        &mut self,
        chunk: &[u8],
        on_token: &mut impl FnMut(&str),
    ) -> Result<(), AppError> {
        if let Some(error) = self.failure.take() {
            return Err(error);
        }
        self.pending.extend_from_slice(chunk);
        let mut consumed = 0usize;
        while let Some(relative) = find_subsequence(&self.pending[consumed..], b"\n\n") {
            let end = consumed + relative;
            let block = String::from_utf8_lossy(&self.pending[consumed..end]).into_owned();
            consumed = end + 2;
            let (event_name, data) = parse_sse_block(&block);
            if data.is_empty() {
                continue;
            }
            match event_name.as_str() {
                "token" => {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&data) {
                        if let Some(token) = value.get("token").and_then(|x| x.as_str()) {
                            on_token(token);
                        }
                    }
                }
                "done" => {
                    let parsed = serde_json::from_str::<StreamDoneEnvelope>(&data)
                        .ok()
                        .map(|envelope| envelope.data)
                        .or_else(|| serde_json::from_str::<SendMessageResponse>(&data).ok());
                    match parsed {
                        Some(response) => self.done = Some(response),
                        None => self.done_unusable = true,
                    }
                }
                "error" => {
                    self.failure = Some(app_error_from_http_response(200, &data));
                }
                _ => {}
            }
        }
        if consumed > 0 {
            self.pending.drain(..consumed);
        }
        if let Some(error) = self.failure.take() {
            return Err(error);
        }
        if self.pending.len() > MAX_SSE_BLOCK_BYTES {
            return Err(AppError::OllamaError(
                "remote chat stream sent an unbounded frame without a block delimiter".into(),
            ));
        }
        Ok(())
    }

    /// The authoritative response, or the error a truncated/failed stream must surface.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::OllamaError`] when the stream ended before any `done` block, or when the
    /// `done` block it did send could not be decoded into the typed DTO.
    pub(crate) fn finish(self) -> Result<SendMessageResponse, AppError> {
        match self.done {
            Some(response) => Ok(response),
            None if self.done_unusable => Err(AppError::OllamaError(
                "remote chat stream sent a done payload that is not a valid response".into(),
            )),
            None => Err(AppError::OllamaError(
                "remote chat stream ended without done event".into(),
            )),
        }
    }
}

impl KernelHttpClient {
    /// Body for `POST /chat` and `POST /chat/stream`, which the host accepts as its
    /// `ChatApiRequest` (`role_path` + `message`).
    ///
    /// Both routes belong to one logical turn, so the identity and every semantic field are
    /// built in exactly one place: a second copy could silently drop the shared turn id.
    #[must_use]
    pub fn chat_api_body(role_path: &Path, req: &SendMessageRequest) -> serde_json::Value {
        serde_json::json!({
            "client_request_id": req.client_request_id,
            "role_path": role_path.to_string_lossy(),
            "message": req.user_message,
            "session_id": req.session_id,
            "scene_id": req.scene_id,
            "include_raw_reply": req.include_raw_reply,
            "adult": req.adult,
        })
    }

    /// Body for `POST /chat/recover`, which the host accepts as a bare [`SendMessageRequest`]
    /// (`role_id`, *not* `role_path`). It carries the same `client_request_id` as
    /// [`Self::chat_api_body`], so recovery addresses the turn the transport already started.
    ///
    /// # Errors
    /// Returns [`AppError::ChatRequestUnconfirmed`] when the request cannot be serialized.
    pub fn chat_recover_body(req: &SendMessageRequest) -> Result<serde_json::Value, AppError> {
        serde_json::to_value(req).map_err(|_| AppError::ChatRequestUnconfirmed)
    }

    /// Map a `/chat/recover` HTTP response onto the only two outcomes recovery may have.
    ///
    /// A recovery failure must never become a new turn, so anything that is not an
    /// identifiable payload conflict collapses into [`AppError::ChatRequestUnconfirmed`]
    /// (missing receipt, old kernel without the route, transport or decode failure).
    ///
    /// # Errors
    /// Returns [`AppError::ChatRequestConflict`] for a reused identity with a different
    /// payload, and [`AppError::ChatRequestUnconfirmed`] for every other outcome.
    pub fn map_recover_response(status: u16, body: &str) -> Result<SendMessageResponse, AppError> {
        if (200..300).contains(&status) {
            return serde_json::from_str(body).map_err(|_| AppError::ChatRequestUnconfirmed);
        }
        Err(match app_error_from_http_response(status, body) {
            AppError::ChatRequestConflict => AppError::ChatRequestConflict,
            _ => AppError::ChatRequestUnconfirmed,
        })
    }

    /// Recovery never falls back to `/chat`, including on an old kernel's 404.
    pub async fn recover_message_via_http(
        conn: &KernelConnection,
        req: &SendMessageRequest,
    ) -> Result<SendMessageResponse, AppError> {
        let body = Self::chat_recover_body(req)?;
        let result = conn
            .http_client()
            .post(format!("{}/chat/recover", conn.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|_| AppError::ChatRequestUnconfirmed)?;
        let status = result.status().as_u16();
        let text = result
            .text()
            .await
            .map_err(|_| AppError::ChatRequestUnconfirmed)?;
        Self::map_recover_response(status, &text)
    }

    pub async fn send_message_via_http(
        conn: &KernelConnection,
        role_path: &Path,
        req: &SendMessageRequest,
    ) -> Result<SendMessageResponse, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let url = format!("{}/chat", conn.base_url);
        let body = Self::chat_api_body(role_path, req);
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
        let body = Self::chat_api_body(role_path, req);
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

        let mut decoder = ChatStreamDecoder::default();
        let mut byte_stream = res.bytes_stream();
        while let Some(chunk) = byte_stream.next().await {
            let chunk = chunk
                .map_err(|e| AppError::OllamaError(format!("remote chat stream chunk: {e}")))?;
            decoder.push(&chunk, &mut on_token)?;
        }
        decoder.finish()
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
        Self::post_chat_json(
            conn,
            &format!("/chat/adult-stage/{route}"),
            "adult stage",
            request,
        )
        .await
    }

    pub async fn send_minimal_message_via_http(
        conn: &KernelConnection,
        request: &MinimalRoleLocalMessageRequest,
    ) -> Result<MinimalRoleMessageResponse, AppError> {
        Self::post_chat_json(conn, "/chat/minimal", "minimal text", request).await
    }

    /// All these JSON chat operations use the token-bearing Rust client. Sharing
    /// transport does not share their distinct DTOs, activation or retry policy.
    async fn post_chat_json<Req, Res>(
        conn: &KernelConnection,
        route: &str,
        operation: &str,
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
            .post(format!("{}{route}", conn.base_url.trim_end_matches('/')))
            .json(request)
            .send()
            .await
            .map_err(|error| {
                Self::map_send_err(&conn.base_url, &format!("{operation} request"), error)
            })?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|error| AppError::OllamaError(format!("{operation} body: {error}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|error| AppError::OllamaError(format!("{operation} JSON: {error}")))
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

#[cfg(test)]
mod stream_decoder_tests {
    use super::ChatStreamDecoder;

    fn token_block(token: &str) -> String {
        format!(
            "event: token\ndata: {}\n\n",
            serde_json::json!({ "token": token })
        )
    }

    fn done_block(reply: &str) -> String {
        // Every field without `#[serde(default)]` on `SendMessageResponse` must be present, otherwise
        // the decoder would legitimately reject the payload as unusable.
        let payload = serde_json::json!({
            "data": {
                "api_version": 1,
                "schema": 16,
                "presence_mode": "co_present",
                "relation_state": "Friend",
                "reply": reply,
                "emotion": {
                    "joy": 0.0, "sadness": 0.0, "anger": 0.0, "fear": 0.0,
                    "surprise": 0.0, "disgust": 0.0, "neutral": 1.0
                },
                "bot_emotion": "neutral",
                "portrait_emotion": "neutral",
                "favorability_delta": 0.0,
                "favorability_current": 50.0,
                "events": [],
                "scene_id": "home",
                "offer_destination_picker": false,
                "timestamp": 1,
                "user_message_id": "u1",
                "assistant_message_id": "a1"
            }
        });
        format!("event: done\ndata: {payload}\n\n")
    }

    fn error_block(code: &str) -> String {
        format!(
            "event: error\ndata: {}\n\n",
            serde_json::json!({ "error": { "code": code, "message": "synthetic" } })
        )
    }

    fn drive(
        chunks: &[&[u8]],
    ) -> Result<(Vec<String>, super::SendMessageResponse), crate::error::AppError> {
        let mut decoder = ChatStreamDecoder::default();
        let mut tokens: Vec<String> = Vec::new();
        for chunk in chunks {
            decoder.push(chunk, &mut |token: &str| tokens.push(token.to_string()))?;
        }
        Ok((tokens, decoder.finish()?))
    }

    #[test]
    fn decodes_tokens_in_wire_order_before_the_terminal_dto() {
        let wire = format!(
            "{}{}{}",
            token_block("你"),
            token_block("好"),
            done_block("你好")
        );
        let (tokens, response) = drive(&[wire.as_bytes()]).expect("stream decodes");
        assert_eq!(tokens, vec!["你".to_string(), "好".to_string()]);
        assert_eq!(response.reply, "你好");
        assert_eq!(response.assistant_message_id.as_deref(), Some("a1"));
    }

    #[test]
    fn survives_a_chunk_split_inside_a_multibyte_character() {
        let wire = format!("{}{}", token_block("清晨"), done_block("清晨"));
        let bytes = wire.as_bytes();
        // Split inside the first multi-byte character (after `event: token\ndata: {"token":"` + 1 byte).
        let split = wire.find('清').expect("token present") + 1;
        assert!(split < bytes.len());
        let (tokens, response) =
            drive(&[&bytes[..split], &bytes[split..]]).expect("stream decodes");
        assert_eq!(tokens, vec!["清晨".to_string()]);
        assert_eq!(response.reply, "清晨");
    }

    #[test]
    fn survives_a_chunk_split_inside_the_block_delimiter() {
        let wire = format!("{}{}", token_block("风"), done_block("风"));
        let bytes = wire.as_bytes();
        let split = wire.find("\n\n").expect("delimiter present") + 1;
        let (tokens, response) =
            drive(&[&bytes[..split], &bytes[split..]]).expect("stream decodes");
        assert_eq!(tokens, vec!["风".to_string()]);
        assert_eq!(response.reply, "风");
    }

    #[test]
    fn decodes_one_byte_at_a_time() {
        let wire = format!("{}{}", token_block("树梢"), done_block("树梢"));
        let bytes = wire.as_bytes();
        let chunks: Vec<&[u8]> = bytes.chunks(1).collect();
        let (tokens, response) = drive(&chunks).expect("stream decodes");
        assert_eq!(tokens, vec!["树梢".to_string()]);
        assert_eq!(response.reply, "树梢");
    }

    #[test]
    fn structured_error_after_a_token_is_a_failure_not_a_partial_reply() {
        let wire = format!("{}{}", token_block("部分"), error_block("LLM_ERROR"));
        let error = drive(&[wire.as_bytes()]).expect_err("error block must fail the stream");
        assert_eq!(error.code(), "LLM_ERROR");
    }

    #[test]
    fn stream_without_done_is_a_failure() {
        let wire = token_block("部分");
        let error = drive(&[wire.as_bytes()]).expect_err("missing done must fail the stream");
        assert_eq!(error.code(), "LLM_ERROR");
        assert!(error.to_string().contains("ended without done event"));
    }

    #[test]
    fn unusable_done_payload_is_distinguished_from_a_missing_one() {
        let wire = format!(
            "{}{}",
            token_block("部分"),
            "event: done\ndata: {\"data\":{\"reply\":\"x\"}}\n\n"
        );
        let error = drive(&[wire.as_bytes()]).expect_err("partial DTO must fail the stream");
        assert_eq!(error.code(), "LLM_ERROR");
        assert!(error.to_string().contains("not a valid response"));
    }

    #[test]
    fn unterminated_frame_is_rejected_instead_of_buffering_forever() {
        let mut decoder = ChatStreamDecoder::default();
        let huge = vec![b'x'; super::MAX_SSE_BLOCK_BYTES + 1];
        let error = decoder
            .push(&huge, &mut |_: &str| {})
            .expect_err("unbounded frame must fail");
        assert_eq!(error.code(), "LLM_ERROR");
    }
}
