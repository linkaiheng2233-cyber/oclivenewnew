//! Deterministic, fail-safe CI impact planning for the OCLive monorepo.

mod model;

pub use model::*;

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    path::{Component, Path, PathBuf},
};

use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CiPlanError {
    #[error("failed to read CI contract {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse CI contract {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid CI contract {path}: {message}")]
    InvalidContract { path: PathBuf, message: String },
    #[error("unknown validation policy `{0}`")]
    UnknownPolicy(String),
}

#[derive(Debug, Clone)]
struct LoadedDescriptor {
    descriptor: Option<ModuleDescriptor>,
    issues: Vec<String>,
}

/// A loaded, validated set of CI planning contracts.
#[derive(Debug, Clone)]
pub struct Planner {
    impact_map: ImpactMap,
    catalog: ValidationCatalog,
    descriptors: BTreeMap<String, LoadedDescriptor>,
    warnings: Vec<String>,
    impact_map_sha256: String,
    validation_catalog_sha256: String,
}

impl Planner {
    /// Load the centrally owned contracts and their referenced module descriptors.
    ///
    /// Central map/catalog corruption is an error because a trustworthy full set cannot be
    /// derived. A broken module descriptor is retained as a planning issue and forces the
    /// resulting plan to the active policy's full validation set.
    ///
    /// # Errors
    ///
    /// Returns an error when central contracts cannot be read, parsed, or validated.
    pub fn load(
        repo_root: impl AsRef<Path>,
        impact_map_path: impl AsRef<Path>,
        validation_catalog_path: impl AsRef<Path>,
    ) -> Result<Self, CiPlanError> {
        let repo_root = repo_root.as_ref();
        let impact_map_path = resolve_path(repo_root, impact_map_path.as_ref());
        let catalog_path = resolve_path(repo_root, validation_catalog_path.as_ref());
        let (impact_map, impact_bytes) = read_json::<ImpactMap>(&impact_map_path)?;
        let (catalog, catalog_bytes) = read_json::<ValidationCatalog>(&catalog_path)?;

        validate_catalog(&catalog, &catalog_path)?;
        validate_impact_map(&impact_map, &catalog, &impact_map_path)?;

        let module_ids = impact_map
            .module_bindings
            .iter()
            .map(|binding| binding.module_id.clone())
            .collect::<BTreeSet<_>>();
        let supported_extensions = impact_map
            .supported_extensions
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let profile_ids = catalog
            .profiles
            .iter()
            .map(|profile| profile.id.clone())
            .collect::<BTreeSet<_>>();

        let mut descriptors = BTreeMap::new();
        let mut warnings = Vec::new();
        for binding in &impact_map.module_bindings {
            let descriptor_path =
                resolve_repo_relative(repo_root, &binding.descriptor).map_err(|message| {
                    CiPlanError::InvalidContract {
                        path: impact_map_path.clone(),
                        message: format!("descriptor for `{}`: {message}", binding.module_id),
                    }
                })?;
            let loaded = load_descriptor(
                &descriptor_path,
                &binding.module_id,
                &module_ids,
                &profile_ids,
                &supported_extensions,
                &mut warnings,
            );
            descriptors.insert(binding.module_id.clone(), loaded);
        }

        warnings.sort();
        warnings.dedup();
        Ok(Self {
            impact_map,
            catalog,
            descriptors,
            warnings,
            impact_map_sha256: sha256_hex(&impact_bytes),
            validation_catalog_sha256: sha256_hex(&catalog_bytes),
        })
    }

