//! User Identity Prompt Template session API (shared by Tauri / HTTP).

use crate::command_error::CommandError;
use crate::domain::user_identity_loader::resolve_active_user_identity;
use crate::models::dto::{
    GetUserIdentityStateRequest, SetSceneUserIdentityRequest, SetUserIdentityRequest,
    UserIdentityDto, UserIdentityStateResponse, OCLIVE_DEFAULT_IDENTITY_SENTINEL,
};
use crate::models::role::{IdentityBinding, Role};
use crate::state::AppState;

use super::ensure_manifest_role_ready;

fn identity_allowed_by_host(state: &AppState, identity_id: &str) -> bool {
    let Some(ref allowed) = state.host_profile.user_identity.allowed_ids else {
        return true;
    };
    allowed.iter().any(|id| id == identity_id)
}

fn reject_identity_not_allowed(state: &AppState, identity_id: &str) -> Result<(), CommandError> {
    if identity_allowed_by_host(state, identity_id) {
        return Ok(());
    }
    Err(crate::error::AppError::InvalidParameter(format!(
        "user identity not allowed by host profile: {identity_id}"
    ))
    .into())
}

fn identity_catalog_or_empty(role: &Role) -> Vec<UserIdentityDto> {
    role.user_identity_catalog
        .as_ref()
        .map(|c| {
            c.identities
                .iter()
                .map(|(id, e)| UserIdentityDto {
                    id: id.clone(),
                    display_name: e.display_name.clone(),
                    maps_to_relation_id: e.maps_to_relation_id.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn default_identity_id(role: &Role) -> String {
    role.user_identity_catalog
        .as_ref()
        .map(|c| c.default_identity_id.clone())
        .unwrap_or_default()
}

async fn sync_relation_for_identity(
    state: &AppState,
    role: &Role,
    role_id: &str,
    identity_id: &str,
) -> Result<(), CommandError> {
    let Some(catalog) = role.user_identity_catalog.as_ref() else {
        return Ok(());
    };
    let Some(entry) = catalog.identities.get(identity_id) else {
        return Ok(());
    };
    let relation_key = entry
        .maps_to_relation_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(identity_id);
    if !role.user_relations.iter().any(|r| r.id == relation_key) {
        return Ok(());
    }
    state
        .db_manager
        .set_use_manifest_default(role_id, false)
        .await?;
    state
        .db_manager
        .set_user_relation(role_id, relation_key)
        .await?;
    let seed = role.initial_favorability_for_relation(relation_key);
    state
        .db_manager
        .ensure_identity_stats_row(role_id, relation_key, seed)
        .await?;
    state
        .db_manager
        .mirror_runtime_from_identity(role_id, relation_key)
        .await?;
    Ok(())
}

/// # Errors
///
/// Returns [`Err`] when the role pack or identity id is invalid.
pub async fn set_user_identity_impl(
    state: &AppState,
    req: &SetUserIdentityRequest,
) -> Result<UserIdentityStateResponse, CommandError> {
    ensure_manifest_role_ready(state, &req.role_id).await?;
    let role = state.load_role_cached_async(&req.role_id).await?;

    if role.user_identity_catalog.is_none() {
        return Err(crate::error::AppError::InvalidParameter(
            "role pack has no user_identities/ catalog".to_string(),
        )
        .into());
    }

    if matches!(role.identity_binding, IdentityBinding::Global) {
        state
            .db_manager
            .clear_all_scene_identities_for_role(&req.role_id)
            .await?;
    }

    if req.identity_id == OCLIVE_DEFAULT_IDENTITY_SENTINEL {
        state
            .db_manager
            .set_use_manifest_default_identity(&req.role_id, true)
            .await?;
    } else {
        let catalog = role.user_identity_catalog.as_ref().expect("checked");
        if !catalog.identities.contains_key(&req.identity_id) {
            return Err(crate::error::AppError::InvalidParameter(format!(
                "unknown user identity: {}",
                req.identity_id
            ))
            .into());
        }
        reject_identity_not_allowed(state, &req.identity_id)?;
        state
            .db_manager
            .set_use_manifest_default_identity(&req.role_id, false)
            .await?;
        state
            .db_manager
            .set_active_user_identity_id(&req.role_id, &req.identity_id)
            .await?;
        sync_relation_for_identity(state, &role, &req.role_id, &req.identity_id).await?;
    }

    get_user_identity_state_impl(
        state,
        &GetUserIdentityStateRequest {
            role_id: req.role_id.clone(),
            scene_id: None,
        },
    )
    .await
}

/// # Errors
///
/// Returns [`Err`] when the role pack, scene, or identity id is invalid.
pub async fn set_scene_user_identity_impl(
    state: &AppState,
    req: &SetSceneUserIdentityRequest,
) -> Result<UserIdentityStateResponse, CommandError> {
    ensure_manifest_role_ready(state, &req.role_id).await?;
    let role = state.load_role_cached_async(&req.role_id).await?;
    if matches!(role.identity_binding, IdentityBinding::Global) {
        return Err(crate::error::AppError::InvalidParameter(
            "This role pack uses global identity_binding; per-scene identity overrides are not used."
                .to_string(),
        )
        .into());
    }
    let catalog = role.user_identity_catalog.as_ref().ok_or_else(|| {
        crate::error::AppError::InvalidParameter(
            "role pack has no user_identities/ catalog".to_string(),
        )
    })?;
    if !catalog.identities.contains_key(&req.identity_id) {
        return Err(crate::error::AppError::InvalidParameter(format!(
            "unknown user identity: {}",
            req.identity_id
        ))
        .into());
    }
    reject_identity_not_allowed(state, &req.identity_id)?;
    let scenes = state.storage.list_scene_ids(&req.role_id)?;
    if !scenes.iter().any(|s| s == &req.scene_id) {
        return Err(crate::error::AppError::InvalidParameter(format!(
            "scene_id not in role pack: {}",
            req.scene_id
        ))
        .into());
    }
    let entry = catalog.identities.get(&req.identity_id).expect("checked");
    let relation_key = entry
        .maps_to_relation_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(req.identity_id.as_str());
    state
        .db_manager
        .set_user_identity_for_scene(&req.role_id, &req.scene_id, &req.identity_id, relation_key)
        .await?;
    get_user_identity_state_impl(
        state,
        &GetUserIdentityStateRequest {
            role_id: req.role_id.clone(),
            scene_id: Some(req.scene_id.clone()),
        },
    )
    .await
}

/// # Errors
///
/// Returns [`Err`] when the role cannot be loaded or DB read fails.
pub async fn get_user_identity_state_impl(
    state: &AppState,
    req: &GetUserIdentityStateRequest,
) -> Result<UserIdentityStateResponse, CommandError> {
    ensure_manifest_role_ready(state, &req.role_id).await?;
    let role = state.load_role_cached_async(&req.role_id).await?;
    let resolved =
        resolve_active_user_identity(state, &role, &req.role_id, req.scene_id.as_deref()).await?;
    let scene_override = if matches!(role.identity_binding, IdentityBinding::PerScene) {
        if let Some(sid) = req.scene_id.as_deref() {
            state
                .db_manager
                .get_user_identity_id_for_scene(&req.role_id, sid)
                .await?
        } else {
            None
        }
    } else {
        None
    };
    let use_manifest_default = if scene_override.is_some() {
        false
    } else {
        state
            .db_manager
            .get_use_manifest_default_identity(&req.role_id)
            .await?
    };
    let current_identity_id = scene_override.unwrap_or_else(|| {
        if use_manifest_default {
            default_identity_id(&role)
        } else {
            resolved.identity_id.clone()
        }
    });
    Ok(UserIdentityStateResponse {
        role_id: req.role_id.clone(),
        identities: identity_catalog_or_empty(&role),
        default_identity_id: default_identity_id(&role),
        current_identity_id,
        use_manifest_default,
        effective_relation_key: resolved.relation_key,
    })
}
