//! Concurrency gate for Performance LLM's observe-only fallback.
//!
//! Runtime process control stays in `performance_llm`; this module owns only
//! fallback admission, in-flight draining, cancellation, and recovery races.

use std::sync::Arc;

use parking_lot::Mutex;

use crate::error::{AppError, Result};

pub(super) const FALLBACK_BLOCKED_RESOURCE_TRANSITION: &str =
    "llm_fallback_blocked_resource_transition";

#[derive(Default)]
struct FallbackAdmissionState {
    block: Option<FallbackBlockState>,
    active_requests: usize,
    next_block_generation: u64,
}

struct FallbackBlockState {
    generation: u64,
    reason: String,
    transitioning: bool,
    reopen_requested: bool,
}

#[derive(Default)]
pub(super) struct FallbackAdmissionGate {
    state: Mutex<FallbackAdmissionState>,
    drained: tokio::sync::Notify,
}

pub(super) struct FallbackRequestGuard {
    gate: Arc<FallbackAdmissionGate>,
}

#[derive(Clone, Copy)]
struct FallbackBlockToken(u64);

pub(super) struct FallbackBlockAttempt {
    gate: Arc<FallbackAdmissionGate>,
    token: FallbackBlockToken,
    finished: bool,
}

impl FallbackBlockAttempt {
    pub(super) fn finish(mut self) -> bool {
        self.finished = true;
        self.gate.finish_block(self.token)
    }
}

impl Drop for FallbackBlockAttempt {
    fn drop(&mut self) {
        if !self.finished {
            self.gate.abandon_block(self.token);
        }
    }
}

impl Drop for FallbackRequestGuard {
    fn drop(&mut self) {
        let drained = {
            let mut state = self.gate.state.lock();
            debug_assert!(
                state.active_requests > 0,
                "fallback request guard must match one admission"
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

impl FallbackAdmissionGate {
    pub(super) fn try_enter(self: &Arc<Self>) -> Result<FallbackRequestGuard> {
        let mut state = self.state.lock();
        if let Some(block) = state.block.as_ref() {
            return Err(AppError::RemoteServiceUnavailable(format!(
                "{FALLBACK_BLOCKED_RESOURCE_TRANSITION}: {}",
                block.reason
            )));
        }
        state.active_requests = state.active_requests.checked_add(1).ok_or_else(|| {
            AppError::RemoteServiceUnavailable("LLM fallback admission counter exhausted".into())
        })?;
        Ok(FallbackRequestGuard {
            gate: Arc::clone(self),
        })
    }

    pub(super) fn begin_block(
        self: &Arc<Self>,
        reason: &str,
    ) -> Result<(FallbackBlockAttempt, String)> {
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
            let token = FallbackBlockToken(state.next_block_generation);
            state.block = Some(FallbackBlockState {
                generation: token.0,
                reason: normalized.clone(),
                transitioning: true,
                reopen_requested: false,
            });
            token
        };
        Ok((
            FallbackBlockAttempt {
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
        }
    }

    pub(super) fn is_blocked(&self) -> bool {
        self.state.lock().block.is_some()
    }

    fn finish_block(&self, token: FallbackBlockToken) -> bool {
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
            false
        } else {
            block.transitioning = false;
            true
        }
    }

    fn abandon_block(&self, token: FallbackBlockToken) {
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
}
