use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use semver::Version;
use sha2::{Digest, Sha256};

use crate::{
    official_manifest_documents, validate_config, validate_manifest, CatalogCandidate,
    CatalogIssue, CatalogScan, ManifestValidation, ResolvedCatalog, ResolvedPackage,
    ScaffoldConfig, ScaffoldError, ScaffoldManifest, ScaffoldSource, ScaffoldTrust,
    ShadowedPackage, ValidationIssue, SCAFFOLD_CONFIG_FILENAME, SCAFFOLD_MANIFEST_FILENAME,
    SCAFFOLD_SCHEMA_VERSION,
};

const DEFAULT_SOURCE_ORDER: [ScaffoldSource; 3] = [
    ScaffoldSource::Project,
    ScaffoldSource::User,
    ScaffoldSource::Official,
];

/// Read one strict scaffold configuration document.
///
/// # Errors
///
/// Returns [`ScaffoldError::Read`] when the file cannot be read and
/// [`ScaffoldError::Parse`] when it is not valid v1 JSON.
pub fn read_scaffold_config(path: &Path) -> Result<ScaffoldConfig, ScaffoldError> {
    let bytes = fs::read(path).map_err(|source| ScaffoldError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|source| ScaffoldError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

/// Read a scaffold configuration when it exists.
///
/// # Errors
///
/// Returns the same errors as [`read_scaffold_config`] for an existing document.
pub fn read_optional_scaffold_config(path: &Path) -> Result<Option<ScaffoldConfig>, ScaffoldError> {
    if path.try_exists().map_err(|source| ScaffoldError::Read {
        path: path.to_path_buf(),
        source,
    })? {
        read_scaffold_config(path).map(Some)
    } else {
        Ok(None)
    }
}

/// Merge valid user and project configuration, with project keys taking precedence.
///
/// # Errors
///
/// Returns [`ScaffoldError::Resolution`] when either input configuration violates the
/// v1 contract. Invalid schema versions are never hidden by the merge.
pub fn merge_scaffold_configs(
    user: Option<&ScaffoldConfig>,
    project: Option<&ScaffoldConfig>,
) -> Result<ScaffoldConfig, ScaffoldError> {
    let mut issues = Vec::new();
    for (label, config) in [("user", user), ("project", project)] {
        if let Some(config) = config {
            issues.extend(
                validate_config(config)
                    .into_iter()
                    .map(|issue| ValidationIssue {
                        code: issue.code,
                        message: format!("{label} config: {}", issue.message),
                    }),
            );
        }
    }
    if !issues.is_empty() {
        return Err(resolution_error(&issues));
    }

    let mut merged = ScaffoldConfig::empty();
    if let Some(config) = user {
        merged.source_order.clone_from(&config.source_order);
        merged.package_sources.clone_from(&config.package_sources);
        merged.package_enabled.clone_from(&config.package_enabled);
    }
    if let Some(config) = project {
        if config.source_order.is_some() {
            merged.source_order.clone_from(&config.source_order);
        }
        merged
            .package_sources
            .extend(config.package_sources.clone());
        merged
            .package_enabled
            .extend(config.package_enabled.clone());
    }
    Ok(merged)
}

/// Return the conventional project-level scaffold configuration path.
#[must_use]
pub fn project_scaffold_config_path(project_root: &Path) -> PathBuf {
    project_root.join(".oclive").join(SCAFFOLD_CONFIG_FILENAME)
}

/// Return the conventional user-level scaffold configuration path.
#[must_use]
pub fn user_scaffold_config_path(oclive_home: &Path) -> PathBuf {
    oclive_home.join(SCAFFOLD_CONFIG_FILENAME)
}

/// Discover strict manifests from project, user, and compiled official sources.
///
/// Local roots are expected to be the `scaffolds` directories themselves. Discovery is
/// deliberately non-recursive: every direct child is one package boundary. Read, parse,
/// validation, duplicate, and containment problems are retained in [`CatalogScan::issues`]
/// so callers can present all evidence before refusing resolution.
#[must_use]
pub fn scan_scaffold_catalog(
    project_scaffolds: &Path,
    user_scaffolds: &Path,
    reader_version: &Version,
) -> CatalogScan {
    let mut scan = CatalogScan::default();
    scan_local_root(
        project_scaffolds,
        ScaffoldSource::Project,
        reader_version,
        &mut scan,
    );
    scan_local_root(
        user_scaffolds,
        ScaffoldSource::User,
        reader_version,
        &mut scan,
    );
    scan_official(reader_version, &mut scan);
    sort_scan(&mut scan);
    scan
}

/// Load and validate a single manifest without adding it to discovery.
///
/// # Errors
///
/// Returns read, parse, or contract validation errors. Local callers should pass
/// [`ScaffoldSource::Project`] or [`ScaffoldSource::User`] so reserved namespaces and
/// built-in entries cannot be claimed by third parties.
pub fn load_scaffold_manifest(
    path: &Path,
    source: ScaffoldSource,
    reader_version: &Version,
) -> Result<CatalogCandidate, ScaffoldError> {
    let bytes = fs::read(path).map_err(|source_error| ScaffoldError::Read {
        path: path.to_path_buf(),
        source: source_error,
    })?;
    candidate_from_bytes(
        &bytes,
        source,
        normalize_path(path),
        source_trust(source),
        reader_version,
        path,
    )
}

/// Resolve a deterministic catalog from a completed scan and merged configuration.
///
/// # Errors
///
/// Resolution fails closed when the scan contains any issue, the configuration is invalid,
/// one source contains duplicate package IDs, or a configured source is unavailable.
pub fn resolve_scaffold_catalog(
    scan: &CatalogScan,
    config: &ScaffoldConfig,
    reader_version: &Version,
) -> Result<ResolvedCatalog, ScaffoldError> {
    if !scan.issues.is_empty() {
        let issues = scan
            .issues
            .iter()
            .map(|issue| ValidationIssue {
                code: issue.code.clone(),
                message: format!(
                    "{}:{}: {}",
                    issue.source.as_str(),
                    issue.locator,
                    issue.message
                ),
            })
            .collect::<Vec<_>>();
        return Err(resolution_error(&issues));
    }
    let config_issues = validate_config(config);
    if !config_issues.is_empty() {
        return Err(resolution_error(&config_issues));
    }

    let source_order = config
        .source_order
        .clone()
        .unwrap_or_else(|| DEFAULT_SOURCE_ORDER.to_vec());
    let mut by_id: BTreeMap<&str, Vec<&CatalogCandidate>> = BTreeMap::new();
    for candidate in &scan.candidates {
        by_id
            .entry(candidate.manifest.package.id.as_str())
            .or_default()
            .push(candidate);
    }
    for (id, forced_source) in &config.package_sources {
        let available = by_id.get(id.as_str()).is_some_and(|candidates| {
            candidates
                .iter()
                .any(|candidate| candidate.source == *forced_source)
        });
        if !available {
            return Err(ScaffoldError::Resolution {
                issues: format!(
                    "package `{id}` is pinned to `{}` but that source is unavailable",
                    forced_source.as_str()
                ),
            });
        }
    }

    let mut packages = Vec::new();
    let mut shadowed = Vec::new();
    let mut warnings = Vec::new();
    for (id, candidates) in by_id {
        if config.package_enabled.get(id) == Some(&false) {
            continue;
        }
        let mut source_counts = BTreeMap::new();
        for candidate in &candidates {
            *source_counts.entry(candidate.source).or_insert(0_usize) += 1;
        }
        let duplicates = source_counts
            .iter()
            .filter_map(|(source, count)| (*count > 1).then_some((*source, *count)))
            .collect::<Vec<_>>();
        if !duplicates.is_empty() {
            let detail = duplicates
                .iter()
                .map(|(source, count)| format!("{}={count}", source.as_str()))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(ScaffoldError::Resolution {
                issues: format!("duplicate package id `{id}` within one source: {detail}"),
            });
        }

        let selected = if let Some(forced_source) = config.package_sources.get(id) {
            candidates
                .iter()
                .find(|candidate| candidate.source == *forced_source)
                .copied()
                .ok_or_else(|| ScaffoldError::Resolution {
                    issues: format!(
                        "package `{id}` is pinned to `{}` but that source is unavailable",
                        forced_source.as_str()
                    ),
                })?
        } else {
            source_order
                .iter()
                .find_map(|source| {
                    candidates
                        .iter()
                        .find(|candidate| candidate.source == *source)
                        .copied()
                })
                .ok_or_else(|| ScaffoldError::Resolution {
                    issues: format!("package `{id}` has no candidate in source_order"),
                })?
        };

        warnings.extend(selected.warnings.clone());
        packages.push(ResolvedPackage {
            source: selected.source,
            locator: selected.locator.clone(),
            manifest_sha256: selected.manifest_sha256.clone(),
            trust: selected.trust,
            manifest: selected.manifest.clone(),
            warnings: selected.warnings.clone(),
        });
        shadowed.extend(
            candidates
                .iter()
                .filter(|candidate| {
                    candidate.source != selected.source || candidate.locator != selected.locator
                })
                .map(|candidate| ShadowedPackage {
                    id: id.to_string(),
                    version: candidate.manifest.package.version.clone(),
                    source: candidate.source,
                    locator: candidate.locator.clone(),
                    selected_source: selected.source,
                }),
        );
    }

    packages.sort_by(|a, b| a.manifest.package.id.cmp(&b.manifest.package.id));
    shadowed.sort_by(|a, b| {
        a.id.cmp(&b.id)
            .then(a.source.cmp(&b.source))
            .then(a.locator.cmp(&b.locator))
    });
    warnings.sort_by(|a, b| a.code.cmp(&b.code).then(a.message.cmp(&b.message)));
    warnings.dedup();
    Ok(ResolvedCatalog {
        schema_version: SCAFFOLD_SCHEMA_VERSION,
        reader_version: reader_version.to_string(),
        source_order,
        packages,
        shadowed,
        warnings,
    })
}

fn scan_local_root(
    root: &Path,
    source: ScaffoldSource,
    reader_version: &Version,
    scan: &mut CatalogScan,
) {
    let exists = match root.try_exists() {
        Ok(exists) => exists,
        Err(error_value) => {
            issue(
                scan,
                source,
                normalize_path(root),
                "source_access_failed",
                error_value.to_string(),
            );
            return;
        }
    };
    if !exists {
        return;
    }
    let canonical_root = match root.canonicalize() {
        Ok(path) => path,
        Err(error_value) => {
            issue(
                scan,
                source,
                normalize_path(root),
                "source_canonicalize_failed",
                error_value.to_string(),
            );
            return;
        }
    };
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error_value) => {
            issue(
                scan,
                source,
                normalize_path(root),
                "source_read_failed",
                error_value.to_string(),
            );
            return;
        }
    };
    let mut entries = entries
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|error_value| {
            issue(
                scan,
                source,
                normalize_path(root),
                "source_entry_read_failed",
                error_value.to_string(),
            );
            Vec::new()
        });
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(error_value) => {
                issue(
                    scan,
                    source,
                    normalize_path(&entry.path()),
                    "package_type_read_failed",
                    error_value.to_string(),
                );
                continue;
            }
        };
        if !file_type.is_dir() && !file_type.is_symlink() {
            continue;
        }
        let manifest_path = entry.path().join(SCAFFOLD_MANIFEST_FILENAME);
        let locator = entry.file_name().to_string_lossy().replace('\\', "/");
        let manifest_exists = match manifest_path.try_exists() {
            Ok(exists) => exists,
            Err(error_value) => {
                issue(
                    scan,
                    source,
                    locator,
                    "manifest_access_failed",
                    error_value.to_string(),
                );
                continue;
            }
        };
        if !manifest_exists {
            issue(
                scan,
                source,
                locator,
                "manifest_missing",
                format!("direct package child is missing {SCAFFOLD_MANIFEST_FILENAME}"),
            );
            continue;
        }
        let canonical_manifest = match manifest_path.canonicalize() {
            Ok(path) => path,
            Err(error_value) => {
                issue(
                    scan,
                    source,
                    locator,
                    "manifest_canonicalize_failed",
                    error_value.to_string(),
                );
                continue;
            }
        };
        if !canonical_manifest.starts_with(&canonical_root) {
            issue(
                scan,
                source,
                locator,
                "manifest_path_escape",
                "manifest or package symlink escapes its configured source root",
            );
            continue;
        }
        let bytes = match fs::read(&canonical_manifest) {
            Ok(bytes) => bytes,
            Err(error_value) => {
                issue(
                    scan,
                    source,
                    locator,
                    "manifest_read_failed",
                    error_value.to_string(),
                );
                continue;
            }
        };
        match candidate_from_bytes(
            &bytes,
            source,
            locator.clone(),
            ScaffoldTrust::UntrustedLocal,
            reader_version,
            &manifest_path,
        ) {
            Ok(candidate) => scan.candidates.push(candidate),
            Err(error_value) => issue(
                scan,
                source,
                locator,
                error_code(&error_value),
                error_value.to_string(),
            ),
        }
    }
}

