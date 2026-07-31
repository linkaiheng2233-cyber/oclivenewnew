//! Cross-distro persona and memory export/import.

use crate::command_error::CommandError;
use crate::error::AppError;
use crate::service::role::session_namespace;
use crate::state::AppState;
use chrono::Utc;
use oclive_kernel_types::{
    PortableLongTermMemoryEntry, PortableMemoryFile, PortablePersonaFile,
    PortableStateExportResponse, PortableStateImportRequest, PortableStateImportResponse,
    PortableStateRequest, PORTABLE_MEMORY_SCHEMA_VERSION, PORTABLE_PERSONA_SCHEMA_VERSION,
};
use std::collections::BTreeMap;

fn safe_filename_part(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                c
            }
        })
        .collect()
}

fn personality_vector(role: &crate::models::Role) -> Vec<f32> {
    vec![
        role.default_personality.stubbornness,
        role.default_personality.clinginess,
        role.default_personality.sensitivity,
        role.default_personality.assertiveness,
        role.default_personality.forgiveness,
        role.default_personality.talkativeness,
        role.default_personality.warmth,
    ]
}

fn invalid_portable(errors: Vec<String>) -> CommandError {
    AppError::InvalidParameter(errors.join("; ")).into()
}

/// Export the installed core persona plus the selected namespace's mutable profile.
///
/// # Errors
///
/// Returns a command error when the role, runtime namespace, database, or JSON serialization fails.
pub async fn export_portable_persona_impl(
    state: &AppState,
    req: &PortableStateRequest,
) -> Result<PortableStateExportResponse, CommandError> {
    let role = state.load_role_cached_async(req.role_id.trim()).await?;
    let namespace = session_namespace(role.id.as_str(), req.session_id.as_deref());
    state
        .db_manager
        .ensure_role_runtime(namespace.as_str())
        .await?;
    let mutable = state
        .db_manager
        .get_mutable_personality(namespace.as_str())
        .await?;
    let file = PortablePersonaFile {
        schema_version: PORTABLE_PERSONA_SCHEMA_VERSION,
        role_id: role.id.clone(),
        role_name: role.name.clone(),
        role_version: role.version.clone(),
        core_profile: role.core_personality.clone(),
        default_personality: personality_vector(role.as_ref()),
        mutable_profile: (!mutable.trim().is_empty()).then_some(mutable),
        exported_at: Some(Utc::now().to_rfc3339()),
        extensions: BTreeMap::new(),
    };
    let content = serde_json::to_string_pretty(&file).map_err(AppError::SerializationError)?;
    Ok(PortableStateExportResponse {
        content,
        suggested_filename: format!("{}.ocpersona", safe_filename_part(role.id.as_str())),
    })
}

/// Restore only the mutable persona. Installed core persona remains immutable.
///
/// # Errors
///
/// Returns a command error when the document is invalid, targets another role, or persistence fails.
pub async fn import_portable_persona_impl(
    state: &AppState,
    req: &PortableStateImportRequest,
) -> Result<PortableStateImportResponse, CommandError> {
    let role = state.load_role_cached_async(req.role_id.trim()).await?;
    let file = oclive_validation::parse_portable_persona(req.content.as_str())
        .map_err(invalid_portable)?;
    if file.role_id != role.id {
        return Err(AppError::InvalidParameter(format!(
            "ocpersona role_id={} 与目标角色 {} 不一致",
            file.role_id, role.id
        ))
        .into());
    }
    if file.core_profile.trim() != role.core_personality.trim() {
        return Err(AppError::InvalidParameter(
            "ocpersona 核心人设与已安装角色不一致；为防止串角，拒绝恢复可变人设".into(),
        )
        .into());
    }
    if file.default_personality != personality_vector(role.as_ref()) {
        return Err(AppError::InvalidParameter(
            "ocpersona 七维基础人格与已安装角色不一致；为防止串角，拒绝恢复可变人设".into(),
        )
        .into());
    }
    let namespace = session_namespace(role.id.as_str(), req.session_id.as_deref());
    state
        .db_manager
        .ensure_role_runtime(namespace.as_str())
        .await?;
    let restored = if let Some(mutable) = file.mutable_profile {
        state
            .db_manager
            .set_mutable_personality(namespace.as_str(), mutable.as_str())
            .await?;
        state.invalidate_personality_cache_for_role(namespace.as_str());
        true
    } else {
        false
    };
    Ok(PortableStateImportResponse {
        imported_long_term: 0,
        skipped_memory_seed: 0,
        mutable_profile_restored: restored,
    })
}

