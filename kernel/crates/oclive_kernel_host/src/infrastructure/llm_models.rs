//! Local GGUF model directory discovery and persistence.

use crate::command_error::CommandError;
use crate::domain::user_llm_env::KEY_LOCAL_MODELS_DIR;
use crate::error::AppError;
use crate::state::{
    ensure_models_dir_for_roles, is_managed_legacy_models_path, migrate_and_cleanup_models,
    paths_equal, reconcile_legacy_models_layout, AppState,
};
use oclive_kernel_types::models::{
    ContentRating, LocalModelManifest, LOCAL_MODEL_MANIFEST_KIND,
    LOCAL_MODEL_MANIFEST_SCHEMA_VERSION, LOCAL_MODEL_MANIFEST_SUFFIX,
};
use sha2::{Digest, Sha256};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

pub use oclive_kernel_types::models::LocalModelFileDto;

pub fn canonical_models_dir(state: &AppState) -> PathBuf {
    ensure_models_dir_for_roles(state.storage.roles_dir())
}

/// Persist user-selected local models directory in app settings.
///
/// # Errors
///
/// Returns database errors from `upsert_app_setting`.
pub async fn persist_local_models_dir(state: &AppState, path: &str) -> Result<(), CommandError> {
    state
        .db_manager
        .upsert_app_setting(KEY_LOCAL_MODELS_DIR, path.trim())
        .await?;
    Ok(())
}

/// Effective GGUF folder: repo-root `models/` (like `roles/`), migrating legacy app-data paths.
///
/// # Errors
///
/// Returns database or persistence errors while reconciling stored paths.
pub async fn local_models_dir_for_state(state: &AppState) -> Result<String, CommandError> {
    let canonical = canonical_models_dir(state);
    let canonical_str = canonical.to_string_lossy().into_owned();
    let app_data = state.directory_plugins.app_data_dir().to_path_buf();
    reconcile_legacy_models_layout(&canonical, &app_data);

    if let Ok(Some(v)) = state.db_manager.get_app_setting(KEY_LOCAL_MODELS_DIR).await {
        let t = v.trim();
        if !t.is_empty() {
            let stored = PathBuf::from(t);
            if paths_equal(&stored, &canonical) {
                return Ok(canonical_str);
            }
            if is_managed_legacy_models_path(&stored, &canonical, &app_data) {
                migrate_and_cleanup_models(&stored, &canonical);
                persist_local_models_dir(state, &canonical_str).await?;
                return Ok(canonical_str);
            }
            return Ok(t.to_string());
        }
    }

    persist_local_models_dir(state, &canonical_str).await?;
    Ok(canonical_str)
}

#[must_use]
pub fn scan_local_model_files_in(dir: &Path) -> Vec<LocalModelFileDto> {
    let mut out = Vec::new();
    scan_model_directory(dir, false, &mut out);

    let Ok(rd) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if !path.is_dir() || is_reserved_model_subdirectory(&path) {
            continue;
        }
        // Child folders are opt-in through sidecars. This exposes independent
        // managed bases without accidentally listing adapters, downloads, or
        // split shards as runnable base models.
        scan_model_directory(&path, true, &mut out);
    }

    out.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.path.to_lowercase().cmp(&b.path.to_lowercase()))
    });
    out.dedup_by(|a, b| a.path.eq_ignore_ascii_case(&b.path));
    out
}

/// Resolve one local model and its optional sidecar metadata.
///
/// # Errors
///
/// Returns a validation error when the file is missing, is not GGUF/BIN, or
/// has a malformed sidecar.
pub fn describe_local_model_file(path: &Path) -> Result<LocalModelFileDto, CommandError> {
    if !path.is_file() || !is_model_file(path) {
        return Err(AppError::InvalidParameter(
            "performance model must be an existing GGUF/BIN file".into(),
        )
        .into());
    }
    describe_local_model_file_inner(path, false)
        .map_err(|error| CommandError::from(AppError::InvalidParameter(error)))?
        .ok_or_else(|| {
            AppError::InvalidParameter("local base model metadata is missing".into()).into()
        })
}

