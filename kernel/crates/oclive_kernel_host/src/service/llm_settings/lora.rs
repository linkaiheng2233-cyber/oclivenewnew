//! Local llama.cpp LoRA adapter lifecycle (import / activate / delete).

use super::LOCAL_LORA_MUTATION;

use crate::command_error::CommandError;
use crate::domain::user_llm_env::{
    apply_user_llm_env, KEY_LLM_PROVIDER, KEY_LOCAL_LORA_ADAPTER_ID, KEY_LOCAL_LORA_ADAPTER_PATH,
    KEY_LOCAL_MODEL_PATH,
};
use crate::error::AppError;
use crate::infrastructure::llm_models::canonical_models_dir;
use crate::infrastructure::lora_adapters::{
    delete_local_lora_adapter, gguf_base_model_architecture, import_local_lora_adapter,
    resolve_local_lora_adapter,
};
use crate::state::AppState;
use oclive_kernel_types::models::{
    ActivateLocalLoraAdapterRequest, DeleteLocalLoraAdapterRequest, ImportLocalLoraAdapterRequest,
    LocalLoraAdapterDto, LoraContentRating,
};
use std::path::PathBuf;

fn require_adult_lora_acknowledgement(
    content_rating: &LoraContentRating,
    acknowledged: bool,
) -> Result<(), CommandError> {
    if *content_rating == LoraContentRating::Adult && !acknowledged {
        return Err(AppError::InvalidParameter(
            "adult-rated LoRA activation requires explicit acknowledgement".into(),
        )
        .into());
    }
    Ok(())
}

async fn persist_local_lora_selection(
    state: &AppState,
    adapter_id: Option<&str>,
    adapter_path: Option<&std::path::Path>,
) -> Result<(), CommandError> {
    state
        .db_manager
        .upsert_app_setting(KEY_LOCAL_LORA_ADAPTER_ID, adapter_id.unwrap_or_default())
        .await?;
    let path = adapter_path
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default();
    state
        .db_manager
        .upsert_app_setting(KEY_LOCAL_LORA_ADAPTER_PATH, path.trim())
        .await?;
    state.mark_user_llm_env_dirty();
    apply_user_llm_env(state).await?;
    Ok(())
}

/// Import a local llama.cpp LoRA GGUF into managed storage.
///
/// This service is intentionally exposed only through the local Tauri command;
/// the HTTP API never accepts arbitrary filesystem source paths.
///
/// # Errors
///
/// Returns validation, checksum, package, or filesystem errors.
pub async fn import_local_lora_adapter_impl(
    state: &AppState,
    request: &ImportLocalLoraAdapterRequest,
) -> Result<LocalLoraAdapterDto, CommandError> {
    let _guard = LOCAL_LORA_MUTATION.lock().await;
    let models_dir = canonical_models_dir(state);
    let active_id = state
        .db_manager
        .get_app_setting(KEY_LOCAL_LORA_ADAPTER_ID)
        .await?
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let request = request.clone();
    tokio::task::spawn_blocking(move || {
        import_local_lora_adapter(&models_dir, &request, active_id.as_deref())
            .map_err(CommandError::from)
    })
    .await
    .map_err(|error| {
        CommandError::from(AppError::InvalidParameter(format!(
            "LoRA import worker failed: {error}"
        )))
    })?
}

