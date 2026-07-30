//! Concurrency gate for Performance LLM requests and resource transitions.
//!
//! Runtime process control stays in `performance_llm`; this module owns request
//! admission, in-flight draining, cancellation, and recovery races. Counting
//! the whole request lifetime prevents a resource transition from killing the
//! managed primary between an external activity snapshot and the actual call.

use std::sync::Arc;

use parking_lot::Mutex;

use crate::error::{AppError, Result};

pub(super) const REQUEST_BLOCKED_RESOURCE_TRANSITION: &str =
    "llm_request_blocked_resource_transition";

#[derive(Default)]
struct RequestAdmissionState {
    block: Option<RequestBlockState>,
    active_requests: usize,
    next_block_generation: u64,
}

struct RequestBlockState {
    generation: u64,
    reason: String,
    transitioning: bool,
    reopen_requested: bool,
}

#[derive(Default)]
pub(super) struct PerformanceRequestGate {
    state: Mutex<RequestAdmissionState>,
    drained: tokio::sync::Notify,
    opened: tokio::sync::Notify,
}

pub(super) struct PerformanceRequestGuard {
    gate: Arc<PerformanceRequestGate>,
}

#[derive(Clone, Copy)]
struct RequestBlockToken(u64);

pub(super) struct RequestBlockAttempt {
    gate: Arc<PerformanceRequestGate>,
    token: RequestBlockToken,
    finished: bool,
}

impl RequestBlockAttempt {
    pub(super) fn finish(mut self) -> bool {
        self.finished = true;
        self.gate.finish_block(self.token)
    }
}

impl Drop for RequestBlockAttempt {
    fn drop(&mut self) {
        if !self.finished {
            self.gate.abandon_block(self.token);
        }
    }
}

impl Drop for PerformanceRequestGuard {
    fn drop(&mut self) {
        let drained = {
            let mut state = self.gate.state.lock();
            debug_assert!(
                state.active_requests > 0,
                "Performance request guard must match one admission"
            );
            if state.active_requests > 0 {
                state.active_requests -= 1;
            }
            state.active_requests == 0
        };
        if drained {
            self.gate.drained.notify_waiters();
        }
    }
}

impl PerformanceRequestGate {
    pub(super) fn try_enter(self: &Arc<Self>) -> Result<PerformanceRequestGuard> {
        let mut state = self.state.lock();
        if let Some(block) = state.block.as_ref() {
            return Err(AppError::RemoteServiceUnavailable(format!(
                "{REQUEST_BLOCKED_RESOURCE_TRANSITION}: {}",
                block.reason
            )));
        }
        state.active_requests = state.active_requests.checked_add(1).ok_or_else(|| {
            AppError::RemoteServiceUnavailable("Performance LLM request counter exhausted".into())
        })?;
        Ok(PerformanceRequestGuard {
            gate: Arc::clone(self),
        })
    }

    pub(super) fn begin_block(
        self: &Arc<Self>,
        reason: &str,
    ) -> Result<(RequestBlockAttempt, String)> {
        let reason = reason.trim();
        let normalized: String = if reason.is_empty() {
            "resource coordination transition is active".into()
        } else {
            reason.into()
        };
        let token = {
            let mut state = self.state.lock();
            if state.block.is_some() {
                return Err(AppError::RemoteServiceUnavailable(
                    "llm_resource_transition_already_active".into(),
                ));
            }
            state.next_block_generation = state.next_block_generation.wrapping_add(1);
            let token = RequestBlockToken(state.next_block_generation);
            state.block = Some(RequestBlockState {
                generation: token.0,
                reason: normalized.clone(),
                transitioning: true,
                reopen_requested: false,
            });
            token
        };
        Ok((
            RequestBlockAttempt {
                gate: Arc::clone(self),
                token,
                finished: false,
            },
            normalized,
        ))
    }

    pub(super) fn open(&self) {
        let mut state = self.state.lock();
        let Some(block) = state.block.as_mut() else {
            return;
        };
        if block.transitioning {
            block.reopen_requested = true;
        } else {
            state.block = None;
            self.opened.notify_waiters();
        }
    }

    pub(super) fn is_blocked(&self) -> bool {
        self.state.lock().block.is_some()
    }

    fn finish_block(&self, token: RequestBlockToken) -> bool {
        let mut state = self.state.lock();
        let Some(block) = state
            .block
            .as_mut()
            .filter(|block| block.generation == token.0)
        else {
            return false;
        };
        if block.reopen_requested {
            state.block = None;
            self.opened.notify_waiters();
            false
        } else {
            block.transitioning = false;
            true
        }
    }

    fn abandon_block(&self, token: RequestBlockToken) {
        let mut state = self.state.lock();
        let Some(block) = state
            .block
            .as_mut()
            .filter(|block| block.generation == token.0)
        else {
            return;
        };
        if block.reopen_requested {
            state.block = None;
            self.opened.notify_waiters();
        } else {
            block.transitioning = false;
        }
    }

    pub(super) async fn wait_until_drained(&self) {
        loop {
            let notified = self.drained.notified();
            if self.state.lock().active_requests == 0 {
                return;
            }
            notified.await;
        }
    }

    pub(super) async fn wait_until_open(&self) {
        loop {
            let notified = self.opened.notified();
            if !self.is_blocked() {
                return;
            }
            notified.await;
        }
    }
}
