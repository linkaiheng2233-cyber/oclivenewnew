//! Role pack `user_identities/` index and in-memory catalog (templates loaded by host).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

fn default_adult_eligible() -> bool {
    true
}

/// On-disk `user_identities/index.json` (before template files are read).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserIdentityIndex {
    pub schema_version: u32,
    pub default_identity_id: String,
    pub identities: HashMap<String, UserIdentityIndexEntry>,
}

/// One identity entry in `index.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserIdentityIndexEntry {
    pub display_name: String,
    pub template_file: String,
    #[serde(default)]
    pub maps_to_relation_id: Option<String>,
    /// Set to `false` for an identity explicitly authored as a minor.
    #[serde(default = "default_adult_eligible")]
    pub adult_eligible: bool,
}

/// Loaded catalog: index metadata + template bodies (host-populated after disk load).
#[derive(Debug, Clone)]
pub struct UserIdentityCatalog {
    pub schema_version: u32,
    pub default_identity_id: String,
    pub identities: HashMap<String, UserIdentityCatalogEntry>,
}

/// One resolved identity with template body in memory.
#[derive(Debug, Clone)]
pub struct UserIdentityCatalogEntry {
    pub display_name: String,
    pub template_body: Arc<str>,
    pub maps_to_relation_id: Option<String>,
    pub adult_eligible: bool,
}
