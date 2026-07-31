//! Versioned, local-only Scaffold Package contracts for OCLive developer tooling.
//!
//! This crate discovers and validates declarations. It deliberately contains no command
//! executor, network installer, marketplace client, or CI orchestration authority.

mod discovery;
mod lockfile;
mod model;
mod official;
mod validation;

pub use discovery::*;
pub use lockfile::*;
pub use model::*;
pub use official::*;
pub use validation::*;

use std::path::PathBuf;

use thiserror::Error;

/// Errors that prevent a scaffold contract from being trusted or resolved.
#[derive(Debug, Error)]
pub enum ScaffoldError {
    #[error("failed to read scaffold contract {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse scaffold contract {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid scaffold contract {path}: {issues}")]
    InvalidContract { path: PathBuf, issues: String },
    #[error("cannot resolve scaffold catalog: {issues}")]
    Resolution { issues: String },
    #[error("failed to write scaffold lock {path}: {source}")]
    WriteLock {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