/// Apply one installed adapter to the managed llama-server, or deactivate LoRA.
///
/// Persistence and runtime changes are transactional from the user's point of
/// view: when the new selection cannot start, the previous selection is restored.
///
/// # Errors
///
/// Returns an error for unsupported distro modes, missing base models, adult
/// content without acknowledgement, invalid adapters, or runtime startup failure.
pub async fn activate_local_lora_adapter_impl(
    state: &AppState,
    request: &ActivateLocalLoraAdapterRequest,
) -> Result<Option<LocalLoraAdapterDto>, CommandError> {
    let _guard = LOCAL_LORA_MUTATION.lock().await;
    let performance = state.performance_llm.as_ref().ok_or_else(|| {
        CommandError::from(AppError::InvalidParameter(
            "LoRA activation requires the managed performance llama.cpp runtime".into(),
        ))
    })?;
    let provider = state
        .db_manager
        .get_app_setting(KEY_LLM_PROVIDER)
        .await?
        .unwrap_or_else(|| "local".to_string());
    if provider.trim() != "local" {
        return Err(AppError::InvalidParameter(
            "switch to the local model provider before activating a LoRA adapter".into(),
        )
        .into());
    }
    let base_model = state
        .db_manager
        .get_app_setting(KEY_LOCAL_MODEL_PATH)
        .await?
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            CommandError::from(AppError::InvalidParameter(
                "select and save a local GGUF base model before activating LoRA".into(),
            ))
        })?;
    if !std::path::Path::new(&base_model).is_file() {
        return Err(AppError::InvalidParameter(
            "the selected local GGUF base model no longer exists".into(),
        )
        .into());
    }
    if !std::path::Path::new(&base_model)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("gguf"))
    {
        return Err(AppError::InvalidParameter(
            "llama.cpp LoRA activation requires a GGUF base model".into(),
        )
        .into());
    }

    let adapter_id = request
        .adapter_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_string);
    let selected = if let Some(id) = adapter_id.as_deref() {
        let models_dir = canonical_models_dir(state);
        let id_owned = id.to_string();
        let resolved = tokio::task::spawn_blocking(move || {
            resolve_local_lora_adapter(&models_dir, &id_owned, Some(&id_owned))
                .map_err(CommandError::from)
        })
        .await
        .map_err(|error| {
            CommandError::from(AppError::InvalidParameter(format!(
                "LoRA verification worker failed: {error}"
            )))
        })??;
        let base_model_path = PathBuf::from(&base_model);
        let base_architecture =
            tokio::task::spawn_blocking(move || gguf_base_model_architecture(&base_model_path))
                .await
                .map_err(|error| {
                    CommandError::from(AppError::InvalidParameter(format!(
                        "base-model GGUF verification worker failed: {error}"
                    )))
                })??;
        if let (Some(adapter_architecture), Some(base_architecture)) = (
            resolved.dto.architecture.as_deref(),
            base_architecture.as_deref(),
        ) {
            if !adapter_architecture.eq_ignore_ascii_case(base_architecture) {
                return Err(AppError::InvalidParameter(format!(
                    "LoRA/base architecture mismatch: adapter '{adapter_architecture}', base model '{base_architecture}'"
                ))
                .into());
            }
        }
        require_adult_lora_acknowledgement(
            &resolved.dto.content_rating,
            request.adult_content_acknowledged,
        )?;
        Some(resolved)
    } else {
        None
    };

    let previous_id = state
        .db_manager
        .get_app_setting(KEY_LOCAL_LORA_ADAPTER_ID)
        .await?
        .unwrap_or_default();
    let previous_path = state
        .db_manager
        .get_app_setting(KEY_LOCAL_LORA_ADAPTER_PATH)
        .await?
        .unwrap_or_default();
    persist_local_lora_selection(
        state,
        adapter_id.as_deref(),
        selected.as_ref().map(|adapter| adapter.gguf_path.as_path()),
    )
    .await?;

    if let Err(error) = performance.apply_runtime_selection().await {
        let rollback_id = (!previous_id.trim().is_empty()).then_some(previous_id.trim());
        let rollback_path =
            (!previous_path.trim().is_empty()).then(|| std::path::Path::new(previous_path.trim()));
        if let Err(rollback_error) =
            persist_local_lora_selection(state, rollback_id, rollback_path).await
        {
            tracing::error!(
                target: "oclive_lora",
                %rollback_error,
                "failed to restore previous LoRA settings after activation failure"
            );
        } else if let Err(rollback_runtime_error) = performance.apply_runtime_selection().await {
            tracing::error!(
                target: "oclive_lora",
                %rollback_runtime_error,
                "previous llama.cpp selection was restored but could not be restarted"
            );
        }
        return Err(AppError::InvalidParameter(format!(
            "LoRA activation failed and the previous selection was restored: {error}"
        ))
        .into());
    }

    Ok(selected.map(|resolved| resolved.dto))
}

/// Delete one inactive managed adapter.
///
/// # Errors
///
/// Returns an error when the adapter is active, invalid, missing, or cannot be removed.
pub async fn delete_local_lora_adapter_impl(
    state: &AppState,
    request: &DeleteLocalLoraAdapterRequest,
) -> Result<(), CommandError> {
    let _guard = LOCAL_LORA_MUTATION.lock().await;
    let id = request.adapter_id.trim();
    let active_id = state
        .db_manager
        .get_app_setting(KEY_LOCAL_LORA_ADAPTER_ID)
        .await?
        .unwrap_or_default();
    if !id.is_empty() && id == active_id.trim() {
        return Err(AppError::InvalidParameter(
            "deactivate the LoRA adapter before deleting it".into(),
        )
        .into());
    }
    let models_dir = canonical_models_dir(state);
    let id = id.to_string();
    tokio::task::spawn_blocking(move || {
        delete_local_lora_adapter(&models_dir, &id).map_err(CommandError::from)
    })
    .await
    .map_err(|error| {
        CommandError::from(AppError::InvalidParameter(format!(
            "LoRA delete worker failed: {error}"
        )))
    })?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adult_lora_is_rejected_without_its_own_acknowledgement() {
        let error = require_adult_lora_acknowledgement(&LoraContentRating::Adult, false)
            .expect_err("adult LoRA acknowledgement");

        assert!(error.to_string().contains("explicit acknowledgement"));
        assert!(require_adult_lora_acknowledgement(&LoraContentRating::Adult, true,).is_ok());
    }
}