fn scan_official(reader_version: &Version, scan: &mut CatalogScan) {
    for document in official_manifest_documents() {
        let path = Path::new(document.locator);
        match candidate_from_bytes(
            document.json.as_bytes(),
            ScaffoldSource::Official,
            document.locator.to_string(),
            ScaffoldTrust::Official,
            reader_version,
            path,
        ) {
            Ok(candidate) => scan.candidates.push(candidate),
            Err(error_value) => issue(
                scan,
                ScaffoldSource::Official,
                document.locator,
                error_code(&error_value),
                error_value.to_string(),
            ),
        }
    }
}

fn candidate_from_bytes(
    bytes: &[u8],
    source: ScaffoldSource,
    locator: String,
    trust: ScaffoldTrust,
    reader_version: &Version,
    path: &Path,
) -> Result<CatalogCandidate, ScaffoldError> {
    let manifest = serde_json::from_slice::<ScaffoldManifest>(bytes).map_err(|source_error| {
        ScaffoldError::Parse {
            path: path.to_path_buf(),
            source: source_error,
        }
    })?;
    let validation = validate_manifest(&manifest, source, reader_version);
    ensure_valid(path, &validation)?;
    Ok(CatalogCandidate {
        source,
        locator,
        manifest_sha256: format!("{:x}", Sha256::digest(bytes)),
        trust,
        manifest,
        warnings: validation.warnings,
    })
}

