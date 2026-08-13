//! Remote theater, role, and scene HTTP endpoints.

use super::app_error_from_http_response;
use super::KernelHttpClient;
use crate::error::AppError;
use crate::kernel_lifecycle::KernelConnection;

use oclive_kernel_types::models::dto::{
    CreateEventRequest, CreateEventResponse, DisplayMetricsDto, GetDisplayMetricsRequest,
    GetRoleInfoRequest, JumpTimeRequest, JumpTimeResponse, RoleInfo, SetRoleInteractionModeRequest,
    SetUserPresenceSceneRequest, SwitchSceneRequest, SwitchSceneResponse, TimeStateResponse,
};

impl KernelHttpClient {
    pub async fn set_role_interaction_mode_via_http(
        conn: &KernelConnection,
        req: &SetRoleInteractionModeRequest,
    ) -> Result<RoleInfo, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/role/interaction_mode", conn.base_url))
            .json(req)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "role/interaction_mode request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("role/interaction_mode body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("role/interaction_mode JSON: {e}")))
    }

    pub async fn get_role_info_via_http(
        conn: &KernelConnection,
        req: &GetRoleInfoRequest,
    ) -> Result<RoleInfo, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let mut req_builder = conn
            .http_client()
            .get(format!("{}/role_info", conn.base_url))
            .query(&[("role_id", req.role_id.as_str())]);
        if let Some(sid) = req.session_id.as_deref().filter(|s| !s.is_empty()) {
            req_builder = req_builder.query(&[("session_id", sid)]);
        }
        let res = req_builder
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "role_info request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("role_info body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("role_info JSON: {e}")))
    }

    pub async fn get_display_metrics_via_http(
        conn: &KernelConnection,
        req: &GetDisplayMetricsRequest,
    ) -> Result<DisplayMetricsDto, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let mut req_builder = conn
            .http_client()
            .get(format!("{}/display_metrics", conn.base_url))
            .query(&[("role_id", req.role_id.as_str())]);
        if let Some(sid) = req.session_id.as_deref().filter(|s| !s.is_empty()) {
            req_builder = req_builder.query(&[("session_id", sid)]);
        }
        let res = req_builder
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "display_metrics request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("display_metrics body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("display_metrics JSON: {e}")))
    }

    pub async fn get_time_state_via_http(
        conn: &KernelConnection,
        role_id: &str,
    ) -> Result<TimeStateResponse, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .get(format!("{}/time/state", conn.base_url))
            .query(&[("role_id", role_id.trim())])
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "time/state request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("time/state body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("time/state JSON: {e}")))
    }

    pub async fn jump_time_via_http(
        conn: &KernelConnection,
        req: &JumpTimeRequest,
    ) -> Result<JumpTimeResponse, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/time/jump", conn.base_url))
            .json(req)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "time/jump request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("time/jump body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("time/jump JSON: {e}")))
    }

    pub async fn switch_scene_via_http(
        conn: &KernelConnection,
        req: &SwitchSceneRequest,
    ) -> Result<SwitchSceneResponse, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/scene/switch", conn.base_url))
            .json(req)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "scene/switch request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("scene/switch body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("scene/switch JSON: {e}")))
    }

    pub async fn set_user_presence_scene_via_http(
        conn: &KernelConnection,
        req: &SetUserPresenceSceneRequest,
    ) -> Result<RoleInfo, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/scene/user_presence", conn.base_url))
            .json(req)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "scene/user_presence request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("scene/user_presence body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("scene/user_presence JSON: {e}")))
    }

    pub async fn create_event_via_http(
        conn: &KernelConnection,
        req: &CreateEventRequest,
    ) -> Result<CreateEventResponse, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/event/create", conn.base_url))
            .json(req)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "event/create request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("event/create body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("event/create JSON: {e}")))
    }

    pub async fn bridge_dispatch_via_http(
        conn: &KernelConnection,
        command: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/bridge/dispatch", conn.base_url))
            .json(&serde_json::json!({
                "command": command,
                "params": params,
            }))
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "bridge/dispatch request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("bridge/dispatch body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("bridge/dispatch JSON: {e}")))
    }
}
