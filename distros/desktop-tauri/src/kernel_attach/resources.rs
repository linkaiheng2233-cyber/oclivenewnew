//! Remote role data, resource, and LLM settings HTTP endpoints.

use super::app_error_from_http_response;
use super::KernelHttpClient;
use crate::error::AppError;
use crate::kernel_lifecycle::KernelConnection;
use oclive_kernel_host::infrastructure::chat_storage::{SessionMeta, StoredMessage};
use oclive_kernel_host::service::{
    GlobalOllamaModelDto, ListCloudModelsRequest, LlmUserSettingsDto, SaveLlmUserSettingsRequest,
    SetGlobalOllamaModelRequest,
};
use oclive_kernel_types::models::dto::RoleInfo;
use oclive_kernel_types::models::{
    ActivateLocalLoraAdapterRequest, DeleteLocalLoraAdapterRequest, LocalLoraAdapterDto,
};

use oclive_kernel_types::{ResourceAdapterTransitionRequest, ResourceAdapterTransitionResponse};

use super::RoleSnapshot;

impl KernelHttpClient {
    pub async fn load_role_via_http(
        conn: &KernelConnection,
        role_id: &str,
    ) -> Result<(), AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let url = format!("{}/role/load", conn.base_url);
        let res = conn
            .http_client()
            .post(&url)
            .json(&serde_json::json!({ "role_id": role_id }))
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "role/load request", e))?;
        let status = res.status();
        if status.is_success() {
            Ok(())
        } else {
            let text = res.text().await.unwrap_or_default();
            Err(app_error_from_http_response(status.as_u16(), &text))
        }
    }

    pub async fn role_snapshot_via_http(
        conn: &KernelConnection,
        role_id: &str,
        scene_id: Option<&str>,
    ) -> Result<RoleSnapshot, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let mut req_builder = conn
            .http_client()
            .get(format!("{}/role_snapshot", conn.base_url))
            .query(&[("role_id", role_id)]);
        if let Some(s) = scene_id.filter(|s| !s.is_empty()) {
            req_builder = req_builder.query(&[("scene_id", s)]);
        }
        let res = req_builder
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "role_snapshot request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("role_snapshot body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("role_snapshot JSON: {e}")))
    }

    pub async fn list_chat_sessions_via_http(
        conn: &KernelConnection,
        role_id: &str,
        scene_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<SessionMeta>, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .get(format!("{}/chat/sessions", conn.base_url))
            .query(&[
                ("role_id", role_id),
                ("scene_id", scene_id),
                ("limit", &limit.to_string()),
                ("offset", &offset.to_string()),
            ])
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "chat/sessions request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("chat/sessions body: {e}")))?;
        if !status.is_success() {
            return Err(AppError::OllamaError(format!(
                "chat/sessions HTTP {status}: {text}"
            )));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("chat/sessions JSON: {e}")))
    }

    /// Ask the authoritative kernel process to apply one registered resource
    /// adapter lifecycle transition.
    pub async fn transition_resource_adapter_via_http(
        conn: &KernelConnection,
        request: &ResourceAdapterTransitionRequest,
    ) -> Result<ResourceAdapterTransitionResponse, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let response = conn
            .http_client()
            .post(format!(
                "{}/resources/adapter/transition",
                conn.base_url.trim_end_matches('/')
            ))
            .json(request)
            .send()
            .await
            .map_err(|error| {
                Self::map_send_err(
                    &conn.base_url,
                    "resources/adapter/transition request",
                    error,
                )
            })?;
        let status = response.status();
        let text = response.text().await.map_err(|error| {
            AppError::OllamaError(format!("resources/adapter/transition body: {error}"))
        })?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text).map_err(|error| {
            AppError::OllamaError(format!("resources/adapter/transition JSON: {error}"))
        })
    }

    /// Tell the kernel process to re-read LLM settings from canonical DB.
    pub async fn reload_llm_via_http(conn: &KernelConnection) -> Result<(), AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/llm/reload", conn.base_url))
            .json(&serde_json::json!({}))
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "llm/reload request", e))?;
        let status = res.status();
        if status.is_success() {
            Ok(())
        } else {
            let text = res.text().await.unwrap_or_default();
            Err(app_error_from_http_response(status.as_u16(), &text))
        }
    }

    pub async fn get_llm_user_settings_via_http(
        conn: &KernelConnection,
        role_id: &str,
        session_id: Option<&str>,
    ) -> Result<LlmUserSettingsDto, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let mut req = conn
            .http_client()
            .get(format!("{}/llm/user_settings", conn.base_url))
            .query(&[("role_id", role_id.trim())]);
        if let Some(sid) = session_id.filter(|s| !s.trim().is_empty()) {
            req = req.query(&[("session_id", sid.trim())]);
        }
        let res = req
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "llm/user_settings GET", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("llm/user_settings body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("llm/user_settings JSON: {e}")))
    }

    pub async fn save_llm_user_settings_via_http(
        conn: &KernelConnection,
        req: &SaveLlmUserSettingsRequest,
    ) -> Result<RoleInfo, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/llm/user_settings", conn.base_url))
            .json(req)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "llm/user_settings POST", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("llm/user_settings POST body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("llm/user_settings POST JSON: {e}")))
    }

    pub async fn activate_local_lora_adapter_via_http(
        conn: &KernelConnection,
        req: &ActivateLocalLoraAdapterRequest,
    ) -> Result<Option<LocalLoraAdapterDto>, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/llm/lora/activate", conn.base_url))
            .json(req)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "llm/lora/activate POST", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("llm/lora/activate body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("llm/lora/activate JSON: {e}")))
    }

    pub async fn delete_local_lora_adapter_via_http(
        conn: &KernelConnection,
        req: &DeleteLocalLoraAdapterRequest,
    ) -> Result<(), AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/llm/lora/delete", conn.base_url))
            .json(req)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "llm/lora/delete POST", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("llm/lora/delete body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        Ok(())
    }

    pub async fn get_global_ollama_model_via_http(
        conn: &KernelConnection,
    ) -> Result<GlobalOllamaModelDto, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .get(format!("{}/llm/global_ollama_model", conn.base_url))
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "llm/global_ollama_model GET", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("llm/global_ollama_model body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("llm/global_ollama_model JSON: {e}")))
    }

    pub async fn set_global_ollama_model_via_http(
        conn: &KernelConnection,
        req: &SetGlobalOllamaModelRequest,
    ) -> Result<GlobalOllamaModelDto, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/llm/global_ollama_model", conn.base_url))
            .json(req)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "llm/global_ollama_model POST", e))?;
        let status = res.status();
        let text = res.text().await.map_err(|e| {
            AppError::OllamaError(format!("llm/global_ollama_model POST body: {e}"))
        })?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("llm/global_ollama_model POST JSON: {e}")))
    }

    pub async fn probe_cloud_llm_via_http(
        conn: &KernelConnection,
        role_id: &str,
        session_id: Option<&str>,
    ) -> Result<(), AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let mut req = conn
            .http_client()
            .post(format!("{}/llm/probe_cloud", conn.base_url))
            .query(&[("role_id", role_id.trim())]);
        if let Some(sid) = session_id.filter(|s| !s.trim().is_empty()) {
            req = req.query(&[("session_id", sid.trim())]);
        }
        let res = req
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "llm/probe_cloud POST", e))?;
        let status = res.status();
        if status.is_success() {
            Ok(())
        } else {
            let text = res.text().await.unwrap_or_default();
            Err(app_error_from_http_response(status.as_u16(), &text))
        }
    }

    pub async fn list_cloud_models_via_http(
        conn: &KernelConnection,
        remote_url: Option<&str>,
        remote_token: Option<&str>,
    ) -> Result<Vec<String>, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let body = ListCloudModelsRequest {
            remote_url: remote_url.map(str::to_string),
            remote_token: remote_token.map(str::to_string),
        };
        let res = conn
            .http_client()
            .post(format!("{}/llm/cloud_models", conn.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "llm/cloud_models POST", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("llm/cloud_models body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("llm/cloud_models JSON: {e}")))
    }

    pub async fn fetch_chat_messages_via_http(
        conn: &KernelConnection,
        session_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<StoredMessage>, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .get(format!("{}/chat/messages", conn.base_url))
            .query(&[
                ("session_id", session_id),
                ("limit", &limit.to_string()),
                ("offset", &offset.to_string()),
            ])
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "chat/messages request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("chat/messages body: {e}")))?;
        if !status.is_success() {
            return Err(AppError::OllamaError(format!(
                "chat/messages HTTP {status}: {text}"
            )));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("chat/messages JSON: {e}")))
    }

    pub async fn chat_storage_proxy_via_http(
        conn: &KernelConnection,
        op: &oclive_kernel_host::service::ChatStorageProxyOp,
    ) -> Result<serde_json::Value, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/chat/storage", conn.base_url))
            .json(op)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "chat/storage request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("chat/storage body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("chat/storage JSON: {e}")))
    }
}
