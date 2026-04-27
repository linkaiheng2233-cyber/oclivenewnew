//! OOCP handler wrapper for `src-tauri/`.
//!
//! The transport-agnostic logic lives in `crates/oclive_core` (kernel).
//! Here we only provide a thin adapter that wires auth/limits into capabilities.

use crate::models::oocp::OocpCapabilities;

pub use oclive_core::oocp_handler::{
    dispatch_oocp_request, MethodError, OocpHandled, OocpMethodHandler,
};

/// `src-tauri` runtime decides whether auth is required based on env.
fn auth_required_from_env() -> bool {
    std::env::var("OOCP_API_TOKEN")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .is_some()
}

/// Build capabilities for the current runtime (v0.1).
pub fn get_capabilities() -> OocpCapabilities {
    oclive_core::oocp_handler::get_capabilities(auth_required_from_env(), 8, 4096)
}
