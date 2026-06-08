#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
// Legacy `crate::domain::*` re-exports from runtime are deprecated; allow until ratchet reaches zero.
#![allow(deprecated)]

//! Headless OCLive kernel: HTTP API, [`AppState`], orchestration, and infrastructure.
//!
//! `oclive-kernel-server` and `oclivenewnew-tauri --api` link this crate directly.
//! The Tauri desktop shell (`oclivenewnew-tauri`) depends on this crate for IPC impls and re-exports.

pub mod command_error;
pub mod domain;
pub mod env_flags;
pub mod error {
    pub use oclive_kernel_types::error::*;
}
pub mod http_api;
pub mod infrastructure;
pub mod models;
pub mod service;
pub mod state;
pub mod utils;

use std::path::Path;
use std::path::PathBuf;

/// Initialize `tracing` (stdout; optional rolling file when `log_dir` or `OCLIVE_LOG_DIR` is set).
#[must_use]
pub fn init_tracing() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let log_dir = std::env::var("OCLIVE_LOG_DIR")
        .ok()
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty());
    init_tracing_with_log_dir(log_dir.as_deref())
}

/// Like [`init_tracing`] but always writes to `log_dir/logs/` when `Some`.
#[must_use]
pub fn init_tracing_with_log_dir(
    log_dir: Option<&Path>,
) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::{EnvFilter, Layer};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let use_json = std::env::var("OCLIVE_LOG_FORMAT")
        .map(|v| v.eq_ignore_ascii_case("json"))
        .unwrap_or(false)
        || std::env::var("RUST_LOG")
            .map(|v| v.to_ascii_lowercase().contains("json"))
            .unwrap_or(false);

    let stdout_layer = if use_json {
        tracing_subscriber::fmt::layer()
            .json()
            .with_target(true)
            .boxed()
    } else {
        tracing_subscriber::fmt::layer().with_target(true).boxed()
    };

    let mut layers = vec![stdout_layer];
    let mut file_guard = None;

    if let Some(dir) = log_dir {
        let logs = dir.join("logs");
        if std::fs::create_dir_all(&logs).is_ok() {
            let Ok(file_appender) = tracing_appender::rolling::RollingFileAppender::builder()
                .rotation(tracing_appender::rolling::Rotation::DAILY)
                .filename_prefix("oclive")
                .filename_suffix("log")
                .max_log_files(7)
                .build(&logs)
            else {
                eprintln!("failed to build rolling log appender");
                return file_guard;
            };
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            file_guard = Some(guard);
            let file_layer = if use_json {
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_target(true)
                    .with_writer(non_blocking)
                    .boxed()
            } else {
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_writer(non_blocking)
                    .boxed()
            };
            layers.push(file_layer);
        }
    }

    if let Err(e) = tracing_subscriber::registry()
        .with(filter)
        .with(layers)
        .try_init()
    {
        eprintln!("tracing subscriber init failed: {e}");
    }

    file_guard
}

/// Standalone HTTP API entry (`--api` / `oclive-kernel-server`); no Tauri window or IPC.
pub fn run_api_server(port: u16) {
    let worker_threads = std::thread::available_parallelism()
        .map(|n| (n.get() / 2).max(2))
        .unwrap_or(2);
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("failed to build tokio runtime: {e}");
            std::process::exit(1);
        }
    };
    let r = rt.block_on(http_api::serve_api(port));
    if let Err(e) = r {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
