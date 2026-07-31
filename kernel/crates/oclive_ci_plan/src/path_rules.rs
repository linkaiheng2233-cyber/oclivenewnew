use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};

use sha2::{Digest, Sha256};

use crate::{CiPlanError, PathSelector, PathSelectorKind, ReasonedSelection, ValidationTier};

pub(crate) fn resolve_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        repo_root.join(path)
    }
}

pub(crate) fn resolve_repo_relative(repo_root: &Path, value: &str) -> Result<PathBuf, String> {
    let normalized = normalize_repo_path(value)?;
    let joined = repo_root.join(normalized.replace('/', std::path::MAIN_SEPARATOR_STR));
    let Ok(canonical) = fs::canonicalize(&joined) else {
        // Missing descriptors remain module metadata issues so the planner can emit a
        // reproducible full-fallback plan instead of losing all diagnostic output.
        return Ok(joined);
    };
    let canonical_root = fs::canonicalize(repo_root)
        .map_err(|error| format!("cannot resolve repository root: {error}"))?;
    if !canonical.starts_with(&canonical_root) {
        return Err("descriptor symlink escapes repository root".to_owned());
    }
    Ok(canonical)
}

pub(crate) fn normalize_repo_path(value: &str) -> Result<String, String> {
    let replaced = value.trim().replace('\\', "/");
    if replaced.is_empty() {
        return Err("path is empty".to_owned());
    }
    let path = Path::new(&replaced);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err("path must stay repository-relative".to_owned());
    }
    let normalized = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
            Component::CurDir => None,
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/");
    if normalized.is_empty() {
        return Err("path is empty after normalization".to_owned());
    }
    Ok(normalized.trim_end_matches('/').to_owned())
}

pub(crate) fn selector_matches(selector: &PathSelector, path: &str) -> bool {
    let selector_value = selector
        .value
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_owned();
    match selector.kind {
        PathSelectorKind::Exact => path == selector_value,
        PathSelectorKind::Prefix => {
            path == selector_value
                || path
                    .strip_prefix(&selector_value)
                    .is_some_and(|tail| tail.starts_with('/'))
        }
    }
}

pub(crate) fn selector_kind_name(kind: PathSelectorKind) -> &'static str {
    match kind {
        PathSelectorKind::Exact => "exact",
        PathSelectorKind::Prefix => "prefix",
    }
}

pub(crate) fn tier_name(tier: ValidationTier) -> &'static str {
    match tier {
        ValidationTier::Fast => "fast",
        ValidationTier::Pr => "pr",
        ValidationTier::Merge => "merge",
        ValidationTier::Nightly => "nightly",
        ValidationTier::Release => "release",
    }
}

pub(crate) fn reasoned_selections(
    values: BTreeMap<String, BTreeSet<String>>,
) -> Vec<ReasonedSelection> {
    values
        .into_iter()
        .map(|(id, reasons)| ReasonedSelection {
            id,
            reasons: reasons.into_iter().collect(),
        })
        .collect()
}

pub(crate) fn sorted_unique(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(crate) fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

pub(crate) fn valid_namespace(value: &str) -> bool {
    valid_id(value) && value.contains('.')
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(crate) fn invalid<T>(path: &Path, message: impl Into<String>) -> Result<T, CiPlanError> {
    Err(CiPlanError::InvalidContract {
        path: path.to_owned(),
        message: message.into(),
    })
}
