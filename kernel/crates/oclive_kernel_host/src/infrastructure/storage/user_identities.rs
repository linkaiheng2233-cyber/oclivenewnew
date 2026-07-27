//! Load `user_identities/` from a role pack directory.

use crate::error::{AppError, Result};
use crate::models::user_identity::{
    UserIdentityCatalog, UserIdentityCatalogEntry, UserIdentityIndex,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// Load `user_identities/index.json` and template bodies when present.
pub fn load_user_identity_catalog(role_dir: &Path) -> Result<Option<Arc<UserIdentityCatalog>>> {
    let base = role_dir.join("user_identities");
    let index_path = base.join("index.json");
    if !index_path.is_file() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&index_path).map_err(AppError::IoError)?;
    let index: UserIdentityIndex =
        serde_json::from_str(&raw).map_err(AppError::SerializationError)?;

    let mut identities = HashMap::new();
    for (id, entry) in &index.identities {
        let rel = entry.template_file.trim();
        let template_path = base.join(rel);
        let body = fs::read_to_string(&template_path).map_err(|e| {
            AppError::InvalidParameter(format!(
                "user_identities template unreadable: {} — {e}",
                template_path.display()
            ))
        })?;
        identities.insert(
            id.clone(),
            UserIdentityCatalogEntry {
                display_name: entry.display_name.clone(),
                template_body: Arc::from(body.trim()),
                maps_to_relation_id: entry.maps_to_relation_id.clone(),
                adult_eligible: entry.adult_eligible,
            },
        );
    }

    Ok(Some(Arc::new(UserIdentityCatalog {
        schema_version: index.schema_version,
        default_identity_id: index.default_identity_id.clone(),
        identities,
    })))
}