    /// Build a deterministic plan for a set of changed repository paths.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested validation policy does not exist.
    pub fn plan(&self, request: PlanRequest) -> Result<CiPlan, CiPlanError> {
        let policy = self
            .catalog
            .policies
            .iter()
            .find(|policy| policy.id == request.policy)
            .ok_or_else(|| CiPlanError::UnknownPolicy(request.policy.clone()))?;
        let included_tiers = policy
            .included_tiers
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();

        let mut changed_files = BTreeSet::new();
        let mut fallback_reasons = BTreeSet::new();
        for changed_file in request.changed_files {
            match normalize_repo_path(&changed_file) {
                Ok(path) => {
                    changed_files.insert(path);
                }
                Err(message) => {
                    fallback_reasons
                        .insert(format!("invalid_changed_path:{changed_file}:{message}"));
                }
            }
        }
        if changed_files.is_empty() {
            fallback_reasons.insert("no_changed_files".to_owned());
        }

        for (module_id, loaded) in &self.descriptors {
            for issue in &loaded.issues {
                fallback_reasons.insert(format!("module_metadata:{module_id}:{issue}"));
            }
        }

        let mut direct = BTreeMap::<String, BTreeSet<String>>::new();
        for changed_file in &changed_files {
            let mut matched = false;
            for binding in &self.impact_map.module_bindings {
                for selector in &binding.selectors {
                    if selector_matches(selector, changed_file) {
                        matched = true;
                        direct
                            .entry(binding.module_id.clone())
                            .or_default()
                            .insert(format!(
                                "changed_path:{changed_file}:{}:{}",
                                selector_kind_name(selector.kind),
                                selector.value
                            ));
                    }
                }
            }
            if !matched {
                fallback_reasons.insert(format!("unmapped_changed_path:{changed_file}"));
            }
        }

        let mut forced_profiles = BTreeMap::<String, BTreeSet<String>>::new();
        for risk in &self.impact_map.risk_overrides {
            let matches = changed_files.iter().any(|path| {
                risk.selectors
                    .iter()
                    .any(|selector| selector_matches(selector, path))
            });
            if !matches {
                continue;
            }
            if risk.full {
                fallback_reasons.insert(format!("risk_override:{}:{}", risk.id, risk.reason));
            }
            for profile in &risk.force_profiles {
                forced_profiles
                    .entry(profile.clone())
                    .or_default()
                    .insert(format!("risk_override:{}:{}", risk.id, risk.reason));
            }
        }

        let mut affected = direct
            .keys()
            .map(|id| (id.clone(), BTreeSet::from(["direct_change".to_owned()])))
            .collect::<BTreeMap<_, _>>();
        let mut queue = direct.keys().cloned().collect::<VecDeque<_>>();
        let mut visited = BTreeSet::new();
        while let Some(source) = queue.pop_front() {
            if !visited.insert(source.clone()) {
                continue;
            }
            let mut targets = BTreeMap::<String, BTreeSet<String>>::new();
            if let Some(policy_targets) = self.impact_map.policy_affects.get(&source) {
                for target in policy_targets {
                    targets
                        .entry(target.clone())
                        .or_default()
                        .insert(format!("policy_affects:{source}"));
                }
            }
            if let Some(Some(descriptor)) = self
                .descriptors
                .get(&source)
                .map(|loaded| loaded.descriptor.as_ref())
            {
                for target in &descriptor.declared_affects {
                    targets
                        .entry(target.clone())
                        .or_default()
                        .insert(format!("declared_affects:{source}"));
                }
            }
            for (target, reasons) in targets {
                let was_new = !affected.contains_key(&target);
                affected.entry(target.clone()).or_default().extend(reasons);
                if was_new {
                    queue.push_back(target);
                }
            }
        }

        let full = !fallback_reasons.is_empty();
        if full {
            for module_id in self.descriptors.keys() {
                affected
                    .entry(module_id.clone())
                    .or_default()
                    .insert("full_fallback".to_owned());
            }
        }

        let mut selected_profiles = forced_profiles;
        if full {
            for profile in &self.catalog.profiles {
                selected_profiles
                    .entry(profile.id.clone())
                    .or_default()
                    .insert("full_fallback".to_owned());
            }
        } else {
            for module_id in affected.keys() {
                if let Some(Some(descriptor)) = self
                    .descriptors
                    .get(module_id)
                    .map(|loaded| loaded.descriptor.as_ref())
                {
                    for profile in &descriptor.validation_profiles {
                        selected_profiles
                            .entry(profile.clone())
                            .or_default()
                            .insert(format!("affected_module:{module_id}"));
                    }
                }
            }
        }

        let profiles_by_id = self
            .catalog
            .profiles
            .iter()
            .map(|profile| (profile.id.as_str(), profile))
            .collect::<BTreeMap<_, _>>();
        let mut validator_reasons = BTreeMap::<String, BTreeSet<String>>::new();
        for profile_id in selected_profiles.keys() {
            if let Some(profile) = profiles_by_id.get(profile_id.as_str()) {
                for validator_id in &profile.validators {
                    validator_reasons
                        .entry(validator_id.clone())
                        .or_default()
                        .insert(format!("validation_profile:{profile_id}"));
                }
            }
        }
        if full {
            for validator in &self.catalog.validators {
                if included_tiers.contains(&validator.tier) {
                    validator_reasons
                        .entry(validator.id.clone())
                        .or_default()
                        .insert("full_fallback".to_owned());
                }
            }
        }

        let mut selected_validators = Vec::new();
        let mut skipped_validators = Vec::new();
        for validator in &self.catalog.validators {
            if !included_tiers.contains(&validator.tier) {
                skipped_validators.push(SkippedValidator {
                    id: validator.id.clone(),
                    reason: format!("tier_not_in_policy:{}", tier_name(validator.tier)),
                });
                continue;
            }
            if let Some(reasons) = validator_reasons.get(&validator.id) {
                selected_validators.push(PlannedValidator {
                    id: validator.id.clone(),
                    tier: validator.tier,
                    gate: validator.gate,
                    platforms: sorted_unique(validator.platforms.clone()),
                    trust: validator.trust,
                    command_id: validator.command_id.clone(),
                    workflow_jobs: sorted_unique(validator.workflow_jobs.clone()),
                    reasons: reasons.iter().cloned().collect(),
                });
            } else {
                skipped_validators.push(SkippedValidator {
                    id: validator.id.clone(),
                    reason: "profile_not_selected".to_owned(),
                });
            }
        }
        selected_validators.sort_by(|left, right| left.id.cmp(&right.id));
        skipped_validators.sort_by(|left, right| left.id.cmp(&right.id));

        Ok(CiPlan {
            schema_version: PLAN_SCHEMA_VERSION,
            base_sha: request.base_sha,
            head_sha: request.head_sha,
            policy: request.policy,
            shadow: request.shadow,
            changed_files: changed_files.into_iter().collect(),
            direct_modules: reasoned_selections(direct),
            affected_modules: reasoned_selections(affected),
            selected_profiles: reasoned_selections(selected_profiles),
            selected_validators,
            skipped_validators,
            fallback: FallbackDecision {
                full,
                reasons: fallback_reasons.into_iter().collect(),
            },
            warnings: self.warnings.clone(),
            impact_map_sha256: self.impact_map_sha256.clone(),
            validation_catalog_sha256: self.validation_catalog_sha256.clone(),
        })
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<(T, Vec<u8>), CiPlanError> {
    let bytes = fs::read(path).map_err(|source| CiPlanError::Read {
        path: path.to_owned(),
        source,
    })?;
    let value = serde_json::from_slice(&bytes).map_err(|source| CiPlanError::Parse {
        path: path.to_owned(),
        source,
    })?;
    Ok((value, bytes))
}

fn load_descriptor(
    path: &Path,
    expected_module_id: &str,
    module_ids: &BTreeSet<String>,
    profile_ids: &BTreeSet<String>,
    supported_extensions: &BTreeSet<String>,
    warnings: &mut Vec<String>,
) -> LoadedDescriptor {
    let (descriptor, _) = match read_json::<ModuleDescriptor>(path) {
        Ok(value) => value,
        Err(error) => {
            return LoadedDescriptor {
                descriptor: None,
                issues: vec![descriptor_error_code(&error)],
            };
        }
    };
    let mut issues = Vec::new();
    if descriptor.schema_version != CONTRACT_SCHEMA_VERSION {
        issues.push(format!(
            "unsupported_schema_version:{}",
            descriptor.schema_version
        ));
    }
    if descriptor.module.id != expected_module_id {
        issues.push(format!(
            "module_id_mismatch:expected={expected_module_id}:actual={}",
            descriptor.module.id
        ));
    }
    if !valid_id(&descriptor.module.id) {
        issues.push(format!("invalid_module_id:{}", descriptor.module.id));
    }
    if descriptor.module.kind.trim().is_empty() {
        issues.push("empty_module_kind".to_owned());
    }
    validate_unique_values("provides", &descriptor.provides, &mut issues);
    validate_unique_values(
        "runtime_requires",
        &descriptor.runtime_requires,
        &mut issues,
    );
    validate_unique_values(
        "declared_affects",
        &descriptor.declared_affects,
        &mut issues,
    );
    validate_unique_values(
        "validation_profiles",
        &descriptor.validation_profiles,
        &mut issues,
    );
    validate_unique_values("platforms", &descriptor.platforms, &mut issues);
    for claim in &descriptor.resource_claims {
        if claim.resource.trim().is_empty() || claim.mode.trim().is_empty() {
            issues.push("invalid_resource_claim".to_owned());
        }
    }
    for target in &descriptor.declared_affects {
        if !module_ids.contains(target) {
            issues.push(format!("unknown_declared_affect:{target}"));
        }
    }
    for profile in &descriptor.validation_profiles {
        if !profile_ids.contains(profile) {
            issues.push(format!("unknown_validation_profile:{profile}"));
        }
    }
    for namespace in descriptor.extensions.keys() {
        if !valid_namespace(namespace) {
            issues.push(format!("invalid_extension_namespace:{namespace}"));
            continue;
        }
        if supported_extensions.contains(namespace) {
            continue;
        }
        let extension = &descriptor.extensions[namespace];
        if extension.required {
            issues.push(format!("unsupported_required_extension:{namespace}"));
        } else {
            warnings.push(format!(
                "module `{expected_module_id}` preserves unsupported optional extension `{namespace}`"
            ));
        }
    }
    LoadedDescriptor {
        descriptor: Some(descriptor),
        issues,
    }
}

fn descriptor_error_code(error: &CiPlanError) -> String {
    match error {
        CiPlanError::Read { source, .. } => format!("descriptor_read_error:{:?}", source.kind()),
        CiPlanError::Parse { source, .. } => {
            format!(
                "descriptor_parse_error:{}:{}",
                source.line(),
                source.column()
            )
        }
        CiPlanError::InvalidContract { message, .. } => {
            format!("descriptor_invalid_contract:{message}")
        }
        CiPlanError::UnknownPolicy(policy) => format!("descriptor_unknown_policy:{policy}"),
    }
}

fn validate_catalog(catalog: &ValidationCatalog, path: &Path) -> Result<(), CiPlanError> {
    if catalog.schema_version != CONTRACT_SCHEMA_VERSION {
        return invalid(
            path,
            format!("unsupported schema_version {}", catalog.schema_version),
        );
    }
    let policy_ids = unique_ids(
        path,
        "policy",
        catalog.policies.iter().map(|value| &value.id),
    )?;
    let profile_ids = unique_ids(
        path,
        "profile",
        catalog.profiles.iter().map(|value| &value.id),
    )?;
    let validator_ids = unique_ids(
        path,
        "validator",
        catalog.validators.iter().map(|value| &value.id),
    )?;
    let command_ids = unique_ids(
        path,
        "command",
        catalog.commands.iter().map(|value| &value.id),
    )?;
    if policy_ids.is_empty()
        || profile_ids.is_empty()
        || validator_ids.is_empty()
        || command_ids.is_empty()
    {
        return invalid(
            path,
            "policies, profiles, validators, and commands must be non-empty",
        );
    }
    for policy in &catalog.policies {
        if policy.included_tiers.is_empty() {
            return invalid(path, format!("policy `{}` has no tiers", policy.id));
        }
    }
    for profile in &catalog.profiles {
        if profile.validators.is_empty() {
            return invalid(path, format!("profile `{}` has no validators", profile.id));
        }
        for validator in &profile.validators {
            if !validator_ids.contains(validator.as_str()) {
                return invalid(
                    path,
                    format!(
                        "profile `{}` references unknown validator `{validator}`",
                        profile.id
                    ),
                );
            }
        }
    }
    for validator in &catalog.validators {
        if !command_ids.contains(validator.command_id.as_str()) {
            return invalid(
                path,
                format!(
                    "validator `{}` references unknown command `{}`",
                    validator.id, validator.command_id
                ),
            );
        }
        if validator.workflow_jobs.is_empty() {
            return invalid(
                path,
                format!("validator `{}` has no workflow_jobs", validator.id),
            );
        }
        unique_strings(path, "workflow job", &validator.workflow_jobs)?;
    }
    for command in &catalog.commands {
        if command.program.trim().is_empty() {
            return invalid(
                path,
                format!("command `{}` has an empty program", command.id),
            );
        }
        if let Some(directory) = &command.working_directory {
            normalize_repo_path(directory).map_err(|message| CiPlanError::InvalidContract {
                path: path.to_owned(),
                message: format!("command `{}` working_directory: {message}", command.id),
            })?;
        }
    }
    Ok(())
}

fn validate_impact_map(
    impact_map: &ImpactMap,
    catalog: &ValidationCatalog,
    path: &Path,
) -> Result<(), CiPlanError> {
    if impact_map.schema_version != CONTRACT_SCHEMA_VERSION {
        return invalid(
            path,
            format!("unsupported schema_version {}", impact_map.schema_version),
        );
    }
    let module_ids = unique_ids(
        path,
        "module binding",
        impact_map
            .module_bindings
            .iter()
            .map(|value| &value.module_id),
    )?;
    let profile_ids = catalog
        .profiles
        .iter()
        .map(|value| value.id.as_str())
        .collect::<BTreeSet<_>>();
    if module_ids.is_empty() {
        return invalid(path, "module_bindings must be non-empty");
    }
    unique_strings(
        path,
        "supported extension",
        &impact_map.supported_extensions,
    )?;
    for namespace in &impact_map.supported_extensions {
        if !valid_namespace(namespace) {
            return invalid(path, format!("invalid extension namespace `{namespace}`"));
        }
    }
    for binding in &impact_map.module_bindings {
        if binding.selectors.is_empty() {
            return invalid(
                path,
                format!("module `{}` has no path selectors", binding.module_id),
            );
        }
        normalize_repo_path(&binding.descriptor).map_err(|message| {
            CiPlanError::InvalidContract {
                path: path.to_owned(),
                message: format!("module `{}` descriptor: {message}", binding.module_id),
            }
        })?;
        for selector in &binding.selectors {
            validate_selector(path, selector)?;
        }
    }
    for (source, targets) in &impact_map.policy_affects {
        if !module_ids.contains(source.as_str()) {
            return invalid(
                path,
                format!("policy_affects has unknown source `{source}`"),
            );
        }
        for target in targets {
            if !module_ids.contains(target.as_str()) {
                return invalid(
                    path,
                    format!("policy_affects `{source}` has unknown target `{target}`"),
                );
            }
        }
    }
    let mut risk_ids = BTreeSet::new();
    for risk in &impact_map.risk_overrides {
        if !risk_ids.insert(risk.id.as_str()) || !valid_id(&risk.id) {
            return invalid(
                path,
                format!("duplicate or invalid risk override `{}`", risk.id),
            );
        }
        if risk.selectors.is_empty() || (!risk.full && risk.force_profiles.is_empty()) {
            return invalid(path, format!("risk override `{}` has no effect", risk.id));
        }
        for selector in &risk.selectors {
            validate_selector(path, selector)?;
        }
        for profile in &risk.force_profiles {
            if !profile_ids.contains(profile.as_str()) {
                return invalid(
                    path,
                    format!(
                        "risk override `{}` references unknown profile `{profile}`",
                        risk.id
                    ),
                );
            }
        }
    }
    Ok(())
}

fn validate_selector(path: &Path, selector: &PathSelector) -> Result<(), CiPlanError> {
    normalize_repo_path(&selector.value).map_err(|message| CiPlanError::InvalidContract {
        path: path.to_owned(),
        message: format!("invalid path selector `{}`: {message}", selector.value),
    })?;
    Ok(())
}

fn unique_ids<'a>(
    path: &Path,
    label: &str,
    values: impl Iterator<Item = &'a String>,
) -> Result<BTreeSet<&'a str>, CiPlanError> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !valid_id(value) || !seen.insert(value.as_str()) {
            return invalid(path, format!("duplicate or invalid {label} id `{value}`"));
        }
    }
    Ok(seen)
}

