//! Authenticated desktop streaming transport for ordinary chat turns.
//!
//! **Why this exists.** `distros/shared`'s `sendMessageStream` used to `fetch` `${baseUrl}/chat/stream`
//! straight from the WebView. The Host registers that route inside its token-protected router
//! (`kernel/crates/oclive_kernel_host/src/http_api/mod.rs`) and the desktop always spawns the kernel
//! with `OCLIVE_API_TOKEN`, while the renderer has no way to obtain the token. With a token configured
//! the product's own SSE transport therefore failed with `401 KERNEL_AUTH_REQUIRED` before any turn
//! started (raw evidence: attempt `R1-P2-H`).
//!
//! These commands move the transfer into Rust, where [`KernelConnection::http_client`] already carries
//! `x-oclive-api-token` as a default header, and deliver tokens to the renderer over a Tauri IPC
//! channel. The token never crosses into JavaScript, the Host keeps requiring it, and no route is
//! exempted.
//!
//! **Identity discipline.** The `transport_id` is a renderer-side stream-instance handle (used only to
//! cancel this bridge); it is deliberately *not* the business `client_request_id`. The chat request
//! identity, its scope and its payload still travel inside one [`SendMessageRequest`] shared by the
//! stream and the later same-identity recovery.
//!
//! [`KernelConnection::http_client`]: crate::kernel_lifecycle::KernelConnection::http_client

use crate::api::chat_backend::ChatBackend;
use crate::api::error::CommandError;
use crate::kernel_attach::role_dir_for_id;
use oclive_kernel_host::state::SharedAppState;
use oclive_kernel_types::models::dto::{SendMessageRequest, SendMessageResponse};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};
use tokio::sync::watch;

/// Transport-level terminal code used only by this IPC bridge; it is not a kernel error code.
pub const CHAT_STREAM_CANCELLED: &str = "CHAT_STREAM_CANCELLED";

/// Incremental and terminal events for one streamed turn, delivered in emission order.
///
/// Tokens arrive as the Host emits them; the authoritative DTO is the command's typed return value,
/// which by construction resolves strictly after the last token.
#[derive(serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChatStreamEvent {
    Token { token: String, accumulated: String },
    Canceled { reason: String },
    Failed { code: String, message: String },
}

/// In-flight renderer stream transports.
///
/// Keyed by the renderer's per-stream handle so a cancel never has to match on the business
/// `client_request_id`, and so a late token from a superseded transport cannot be mistaken for the
/// current one.
#[derive(Default)]
pub struct ChatStreamRegistry {
    senders: Mutex<HashMap<String, watch::Sender<bool>>>,
}

impl ChatStreamRegistry {
    /// Register a transport and return the receiver this bridge waits on.
    pub fn register(&self, transport_id: &str) -> watch::Receiver<bool> {
        let (tx, rx) = watch::channel(false);
        self.senders.lock().insert(transport_id.to_string(), tx);
        rx
    }

    /// Forget a transport that has settled.
    pub fn release(&self, transport_id: &str) {
        self.senders.lock().remove(transport_id);
    }

    /// Ask the named transport to stop. Returns whether a live transport was found.
    pub fn cancel(&self, transport_id: &str) -> bool {
        let sender = self.senders.lock().remove(transport_id);
        match sender {
            Some(tx) => {
                let _ = tx.send(true);
                true
            }
            None => false,
        }
    }
}

/// Stream one ordinary chat turn through the token-bearing Rust client.
///
/// # Errors
///
/// Returns the same [`CommandError`] surface as the non-streaming `send_message` command, plus a
/// transport-level cancellation while the renderer has stopped caring about this bridge.
#[tauri::command]
pub async fn send_message_stream(
    req: SendMessageRequest,
    transport_id: String,
    on_event: Channel<ChatStreamEvent>,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<SendMessageResponse, CommandError> {
    let user_message = req.user_message.trim().to_string();
    if user_message.is_empty() {
        return Err(crate::error::AppError::EmptyMessage.into());
    }
    let mut req = req;
    req.user_message = user_message;
    let role_path = role_dir_for_id(state.as_ref(), &req.role_id)?;

    let key = transport_id.trim().to_string();
    let mut cancel_rx = app.state::<ChatStreamRegistry>().register(&key);
    let backend = ChatBackend::from_app(&app, state.inner().clone());

    let accumulated = Arc::new(Mutex::new(String::new()));
    let sink = Arc::clone(&accumulated);
    let token_channel = on_event.clone();

    let outcome = tokio::select! {
        biased;
        // Dropping the stream future closes the loopback request; the Host keeps executing the turn
        // server-side and never starts a second generation for it.
        _ = cancel_rx.changed() => None,
        result = backend.send_message_stream(&role_path, &req, move |token: &str| {
            let text = {
                let mut acc = sink.lock();
                acc.push_str(token);
                acc.clone()
            };
            let _ = token_channel.send(ChatStreamEvent::Token {
                token: token.to_string(),
                accumulated: text,
            });
        }) => Some(result),
    };
    app.state::<ChatStreamRegistry>().release(&key);

    match outcome {
        None => {
            let reason = "the renderer cancelled this stream transport".to_string();
            let _ = on_event.send(ChatStreamEvent::Canceled {
                reason: reason.clone(),
            });
            Err(
                crate::error::AppError::Unknown(format!("{CHAT_STREAM_CANCELLED}: {reason}"))
                    .into(),
            )
        }
        Some(Ok(response)) => Ok(response),
        Some(Err(error)) => {
            let body = error.kernel_error_body();
            let _ = on_event.send(ChatStreamEvent::Failed {
                code: body.code,
                message: body.message,
            });
            Err(error.into())
        }
    }
}

/// Stop one in-flight renderer stream transport. Returns whether it was still live.
///
/// # Errors
///
/// This command has no failure mode of its own; the result type keeps the IPC surface uniform.
#[tauri::command]
pub fn cancel_message_stream(transport_id: String, app: AppHandle) -> Result<bool, CommandError> {
    Ok(app
        .state::<ChatStreamRegistry>()
        .cancel(transport_id.trim()))
}

#[cfg(test)]
mod tests {
    use super::ChatStreamRegistry;

    #[test]
    fn cancel_wakes_only_the_named_transport() {
        let registry = ChatStreamRegistry::default();
        let mut first = registry.register("transport-a");
        let mut second = registry.register("transport-b");

        assert!(!*first.borrow_and_update());
        assert!(registry.cancel("transport-a"));
        // The named transport observes the cancellation; the unrelated one is untouched.
        assert!(*first.borrow_and_update());
        assert!(!*second.borrow_and_update());
        // Cancelling the same transport twice is a no-op the second time.
        assert!(!registry.cancel("transport-a"));
    }

    #[test]
    fn release_removes_the_transport_without_cancelling() {
        let registry = ChatStreamRegistry::default();
        let mut receiver = registry.register("transport-c");
        registry.release("transport-c");

        assert!(!registry.cancel("transport-c"));
        // The value never flipped; the bridge's `changed()` may still resolve because the sender is
        // gone, which is why cancellation is also treated as a terminal signal.
        assert!(!*receiver.borrow_and_update());
    }
}