/// Resolve a local model and verify its declared SHA-256 when present.
///
/// # Errors
///
/// Returns a validation or I/O error when the model, sidecar, or declared
/// checksum is invalid.
pub fn verify_local_model_file(path: &Path) -> Result<LocalModelFileDto, CommandError> {
    let descriptor = describe_local_model_file(path)?;
    let Some(expected) = descriptor.sha256.as_deref() else {
        return Ok(descriptor);
    };
    let file = std::fs::File::open(path)?;
    let mut reader = BufReader::with_capacity(1024 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let actual = format!("{:x}", hasher.finalize());
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(AppError::InvalidParameter(format!(
            "local base model SHA-256 mismatch for '{}': expected {expected}, got {actual}",
            path.display()
        ))
        .into());
    }
    Ok(descriptor)
}

fn scan_model_directory(dir: &Path, require_manifest: bool, out: &mut Vec<LocalModelFileDto>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || !is_model_file(&path) {
            continue;
        }
        match describe_local_model_file_inner(&path, require_manifest) {
            Ok(Some(model)) => out.push(model),
            Ok(None) => {}
            Err(error) => {
                tracing::warn!(
                    target: "oclive_models",
                    path = %path.display(),
                    %error,
                    "ignored local model with invalid metadata"
                );
            }
        }
    }
}

fn describe_local_model_file_inner(
    path: &Path,
    require_manifest: bool,
) -> Result<Option<LocalModelFileDto>, String> {
    let metadata = path
        .metadata()
        .map_err(|error| format!("read local model metadata: {error}"))?;
    let manifest = read_local_model_manifest(path)?;
    if require_manifest && manifest.is_none() {
        return Ok(None);
    }
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("model");
    let (name, content_rating, description, license, source, sha256) =
        if let Some(manifest) = manifest {
            (
                manifest.name,
                manifest.content_rating,
                manifest.description,
                manifest.license,
                manifest.source,
                manifest.sha256,
            )
        } else {
            (
                file_name.to_string(),
                ContentRating::General,
                None,
                None,
                None,
                None,
            )
        };
    Ok(Some(LocalModelFileDto {
        name,
        path: path.to_string_lossy().into_owned(),
        size_bytes: metadata.len(),
        content_rating,
        description,
        license,
        source,
        sha256,
    }))
}

fn read_local_model_manifest(path: &Path) -> Result<Option<LocalModelManifest>, String> {
    let manifest_path = local_model_manifest_path(path);
    if !manifest_path.is_file() {
        return Ok(None);
    }
    let bytes = std::fs::read(&manifest_path)
        .map_err(|error| format!("read {}: {error}", manifest_path.display()))?;
    let manifest: LocalModelManifest = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    if manifest.schema_version != LOCAL_MODEL_MANIFEST_SCHEMA_VERSION {
        return Err(format!(
            "{} uses unsupported schemaVersion {}",
            manifest_path.display(),
            manifest.schema_version
        ));
    }
    if manifest.kind != LOCAL_MODEL_MANIFEST_KIND {
        return Err(format!(
            "{} has unsupported kind '{}'",
            manifest_path.display(),
            manifest.kind
        ));
    }
    let actual_file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if !manifest.file_name.eq_ignore_ascii_case(actual_file_name) {
        return Err(format!(
            "{} targets '{}' instead of '{}'",
            manifest_path.display(),
            manifest.file_name,
            actual_file_name
        ));
    }
    if manifest.name.trim().is_empty() {
        return Err(format!("{} has an empty name", manifest_path.display()));
    }
    if manifest.sha256.as_deref().is_some_and(|sha256| {
        sha256.len() != 64 || !sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
    }) {
        return Err(format!("{} has an invalid sha256", manifest_path.display()));
    }
    Ok(Some(manifest))
}

fn local_model_manifest_path(path: &Path) -> PathBuf {
    let mut file_name = path
        .file_name()
        .map_or_else(std::ffi::OsString::new, std::ffi::OsString::from);
    file_name.push(LOCAL_MODEL_MANIFEST_SUFFIX);
    path.with_file_name(file_name)
}

