//! Deterministic, fail-safe CI impact planning for the OCLive monorepo.

mod contract_loader;
mod model;
mod path_rules;
mod planning;

pub use model::*;

use std::{collections::BTreeMap, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CiPlanError {
    #[error("failed to read CI contract {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse CI contract {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid CI contract {path}: {message}")]
    InvalidContract { path: PathBuf, message: String },
    #[error("unknown validation policy `{0}`")]
    UnknownPolicy(String),
}

#[derive(Debug, Clone)]
pub(crate) struct LoadedDescriptor {
    pub(crate) descriptor: Option<ModuleDescriptor>,
    pub(crate) issues: Vec<String>,
}

/// A loaded, validated set of CI planning contracts.
#[derive(Debug, Clone)]
pub struct Planner {
    pub(crate) impact_map: ImpactMap,
    pub(crate) catalog: ValidationCatalog,
    pub(crate) descriptors: BTreeMap<String, LoadedDescriptor>,
    pub(crate) warnings: Vec<String>,
    pub(crate) impact_map_sha256: String,
    pub(crate) validation_catalog_sha256: String,
}

#[cfg(test)]
mod tests;