fn ensure_valid(path: &Path, validation: &ManifestValidation) -> Result<(), ScaffoldError> {
    if validation.is_valid() {
        return Ok(());
    }
    Err(ScaffoldError::InvalidContract {
        path: path.to_path_buf(),
        issues: validation
            .errors
            .iter()
            .map(|issue| format!("{}: {}", issue.code, issue.message))
            .collect::<Vec<_>>()
            .join("; "),
    })
}

fn source_trust(source: ScaffoldSource) -> ScaffoldTrust {
    match source {
        ScaffoldSource::Official => ScaffoldTrust::Official,
        ScaffoldSource::Project | ScaffoldSource::User => ScaffoldTrust::UntrustedLocal,
    }
}

fn sort_scan(scan: &mut CatalogScan) {
    scan.candidates.sort_by(|a, b| {
        a.source
            .cmp(&b.source)
            .then(a.manifest.package.id.cmp(&b.manifest.package.id))
            .then(a.manifest.package.version.cmp(&b.manifest.package.version))
            .then(a.locator.cmp(&b.locator))
    });
    scan.issues.sort_by(|a, b| {
        a.source
            .cmp(&b.source)
            .then(a.locator.cmp(&b.locator))
            .then(a.code.cmp(&b.code))
    });
    scan.issues.dedup();
}

