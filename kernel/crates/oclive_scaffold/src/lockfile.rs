use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use tempfile::NamedTempFile;

use crate::{
    ResolvedCatalog, ScaffoldError, ScaffoldLock, ScaffoldLockPackage, SCAFFOLD_CONTRACT_VERSION,
    SCAFFOLD_LOCK_FILENAME, SCAFFOLD_LOCK_SCHEMA_VERSION,
};

/// Build a deterministic lock document from one resolved catalog.
#[must_use]
pub fn build_scaffold_lock(catalog: &ResolvedCatalog) -> ScaffoldLock {
    let mut packages = catalog
        .packages
        .iter()
        .map(|package| {
            let mut permissions = package.manifest.permissions.clone();
            permissions.sort();
            permissions.dedup();
            let mut unresolved_dependencies = package.manifest.dependencies.clone();
            unresolved_dependencies.sort_by(|a, b| a.id.cmp(&b.id));
            let mut unresolved_extends = package.manifest.extends.clone();
            unresolved_extends.sort_by(|a, b| a.id.cmp(&b.id));
            ScaffoldLockPackage {
                id: package.manifest.package.id.clone(),
                version: package.manifest.package.version.clone(),
                source: package.source,
                locator: package.locator.clone(),
                manifest_sha256: package.manifest_sha256.clone(),
                maintainer: package.manifest.package.maintainer.clone(),
                trust: package.trust,
                command_namespace: package.manifest.command_namespace.clone(),
                permissions,
                unresolved_dependencies,
                unresolved_extends,
                composition_declared: !package.manifest.composition.is_empty(),
            }
        })
        .collect::<Vec<_>>();
    packages.sort_by(|a, b| a.id.cmp(&b.id));
    let mut warnings = catalog.warnings.clone();
    warnings.sort_by(|a, b| a.code.cmp(&b.code).then(a.message.cmp(&b.message)));
    warnings.dedup();
    ScaffoldLock {
        schema_version: SCAFFOLD_LOCK_SCHEMA_VERSION,
        scaffold_contract: SCAFFOLD_CONTRACT_VERSION.to_string(),
        reader_version: catalog.reader_version.clone(),
        source_order: catalog.source_order.clone(),
        packages,
        warnings,
    }
}

/// Return the conventional project-local lock path.
#[must_use]
pub fn project_scaffold_lock_path(project_root: &Path) -> PathBuf {
    project_root.join(".oclive").join(SCAFFOLD_LOCK_FILENAME)
}

/// Atomically replace a scaffold lock document using a temporary file in the same directory.
///
/// # Errors
///
/// Returns [`ScaffoldError::WriteLock`] when directories, serialization, flushing, or the
/// final atomic persistence operation fails.
pub fn write_scaffold_lock_atomic(path: &Path, lock: &ScaffoldLock) -> Result<(), ScaffoldError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| ScaffoldError::WriteLock {
        path: path.to_path_buf(),
        source,
    })?;
    let bytes = serde_json::to_vec_pretty(lock)
        .map_err(std::io::Error::other)
        .map_err(|source| ScaffoldError::WriteLock {
            path: path.to_path_buf(),
            source,
        })?;
    let mut temporary =
        NamedTempFile::new_in(parent).map_err(|source| ScaffoldError::WriteLock {
            path: path.to_path_buf(),
            source,
        })?;
    temporary
        .write_all(&bytes)
        .and_then(|()| temporary.write_all(b"\n"))
        .and_then(|()| temporary.as_file_mut().sync_all())
        .map_err(|source| ScaffoldError::WriteLock {
            path: path.to_path_buf(),
            source,
        })?;
    temporary
        .persist(path)
        .map_err(|error_value| ScaffoldError::WriteLock {
            path: path.to_path_buf(),
            source: error_value.error,
        })?;
    Ok(())
}
