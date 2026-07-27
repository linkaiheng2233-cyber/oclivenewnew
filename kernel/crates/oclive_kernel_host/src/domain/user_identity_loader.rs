//! Resolve active User Identity Prompt Template for a turn (catalog + DB + legacy fallback).

use crate::domain::chat_turn::relation_favor_for_key;
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::error::Result;
use crate::models::role::{IdentityBinding, Role};
use crate::state::AppState;

/// Resolved identity for prompt injection and relation/favor routing.
#[derive(Debug, Clone)]
pub struct ResolvedUserIdentity {
    pub identity_id: String,
    pub template_body: String,
    pub relation_key: String,
    pub relation_hint: String,
    pub adult_eligible: bool,
}

/// DB identity pick + manifest-default flag in one round-trip per binding mode.
async fn effective_identity_state_from_db(
    state: &AppState,
    role: &Role,
    role_id: &str,
    scene_id: Option<&str>,
) -> Result<(Option<String>, bool)> {
    if matches!(role.identity_binding, IdentityBinding::PerScene) {
        if let Some(sid) = scene_id {
            let id = state
                .db_manager
                .get_user_identity_id_for_scene(role_id, sid)
                .await?;
            let use_manifest_default = id.is_none();
            return Ok((id, use_manifest_default));
        }
        return Ok((None, true));
    }
    let (use_manifest_default, db_id) = state.db_manager.get_global_identity_state(role_id).await?;
    Ok((
        if use_manifest_default { None } else { db_id },
        use_manifest_default,
    ))
}

fn catalog_entry_for_id<'a>(
    role: &'a Role,
    identity_id: &str,
) -> Option<&'a crate::models::user_identity::UserIdentityCatalogEntry> {
    role.user_identity_catalog
        .as_ref()
        .and_then(|c| c.identities.get(identity_id))
}

/// Priority merge: session identity → catalog default → legacy prompt hint (shared policy across hosts).
///
/// # Errors
///
/// Returns [`Err`] when DB reads for scene/global identity state fail.
pub async fn resolve_active_user_identity(
    state: &AppState,
    role: &Role,
    role_id: &str,
    scene_id: Option<&str>,
) -> Result<ResolvedUserIdentity> {
    let catalog_default = role
        .user_identity_catalog
        .as_ref()
        .map(|c| c.default_identity_id.as_str());

    let (db_id, use_manifest_default) =
        effective_identity_state_from_db(state, role, role_id, scene_id).await?;
    let host_default = if !use_manifest_default && db_id.is_none() {
        state
            .host_profile
            .user_identity
            .default_id
            .as_deref()
            .filter(|s| !s.is_empty())
    } else {
        None
    };
    let chosen_id = db_id
        .as_deref()
        .or(host_default)
        .or(catalog_default)
        .map(str::to_string);

    if let Some(ref identity_id) = chosen_id {
        if let Some(entry) = catalog_entry_for_id(role, identity_id) {
            let relation_key = entry
                .maps_to_relation_id
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or(identity_id.as_str())
                .to_string();
            let rf = relation_favor_for_key(role, relation_key.as_str());
            return Ok(ResolvedUserIdentity {
                identity_id: identity_id.clone(),
                template_body: entry.template_body.to_string(),
                relation_key,
                relation_hint: rf.relation_hint.to_string(),
                adult_eligible: entry.adult_eligible,
            });
        }
    }

    let relation_key = resolve_effective_user_relation_key(state, role, role_id, scene_id).await?;
    let rf = relation_favor_for_key(role, relation_key.as_str());
    Ok(ResolvedUserIdentity {
        identity_id: String::new(),
        template_body: String::new(),
        relation_key,
        relation_hint: rf.relation_hint.to_string(),
        adult_eligible: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::user_identity::UserIdentityIndexEntry;
    use crate::models::user_identity::{UserIdentityCatalog, UserIdentityCatalogEntry};
    use std::collections::HashMap;
    use std::sync::Arc;

    fn role_with_catalog() -> Role {
        let mut role = Role::default();
        role.default_relation = "classmate".to_string();
        role.user_relations = vec![crate::models::role::UserRelation {
            id: "classmate".to_string(),
            name: "同学".to_string(),
            prompt_hint: "legacy hint".to_string(),
            favor_multiplier: 1.0,
            initial_favorability: 20.0,
        }];
        let mut identities = HashMap::new();
        identities.insert(
            "classmate".to_string(),
            UserIdentityCatalogEntry {
                display_name: "同学".to_string(),
                template_body: Arc::from("【用户身份说明】模板正文"),
                maps_to_relation_id: Some("classmate".to_string()),
                adult_eligible: true,
            },
        );
        role.user_identity_catalog = Some(Arc::new(UserIdentityCatalog {
            schema_version: 1,
            default_identity_id: "classmate".to_string(),
            identities,
        }));
        role
    }

    #[test]
    fn catalog_default_yields_template_body() {
        let role = role_with_catalog();
        let entry = catalog_entry_for_id(&role, "classmate").expect("entry");
        assert!(entry.template_body.contains("模板正文"));
    }

    #[test]
    fn index_entry_maps_to_relation_default_is_identity_id() {
        let entry = UserIdentityIndexEntry {
            display_name: "x".to_string(),
            template_file: "x.md".to_string(),
            maps_to_relation_id: None,
            adult_eligible: true,
        };
        let id = "friend";
        let relation_key = entry
            .maps_to_relation_id
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(id);
        assert_eq!(relation_key, "friend");
    }
}