fn is_reserved_model_subdirectory(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.eq_ignore_ascii_case("adapters") || name.eq_ignore_ascii_case("downloads")
        })
}

fn is_model_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("gguf") || extension.eq_ignore_ascii_case("bin")
        })
}

#[must_use]
pub fn model_name_from_gguf_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("local-model")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_file(path: &Path) {
        std::fs::write(path, b"gguf").expect("write model fixture");
    }

    #[test]
    fn scans_root_models_and_manifested_child_bases_only() {
        let temp = tempfile::tempdir().expect("temp dir");
        let loose = temp.path().join("loose.gguf");
        write_file(&loose);

        let adult_dir = temp.path().join("adult-base");
        std::fs::create_dir(&adult_dir).expect("adult dir");
        let adult = adult_dir.join("adult.gguf");
        write_file(&adult);
        let adult_sidecar = local_model_manifest_path(&adult);
        std::fs::write(
            &adult_sidecar,
            serde_json::to_vec_pretty(&LocalModelManifest {
                schema_version: LOCAL_MODEL_MANIFEST_SCHEMA_VERSION,
                kind: LOCAL_MODEL_MANIFEST_KIND.to_string(),
                file_name: "adult.gguf".to_string(),
                name: "Adult base".to_string(),
                content_rating: ContentRating::Adult,
                description: Some("adult-only fixture".to_string()),
                license: None,
                source: None,
                sha256: Some("a".repeat(64)),
            })
            .expect("serialize manifest"),
        )
        .expect("write manifest");

        let ignored_dir = temp.path().join("unregistered");
        std::fs::create_dir(&ignored_dir).expect("ignored dir");
        write_file(&ignored_dir.join("ignored.gguf"));

        let adapter_dir = temp.path().join("adapters");
        std::fs::create_dir(&adapter_dir).expect("adapter dir");
        let adapter = adapter_dir.join("adapter.gguf");
        write_file(&adapter);
        std::fs::write(
            local_model_manifest_path(&adapter),
            std::fs::read(&adult_sidecar).expect("read adult sidecar"),
        )
        .expect("write adapter sidecar");

        let scanned = scan_local_model_files_in(temp.path());
        assert_eq!(scanned.len(), 2);
        assert!(scanned
            .iter()
            .any(|model| model.path == loose.to_string_lossy()));
        let adult = scanned
            .iter()
            .find(|model| model.path == adult.to_string_lossy())
            .expect("adult base");
        assert_eq!(adult.name, "Adult base");
        assert_eq!(adult.content_rating, ContentRating::Adult);
    }

    #[test]
    fn rejects_a_mismatched_sidecar_target() {
        let temp = tempfile::tempdir().expect("temp dir");
        let model = temp.path().join("base.gguf");
        write_file(&model);
        std::fs::write(
            local_model_manifest_path(&model),
            r#"{
              "schemaVersion": 1,
              "kind": "oclive.local-base-model",
              "fileName": "other.gguf",
              "name": "Wrong target",
              "contentRating": "adult"
            }"#,
        )
        .expect("write manifest");

        let error = describe_local_model_file(&model).expect_err("must reject sidecar");
        assert!(error.to_string().contains("targets 'other.gguf'"));
    }

    #[test]
    fn verifies_a_declared_model_checksum() {
        let temp = tempfile::tempdir().expect("temp dir");
        let model = temp.path().join("base.gguf");
        write_file(&model);
        let sha256 = format!("{:x}", Sha256::digest(b"gguf"));
        std::fs::write(
            local_model_manifest_path(&model),
            serde_json::to_vec_pretty(&LocalModelManifest {
                schema_version: LOCAL_MODEL_MANIFEST_SCHEMA_VERSION,
                kind: LOCAL_MODEL_MANIFEST_KIND.to_string(),
                file_name: "base.gguf".to_string(),
                name: "Verified base".to_string(),
                content_rating: ContentRating::General,
                description: None,
                license: None,
                source: None,
                sha256: Some(sha256),
            })
            .expect("serialize manifest"),
        )
        .expect("write manifest");

        let verified = verify_local_model_file(&model).expect("verified base");
        assert_eq!(verified.name, "Verified base");
    }
}
