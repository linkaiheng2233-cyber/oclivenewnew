//! Explicit user cancellation. Transport detach must not call this implicitly.

use crate::error::{AppError, Result};
use crate::models::dto::{CancelChatRequest, SendMessageRequest};
use crate::state::AppState;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::watch;

const ACTIVE: u8 = 0;
const STOPPED: u8 = 1;
const COMMITTING: u8 = 2;
const MAX_PENDING: usize = 1024;
const INTENT_TTL: Duration = Duration::from_secs(60);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct Key {
    role: String,
    session: Option<String>,
    thread: Option<String>,
    request: String,
}

impl Key {
    fn new(role: &str, session: Option<&str>, thread: Option<&str>, request: &str) -> Result<Self> {
        super::request_receipt::validate_client_request_id(request)?;
        if role.is_empty() || role.len() > 256 || session.is_some_and(|s| s.len() > 256) {
            return Err(AppError::InvalidParameter(
                "invalid cancellation namespace".into(),
            ));
        }
        if let Some(thread) = thread {
            super::chat_thread::validate_id(thread)?;
        }
        Ok(Self {
            role: role.to_owned(),
            session: session.map(str::to_owned),
            thread: thread.map(str::to_owned),
            request: request.to_owned(),
        })
    }
}

pub(crate) struct TurnCancellation {
    phase: AtomicU8,
    stopped: watch::Sender<bool>,
    visible: Mutex<String>,
}

impl Default for TurnCancellation {
    fn default() -> Self {
        Self {
            phase: AtomicU8::new(ACTIVE),
            stopped: watch::channel(false).0,
            visible: Mutex::new(String::new()),
        }
    }
}

impl TurnCancellation {
    fn stop(&self) -> bool {
        // Serialize the last accepted token with the stop boundary.
        let _text = self.visible.lock();
        match self
            .phase
            .compare_exchange(ACTIVE, STOPPED, Ordering::SeqCst, Ordering::SeqCst)
        {
            Ok(_) => {
                self.stopped.send_replace(true);
                true
            }
            Err(STOPPED) => true,
            Err(_) => false,
        }
    }

    /// Once persistence begins, late stop requests cannot interrupt a commit.
    pub(crate) fn begin_commit(&self) -> bool {
        self.phase
            .compare_exchange(ACTIVE, COMMITTING, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    pub(crate) async fn cancelled(&self) {
        let mut receiver = self.stopped.subscribe();
        if *receiver.borrow_and_update() {
            return;
        }
        while receiver.changed().await.is_ok() {
            if *receiver.borrow_and_update() {
                return;
            }
        }
    }

    pub(crate) fn capture(
        self: &Arc<Self>,
        downstream: oclive_kernel_contracts::LlmTokenSink,
    ) -> oclive_kernel_contracts::LlmTokenSink {
        let control = Arc::clone(self);
        Arc::new(move |token| {
            {
                let mut text = control.visible.lock();
                if control.phase.load(Ordering::SeqCst) != ACTIVE {
                    return;
                }
                text.push_str(token);
            }
            downstream(token);
        })
    }

    pub(crate) fn visible(&self) -> String {
        self.visible.lock().clone()
    }
}

struct Entry {
    control: Arc<TurnCancellation>,
    registered: bool,
    created: Instant,
}

#[derive(Default)]
pub(crate) struct ChatCancellations(Mutex<HashMap<Key, Entry>>);

impl ChatCancellations {
    pub(crate) fn register(&self, request: &SendMessageRequest) -> Result<TurnRegistration<'_>> {
        let key = request
            .client_request_id
            .as_deref()
            .map(|id| {
                Key::new(
                    &request.role_id,
                    request.session_id.as_deref(),
                    request.chat_thread_id.as_deref(),
                    id,
                )
            })
            .transpose()?;
        let control = if let Some(key) = key.as_ref() {
            let mut entries = self.0.lock();
            Self::prune(&mut entries);
            let entry = entries.entry(key.clone()).or_insert_with(|| Entry {
                control: Arc::new(TurnCancellation::default()),
                registered: false,
                created: Instant::now(),
            });
            if entry.registered {
                return Err(AppError::InvalidParameter(
                    "chat turn is already registered".into(),
                ));
            }
            entry.registered = true;
            Arc::clone(&entry.control)
        } else {
            Arc::new(TurnCancellation::default())
        };
        Ok(TurnRegistration {
            registry: self,
            key,
            control,
        })
    }

    fn prune(entries: &mut HashMap<Key, Entry>) {
        entries.retain(|_, entry| entry.registered || entry.created.elapsed() < INTENT_TTL);
    }

    fn cancel(&self, key: Key) -> Result<bool> {
        let mut entries = self.0.lock();
        Self::prune(&mut entries);
        if let Some(entry) = entries.get(&key) {
            return Ok(entry.control.stop());
        }
        // A stop may reach the server before the original send. Bound these intents;
        // never evict an active generation to make room for an unknown request.
        if entries.values().filter(|entry| !entry.registered).count() >= MAX_PENDING {
            return Err(AppError::InvalidParameter(
                "too many pending cancellation requests".into(),
            ));
        }
        let control = Arc::new(TurnCancellation::default());
        control.stop();
        entries.insert(
            key,
            Entry {
                control,
                registered: false,
                created: Instant::now(),
            },
        );
        Ok(true)
    }
}

pub(crate) struct TurnRegistration<'a> {
    registry: &'a ChatCancellations,
    key: Option<Key>,
    pub(crate) control: Arc<TurnCancellation>,
}