/// Export creator seeds and runtime LTM. Short-term cache, chats and situation state are excluded.
///
/// # Errors
///
/// Returns a command error when the role, database, or JSON serialization fails.
pub async fn export_portable_memory_impl(
    state: &AppState,
    req: &PortableStateRequest,
) -> Result<PortableStateExportResponse, CommandError> {
    let role = state.load_role_cached_async(req.role_id.trim()).await?;
    let namespace = session_namespace(role.id.as_str(), req.session_id.as_deref());
    let count = state.db_manager.count_memories(namespace.as_str()).await?;
    let limit = i32::try_from(count).unwrap_or(i32::MAX);
    let memories = state
        .db_manager
        .load_memories_paged(namespace.as_str(), limit, 0)
        .await?;
    let long_term = memories
        .into_iter()
        .map(|memory| PortableLongTermMemoryEntry {
            content: memory.content,
            importance: memory.importance,
            weight: memory.weight,
            created_at: Some(memory.created_at.to_rfc3339()),
            accessed_at: memory.accessed_at.map(|value| value.to_rfc3339()),
            scene_id: memory.scene_id,
            mention_count: memory.mention_count,
        })
        .collect();
    let file = PortableMemoryFile {
        schema_version: PORTABLE_MEMORY_SCHEMA_VERSION,
        role_id: role.id.clone(),
        session_id: req.session_id.clone(),
        memory_seed: role.memory_seed.clone(),
        long_term,
        exported_at: Some(Utc::now().to_rfc3339()),
        extensions: BTreeMap::new(),
    };
    let content = serde_json::to_string_pretty(&file).map_err(AppError::SerializationError)?;
    Ok(PortableStateExportResponse {
        content,
        suggested_filename: format!("{}.ocmemory", safe_filename_part(role.id.as_str())),
    })
}

/// Merge portable LTM into the selected namespace. Seed entries remain package-owned read-only data.
///
/// # Errors
///
/// Returns a command error when the document is invalid, targets another role, or persistence fails.
pub async fn import_portable_memory_impl(
    state: &AppState,
    req: &PortableStateImportRequest,
) -> Result<PortableStateImportResponse, CommandError> {
    let role = state.load_role_cached_async(req.role_id.trim()).await?;
    let file =
        oclive_validation::parse_portable_memory(req.content.as_str()).map_err(invalid_portable)?;
    if file.role_id != role.id {
        return Err(AppError::InvalidParameter(format!(
            "ocmemory role_id={} 与目标角色 {} 不一致",
            file.role_id, role.id
        ))
        .into());
    }
    let namespace = session_namespace(role.id.as_str(), req.session_id.as_deref());
    state
        .db_manager
        .ensure_role_runtime(namespace.as_str())
        .await?;
    let mut imported = 0_u32;
    for entry in &file.long_term {
        state
            .db_manager
            .import_portable_memory(namespace.as_str(), entry)
            .await?;
        imported = imported.saturating_add(1);
    }
    Ok(PortableStateImportResponse {
        imported_long_term: imported,
        skipped_memory_seed: u32::try_from(file.memory_seed.len()).unwrap_or(u32::MAX),
        mutable_profile_restored: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_filenames_are_sanitized() {
        assert_eq!(safe_filename_part("a/b:c"), "a_b_c");
    }
}
