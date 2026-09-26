//! Default-off, compile-time gated fault injection for the CP-INT C3 receipt window.
//!
//! This module exists **only** when the `test-faults` cargo feature is enabled. It is not part of
//! `default`, so a normal release/debug build of this crate contains none of this code, none of the
//! environment keys below, and none of its log text. The single call site in
//! `domain::chat_engine::request_receipt` is guarded by the same `#[cfg(feature = "test-faults")]`.
//!
//! The window it reproduces is real product behaviour, not a synthetic path: `execute()` has already
//! awaited the turn operation (so the user row and the assistant row are committed and the request
//! receipt is still `running`) and has not yet reached `complete_chat_request`. The fault parks the
//! task after publishing the observed facts, so the operator can terminate the owning process and
//! then restart against the same database.
//!
//! Activation is joint: the cargo feature must be compiled in **and** `OCLIVE_TEST_RECEIPT_FAULT_ID`
//! must equal the current request identity. A second process started without that key never parks,
//! even though it links the same binary.

use crate::models::dto::SendMessageResponse;
use std::path::PathBuf;
use std::time::Duration;

const ENV_ARM_ID: &str = "OCLIVE_TEST_RECEIPT_FAULT_ID";
const ENV_READY_FILE: &str = "OCLIVE_TEST_RECEIPT_READY_FILE";
const ENV_RELEASE_FILE: &str = "OCLIVE_TEST_RECEIPT_RELEASE_FILE";

/// Upper bound on the park. The C3 verification kills the owning process instead of releasing it.
const PARK_TIMEOUT: Duration = Duration::from_secs(900);
const PARK_POLL: Duration = Duration::from_millis(200);

/// Facts observed at the parked instant; assembled by the call site from values it already holds.
pub(crate) struct ReceiptWindowFacts<'a> {
    pub scope: &'a str,
    pub request_id: &'a str,
    pub receipt_state_at_park: &'a str,
    pub response: &'a SendMessageResponse,
}

fn env_value(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Whether this process armed the receipt-window fault for exactly this request identity.
pub(crate) fn armed_for(request_id: &str) -> bool {
    env_value(ENV_ARM_ID).is_some_and(|armed| armed.eq_ignore_ascii_case(request_id))
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"<unserializable>\"".to_string())
}

/// Publish the parked-window facts and hold the turn here until release or timeout.
///
/// The readiness file is written with `create_new` semantics via a temporary sibling rename so a
/// reader never observes a half-written document.
pub(crate) async fn hold_before_receipt_commit(facts: &ReceiptWindowFacts<'_>) {
    if !armed_for(facts.request_id) {
        return;
    }

    let reply = facts.response.reply.clone();
    let user_message_id = facts.response.user_message_id.clone().unwrap_or_default();
    let assistant_message_id = facts
        .response
        .assistant_message_id
        .clone()
        .unwrap_or_default();
    let document = format!(
        concat!(
            "{{\n",
            "  \"fault\": \"receipt_window_before_complete\",\n",
            "  \"pid\": {pid},\n",
            "  \"scope\": {scope},\n",
            "  \"request_id\": {request_id},\n",
            "  \"receipt_state_at_park\": {state},\n",
            "  \"user_message_id\": {user_message_id},\n",
            "  \"assistant_message_id\": {assistant_message_id},\n",
            "  \"reply_len\": {reply_len},\n",
            "  \"reply\": {reply}\n",
            "}}\n"
        ),
        pid = std::process::id(),
        scope = json_string(facts.scope),
        request_id = json_string(facts.request_id),
        state = json_string(facts.receipt_state_at_park),
        user_message_id = json_string(&user_message_id),
        assistant_message_id = json_string(&assistant_message_id),
        reply_len = reply.chars().count(),
        reply = json_string(&reply),
    );

    let ready = env_value(ENV_READY_FILE);
    if let Some(path) = ready.as_deref() {
        write_atomic(PathBuf::from(path), &document);
    }
    tracing::warn!(
        target: "oclive_test_faults",
        request_id = facts.request_id,
        receipt_state = facts.receipt_state_at_park,
        assistant_message_id = assistant_message_id.as_str(),
        "receipt-window fault armed: parked after the turn committed and before the outcome receipt"
    );

    let release = env_value(ENV_RELEASE_FILE).map(PathBuf::from);
    let started = std::time::Instant::now();
    loop {
        if let Some(path) = release.as_ref() {
            if path.exists() {
                tracing::warn!(target: "oclive_test_faults", "receipt-window fault released");
                return;
            }
        }
        if started.elapsed() >= PARK_TIMEOUT {
            tracing::warn!(
                target: "oclive_test_faults",
                "receipt-window fault park timed out; continuing to the outcome receipt"
            );
            return;
        }
        tokio::time::sleep(PARK_POLL).await;
    }
}

fn write_atomic(path: PathBuf, contents: &str) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let temporary = path.with_extension("partial");
    if std::fs::write(&temporary, contents).is_ok() {
        let _ = std::fs::rename(&temporary, &path);
    }
}