impl Drop for TurnRegistration<'_> {
    fn drop(&mut self) {
        if let Some(key) = self.key.as_ref() {
            let mut entries = self.registry.0.lock();
            if entries
                .get(key)
                .is_some_and(|e| Arc::ptr_eq(&e.control, &self.control))
            {
                entries.remove(key);
            }
        }
    }
}

/// Request cancellation of one matching user turn. The send transport stays open
/// until its partial reply and stopped flag are persisted and returned normally.
/// `false` means that turn has already entered its indivisible commit boundary.
///
/// # Errors
/// Returns validation errors for a noncanonical UUID, invalid namespace, or a
/// saturated early-intent buffer. Previously executed tools are not undone.
pub fn cancel_message(state: &AppState, request: &CancelChatRequest) -> Result<bool> {
    let key = Key::new(
        &request.role_id,
        request.session_id.as_deref(),
        request.chat_thread_id.as_deref(),
        &request.client_request_id,
    )?;
    state.chat_cancellations.cancel(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> SendMessageRequest {
        SendMessageRequest {
            role_id: "mumu".into(),
            client_request_id: Some(uuid::Uuid::new_v4().to_string()),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn early_stop_is_scoped_and_late_tokens_are_ignored() {
        let registry = ChatCancellations::default();
        let req = request();
        let key = Key::new(
            &req.role_id,
            None,
            None,
            req.client_request_id.as_deref().unwrap(),
        )
        .unwrap();
        registry.cancel(key).unwrap();
        let registration = registry.register(&req).unwrap();
        tokio::time::timeout(Duration::from_millis(100), registration.control.cancelled())
            .await
            .unwrap();
        let sink = registration
            .control
            .capture(Arc::new(|_| panic!("late token must not escape")));
        sink("late");
        assert_eq!(registration.control.visible(), "");
        assert!(!registration.control.begin_commit());
        let other = SendMessageRequest {
            role_id: "other".into(),
            ..req
        };
        assert!(registry.register(&other).unwrap().control.begin_commit());
        drop(registration);
        assert!(registry.0.lock().is_empty());
    }

    #[test]
    fn late_stop_cannot_interrupt_persistence_and_registration_is_released() {
        let registry = ChatCancellations::default();
        let req = request();
        let registration = registry.register(&req).unwrap();
        assert!(registration.control.begin_commit());
        assert!(!registration.control.stop());
        drop(registration);
        assert!(registry.0.lock().is_empty());
    }

    #[test]
    fn pending_intents_are_bounded_without_evicting_active_turns() {
        let registry = ChatCancellations::default();
        let active = request();
        let _registration = registry.register(&active).unwrap();
        for _ in 0..MAX_PENDING {
            registry
                .cancel(Key::new("mumu", None, None, &uuid::Uuid::new_v4().to_string()).unwrap())
                .unwrap();
        }
        assert!(registry
            .cancel(Key::new("mumu", None, None, &uuid::Uuid::new_v4().to_string()).unwrap())
            .is_err());
        assert!(!registry.0.lock().is_empty());
        let mut entries = registry.0.lock();
        for entry in entries.values_mut().filter(|e| !e.registered) {
            entry.created = Instant::now() - INTENT_TTL;
        }
        ChatCancellations::prune(&mut entries);
        assert_eq!(entries.len(), 1);
    }
}
