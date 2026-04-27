//! Oclive kernel runtime crate (no tauri).
//!
//! This crate will progressively absorb the runtime currently hosted in `src-tauri/src/*`
//! so that both:
//! - `crates/oclive_kernel_server` (headless kernel)
//! - `src-tauri` (official desktop distribution)
//!
//! can depend on the same kernel runtime implementation.

pub mod api;
pub mod domain;
pub mod env_flags;
pub mod error;
pub mod http_api;
pub mod infrastructure;
pub mod models;
pub mod state;
pub mod utils;
