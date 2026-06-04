//! Headless kernel entry surface (HTTP `--api`, tracing init).
//!
//! **Transitional facade**: re-exports [`oclivenewnew_tauri::run_api_server`] and tracing helpers.
//! HTTP routing and [`AppState`] construction still live in `src-tauri` until a dedicated
//! `oclive_kernel_host` implementation is extracted (see optimization plan §3.3).
//!
//! Consumers enable the `headless` feature on `oclivenewnew-tauri` (disables `desktop` /
//! deep-link deps) while still linking the library crate—not a windowless binary yet.

pub use oclivenewnew_tauri::{http_api, init_tracing, init_tracing_with_log_dir, run_api_server};
