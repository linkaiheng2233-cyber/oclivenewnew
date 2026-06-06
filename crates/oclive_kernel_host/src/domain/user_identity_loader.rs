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
}

async fn effective_identity_id_from_db(
    state: &AppState,
    role: &Role,
    role_id: &str,
    scene_id: Option<&str>,
) -> Result<Option<String>> {
    if matches!(role.identity_binding, IdentityBinding::PerScene) {
        if let Some(sid) = scene_id {
            return state
                .db_manager
                .get_user_identity_id_for_scene(role_id, sid)
                .await;
        }
        return Ok(None);
    }
    if state
        .db_manager
        .get_use_manifest_default_identity(role_id)
        .await?
    {
        return Ok(None);
    }
    state.db_manager.get_active_user_identity_id(role_id).await
}

fn catalog_entry_for_id<'a>(
    role: &'a Role,
    identity_id: &str,
) -> Option<&'a crate::models::user_identity::UserIdentityCatalogEntry> {
    role.user_identity_catalog
        .as_ref()
        .and_then(|c| c.identities.get(identity_id))
}

/// Priority: session identity id → catalog `default_identity_id` → legacy `user_relations.prompt_hint`.
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

    let db_id = effective_identity_id_from_db(state, role, role_id, scene_id).await?;
    let use_manifest_default = if matches!(role.identity_binding, IdentityBinding::PerScene) {
        scene_id.is_none()
            || state
                .db_manager
                .get_user_identity_id_for_scene(role_id, scene_id.unwrap_or(""))
                .await?
                .is_none()
    } else {
        state
            .db_manager
            .get_use_manifest_default_identity(role_id)
            .await?
    };
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
