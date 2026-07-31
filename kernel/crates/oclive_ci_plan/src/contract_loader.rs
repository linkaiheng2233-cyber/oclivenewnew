use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use crate::{
    path_rules::{
        invalid, normalize_repo_path, resolve_path, resolve_repo_relative, sha256_hex, valid_id,
        valid_namespace,
    },
    CiPlanError, ImpactMap, LoadedDescriptor, ModuleDescriptor, PathSelector, Planner,
    ValidationCatalog, CONTRACT_SCHEMA_VERSION,
};

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
