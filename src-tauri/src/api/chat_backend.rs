//! Chat command backend: HTTP kernel attach vs in-process [`ConversationStore`].
//!
//! **Desktop production** always routes through [`ChatBackend::Http`] (loopback kernel is the
//! single authoritative writer). [`ChatBackend::Local`] and the in-memory [`AppState`] shadow
//! exist for tests and alternate hosts only — they must not be treated as authoritative writers.

use crate::error::AppError;
use crate::infrastructure::chat_storage::{SessionMeta, StoredMessage};
use crate::kernel_attach::KernelHttpClient;
use crate::kernel_lifecycle::SharedKernelConnection;
use crate::models::dto::{SendMessageRequest, SendMessageResponse};
use crate::state::AppState;
use oclive_kernel_host::service::{execute_chat_storage_proxy, ChatStorageProxyOp};
use std::sync::Arc;
use tauri::{AppHandle, Manager};

pub enum ChatBackend {
    Http(SharedKernelConnection),
    /// In-process shadow store — **non-authoritative** on desktop; kernel HTTP owns writes.
    Local(Arc<AppState>),
}

impl ChatBackend {
    #[must_use]
    pub fn from_app(app: &AppHandle, state: Arc<AppState>) -> Self {
        if let Some(conn) = app.try_state::<SharedKernelConnection>() {
            Self::Http(Arc::clone(&conn))
        } else {
            // Desktop `.setup` always registers `SharedKernelConnection`; Local is test-only.
            debug_assert!(
                cfg!(test),
                "desktop shell should use Http backend; Local AppState is not an authoritative writer"
            );
            Self::Local(state)
        }
    }

    pub async fn send_message(
        &self,
        role_path: &std::path::Path,
        req: &SendMessageRequest,
    ) -> Result<SendMessageResponse, AppError> {
        match self {
            Self::Http(conn) => {
                match KernelHttpClient::send_message_via_http(conn, role_path, req).await {
                    Ok(res) => Ok(res),
                    Err(AppError::RoleRuntimeNotReady) => {
                        KernelHttpClient::load_role_via_http(conn, req.role_id.trim()).await?;
                        KernelHttpClient::send_message_via_http(conn, role_path, req).await
                    }
                    Err(e) => Err(e),
                }
            }
            Self::Local(state) => {
                debug_assert!(
                    cfg!(test),
                    "Local chat backend must not be used for authoritative desktop writes"
                );
                crate::domain::chat_engine::process_message(state.as_ref(), req).await
            }
        }
    }

    pub async fn list_chat_sessions(
        &self,
        role_id: &str,
        scene_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<SessionMeta>, AppError> {
        match self {
            Self::Http(conn) => {
                KernelHttpClient::list_chat_sessions_via_http(
                    conn, role_id, scene_id, limit, offset,
                )
                .await
            }
            Self::Local(state) => {
                state
                    .conversation_store
                    .list_sessions(role_id, scene_id, limit, offset)
                    .await
            }
        }
    }

    pub async fn fetch_chat_messages(
        &self,
        session_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<StoredMessage>, AppError> {
        match self {
            Self::Http(conn) => {
                KernelHttpClient::fetch_chat_messages_via_http(conn, session_id, limit, offset)
                    .await
            }
            Self::Local(state) => {
                state
                    .conversation_store
                    .fetch_messages(session_id, limit, offset)
                    .await
            }
        }
    }

    /// Run a chat-storage admin op on the kernel HTTP writer or local store.
    pub async fn storage_proxy(
        &self,
        op: ChatStorageProxyOp,
    ) -> Result<serde_json::Value, AppError> {
        match self {
            Self::Http(conn) => KernelHttpClient::chat_storage_proxy_via_http(conn, &op).await,
            Self::Local(state) => execute_chat_storage_proxy(state.as_ref(), op).await,
        }
    }
}