fn issue(
    scan: &mut CatalogScan,
    source: ScaffoldSource,
    locator: impl Into<String>,
    code: impl Into<String>,
    message: impl Into<String>,
) {
    scan.issues.push(CatalogIssue {
        source,
        locator: locator.into(),
        code: code.into(),
        message: message.into(),
    });
}

fn error_code(error_value: &ScaffoldError) -> &'static str {
    match error_value {
        ScaffoldError::Read { .. } => "manifest_read_failed",
        ScaffoldError::Parse { .. } => "manifest_parse_failed",
        ScaffoldError::InvalidContract { .. } => "manifest_contract_invalid",
        ScaffoldError::Resolution { .. } => "manifest_resolution_failed",
        ScaffoldError::WriteLock { .. } => "manifest_lock_write_failed",
        ScaffoldError::Generation { .. } => "scaffold_generation_failed",
        ScaffoldError::WriteGeneration { .. } => "scaffold_generation_write_failed",
    }
}

fn resolution_error(issues: &[ValidationIssue]) -> ScaffoldError {
    ScaffoldError::Resolution {
        issues: issues
            .iter()
            .map(|issue| format!("{}: {}", issue.code, issue.message))
            .collect::<Vec<_>>()
            .join("; "),
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Return all source kinds exactly once for CLI parsing and schema tooling.
#[must_use]
pub fn scaffold_source_kinds() -> BTreeSet<ScaffoldSource> {
    DEFAULT_SOURCE_ORDER.into_iter().collect()
}