fn unique_strings(path: &Path, label: &str, values: &[String]) -> Result<(), CiPlanError> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return invalid(path, format!("duplicate {label} `{value}`"));
        }
    }
    Ok(())
}

fn validate_unique_values(label: &str, values: &[String], issues: &mut Vec<String>) {
    let mut seen = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() {
            issues.push(format!("empty_{label}"));
        } else if !seen.insert(value) {
            issues.push(format!("duplicate_{label}:{value}"));
        }
    }
}

fn resolve_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        repo_root.join(path)
    }
}

fn resolve_repo_relative(repo_root: &Path, value: &str) -> Result<PathBuf, String> {
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

fn normalize_repo_path(value: &str) -> Result<String, String> {
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

fn selector_matches(selector: &PathSelector, path: &str) -> bool {
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

fn selector_kind_name(kind: PathSelectorKind) -> &'static str {
    match kind {
        PathSelectorKind::Exact => "exact",
        PathSelectorKind::Prefix => "prefix",
    }
}

fn tier_name(tier: ValidationTier) -> &'static str {
    match tier {
        ValidationTier::Fast => "fast",
        ValidationTier::Pr => "pr",
        ValidationTier::Merge => "merge",
        ValidationTier::Nightly => "nightly",
        ValidationTier::Release => "release",
    }
}

fn reasoned_selections(values: BTreeMap<String, BTreeSet<String>>) -> Vec<ReasonedSelection> {
    values
        .into_iter()
        .map(|(id, reasons)| ReasonedSelection {
            id,
            reasons: reasons.into_iter().collect(),
        })
        .collect()
}

fn sorted_unique(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn valid_namespace(value: &str) -> bool {
    valid_id(value) && value.contains('.')
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn invalid<T>(path: &Path, message: impl Into<String>) -> Result<T, CiPlanError> {
    Err(CiPlanError::InvalidContract {
        path: path.to_owned(),
        message: message.into(),
    })
}

#[cfg(test)]
mod tests;
