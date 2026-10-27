//! Typed identity/relation adapter. A connected desktop never falls back to its UI-only DB.
use super::{app_error_from_http_response, KernelHttpClient};
use crate::error::AppError;
use crate::kernel_lifecycle::KernelConnection;
use oclive_kernel_types::models::dto::{
    ClearSceneUserRelationRequest, GetUserIdentityStateRequest, RoleInfo,
    SetSceneUserIdentityRequest, SetSceneUserRelationRequest, SetUserIdentityRequest,
    SetUserRelationRequest, UserIdentityStateResponse,
};
use serde::de::DeserializeOwned;
use serde_json::json;

impl KernelHttpClient {
    async fn role_state_response<T: DeserializeOwned>(
        conn: &KernelConnection,
        request: reqwest::RequestBuilder,
        operation: &str,
    ) -> Result<T, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let response = request
            .send()
            .await
            .map_err(|error| Self::map_send_err(&conn.base_url, operation, error))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| AppError::OllamaError(format!("{operation} body: {error}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &body));
        }
        serde_json::from_str(&body)
            .map_err(|error| AppError::OllamaError(format!("{operation} JSON: {error}")))
    }

    pub(crate) async fn get_user_identity_state_via_http(
        conn: &KernelConnection,
        req: &GetUserIdentityStateRequest,
    ) -> Result<UserIdentityStateResponse, AppError> {
        let mut request = conn
            .http_client()
            .get(format!("{}/user_identity/state", conn.base_url))
            .query(&[("role_id", req.role_id.as_str())]);
        if let Some(scene) = req
            .scene_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            request = request.query(&[("scene_id", scene)]);
        }
        Self::role_state_response(conn, request, "user_identity/state").await
    }

    pub(crate) async fn set_user_identity_via_http(
        conn: &KernelConnection,
        req: &SetUserIdentityRequest,
    ) -> Result<UserIdentityStateResponse, AppError> {
        let request = conn
            .http_client()
            .post(format!("{}/user_identity/set", conn.base_url))
            .json(&json!({"role_id":req.role_id,"identity_id":req.identity_id}));
        Self::role_state_response(conn, request, "user_identity/set").await
    }

    pub(crate) async fn set_scene_user_identity_via_http(
        conn: &KernelConnection,
        req: &SetSceneUserIdentityRequest,
    ) -> Result<UserIdentityStateResponse, AppError> {
        let request = conn.http_client().post(format!("{}/user_identity/scene_set", conn.base_url))
            .json(&json!({"role_id":req.role_id,"scene_id":req.scene_id,"identity_id":req.identity_id}));
        Self::role_state_response(conn, request, "user_identity/scene_set").await
    }

    pub(crate) async fn set_user_relation_via_http(
        conn: &KernelConnection,
        req: &SetUserRelationRequest,
    ) -> Result<RoleInfo, AppError> {
        let request = conn
            .http_client()
            .post(format!("{}/user_relation/set", conn.base_url))
            .json(&json!({"role_id":req.role_id,"relation":req.relation}));
        Self::role_state_response(conn, request, "user_relation/set").await
    }

    pub(crate) async fn set_scene_user_relation_via_http(
        conn: &KernelConnection,
        req: &SetSceneUserRelationRequest,
    ) -> Result<RoleInfo, AppError> {
        let request = conn
            .http_client()
            .post(format!("{}/user_relation/scene_set", conn.base_url))
            .json(&json!({"role_id":req.role_id,"scene_id":req.scene_id,"relation":req.relation}));
        Self::role_state_response(conn, request, "user_relation/scene_set").await
    }

    pub(crate) async fn clear_scene_user_relation_via_http(
        conn: &KernelConnection,
        req: &ClearSceneUserRelationRequest,
    ) -> Result<RoleInfo, AppError> {
        let request = conn
            .http_client()
            .post(format!("{}/user_relation/scene_clear", conn.base_url))
            .json(&json!({"role_id":req.role_id,"scene_id":req.scene_id}));
        Self::role_state_response(conn, request, "user_relation/scene_clear").await
    }
}

#[cfg(test)]
#[path = "role_state_tests.rs"]
mod tests;
