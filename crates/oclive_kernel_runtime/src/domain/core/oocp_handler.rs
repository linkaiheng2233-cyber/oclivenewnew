//! OOCP handler wrapper for `oclive_kernel_runtime`.
//!
//! Transport-agnostic dispatch lives in `crates/oclive_core`.
//! This wrapper wires env-based auth and runtime limits into the handshake capabilities.

use crate::models::oocp::OocpCapabilities;

pub use oclive_core::oocp_handler::{
    dispatch_oocp_request, MethodError, OocpHandled, OocpMethodHandler,
};

fn auth_required_from_env() -> bool {
    std::env::var("OOCP_API_TOKEN")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .is_some()
}

pub fn get_capabilities() -> OocpCapabilities {
    oclive_core::oocp_handler::get_capabilities(auth_required_from_env(), 8, 4096)
}
