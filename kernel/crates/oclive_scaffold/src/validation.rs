use std::{collections::BTreeSet, path::Path};

use semver::{Version, VersionReq};

use crate::{
    CommandEntry, GeneratorDriver, ManifestValidation, ScaffoldConfig, ScaffoldManifest,
    ScaffoldSource, ValidationIssue, SCAFFOLD_CONFIG_SCHEMA_VERSION, SCAFFOLD_CONTRACT_VERSION,
    SCAFFOLD_SCHEMA_VERSION,
};

const ALLOWED_PERMISSIONS: &[&str] = &[
    "project.read",
    "project.write",
    "user_config.read",
    "environment.read",
    "process.spawn",
    "network.connect",
];
const SUPPORTED_EXTENSIONS: &[&str] = &["com.oclive.scaffold.v1"];

/// Strictly validate a parsed v1 manifest for its discovery source.
#[must_use]
pub fn validate_manifest(
    manifest: &ScaffoldManifest,
    source: ScaffoldSource,
    reader_version: &Version,
) -> ManifestValidation {
    let mut result = ManifestValidation::default();
    if manifest.schema_version != SCAFFOLD_SCHEMA_VERSION {
        error(
            &mut result,
            "unsupported_schema_version",
            format!(
                "schema_version {} is unsupported; upgrade the reader or migrate the package to v{}",
                manifest.schema_version, SCAFFOLD_SCHEMA_VERSION
            ),
        );
    }
    if !valid_namespace(&manifest.package.id) {
        error(
            &mut result,
            "invalid_package_id",
            "package.id must be a lowercase reverse-domain namespace with at least three segments",
        );
    }
    if Version::parse(&manifest.package.version).is_err() {
        error(
            &mut result,
            "invalid_package_version",
            "package.version must be valid SemVer",
        );
    }
    for (field, value) in [
        ("package.display_name", &manifest.package.display_name),
        ("package.description", &manifest.package.description),
        ("package.maintainer", &manifest.package.maintainer),
    ] {
        if value.trim().is_empty() {
            error(
                &mut result,
                "empty_identity_field",
                format!("{field} cannot be empty"),
            );
        }
    }
    validate_compatibility(manifest, reader_version, &mut result);

    if !valid_namespace(&manifest.command_namespace) {
        error(
            &mut result,
            "invalid_command_namespace",
            "command_namespace must be a lowercase reverse-domain namespace",
        );
    }
    if source != ScaffoldSource::Official && manifest.command_namespace.starts_with("com.oclive.") {
        error(
            &mut result,
            "reserved_command_namespace",
            "third-party packages cannot use the reserved com.oclive.* namespace",
        );
    }
    if source != ScaffoldSource::Official && manifest.package.id.starts_with("com.oclive.") {
        error(
            &mut result,
            "reserved_package_namespace",
            "third-party packages cannot use the reserved com.oclive.* package namespace",
        );
    }

    let package_permissions = validate_permissions(&manifest.permissions, &mut result);
    let mut generator_ids = BTreeSet::new();
    for generator in &manifest.generators {
        if !valid_component_id(&generator.id) || !generator_ids.insert(generator.id.as_str()) {
            error(
                &mut result,
                "invalid_or_duplicate_generator",
                format!("generator id `{}` is invalid or duplicated", generator.id),
            );
        }
        if !valid_component_id(&generator.kind) {
            error(
                &mut result,
                "invalid_generator_kind",
                format!("generator kind `{}` is invalid", generator.kind),
            );
        }
        match &generator.driver {
            GeneratorDriver::Builtin { target } => {
                if source != ScaffoldSource::Official {
                    error(
                        &mut result,
                        "reserved_builtin_driver",
                        "only compiled official packages may reference builtin generators",
                    );
                }
                validate_builtin_target(target, &mut result);
            }
            GeneratorDriver::Instruction { path, sha256 } => {
                validate_relative_path(path, "generator instruction", &mut result);
                match sha256 {
                    Some(value) if !valid_sha256(value) => error(
                        &mut result,
                        "invalid_instruction_digest",
                        format!(
                            "generator `{}` instruction.sha256 must be 64 lowercase hexadecimal characters",
                            generator.id
                        ),
                    ),
                    None => warning(
                        &mut result,
                        "instruction_digest_required_for_generation",
                        format!(
                            "generator `{}` remains discoverable but cannot run in Stage 2B until instruction.sha256 is added and scaffold_contract is raised to >=1.1,<2",
                            generator.id
                        ),
                    ),
                    Some(_) => {}
                }
            }
        }
    }

    let mut command_names = BTreeSet::new();
    for command in &manifest.commands {
        if !valid_component_id(&command.name) || !command_names.insert(command.name.as_str()) {
            error(
                &mut result,
                "invalid_or_duplicate_command",
                format!("command name `{}` is invalid or duplicated", command.name),
            );
        }
        if command.description.trim().is_empty() {
            error(
                &mut result,
                "empty_command_description",
                format!("command `{}` must have a description", command.name),
            );
        }
        let command_permissions = validate_permissions(&command.permissions, &mut result);
        for permission in command_permissions.difference(&package_permissions) {
            error(
                &mut result,
                "undeclared_command_permission",
                format!(
                    "command `{}` requests `{permission}` outside package.permissions",
                    command.name
                ),
            );
        }
        match &command.entry {
            CommandEntry::Builtin { target } => {
                if source != ScaffoldSource::Official {
                    error(
                        &mut result,
                        "reserved_builtin_command",
                        "only compiled official packages may reference builtin commands",
                    );
                }
                validate_builtin_target(target, &mut result);
            }
            CommandEntry::Script { path, .. } => {
                validate_relative_path(path, "command script", &mut result);
            }
        }
    }

    validate_references(manifest, &mut result);
    for (namespace, extension) in &manifest.extensions {
        if !valid_namespace(namespace) {
            error(
                &mut result,
                "invalid_extension_namespace",
                format!("extension namespace `{namespace}` is invalid"),
            );
        } else if !SUPPORTED_EXTENSIONS.contains(&namespace.as_str()) {
            if extension.required {
                error(
                    &mut result,
                    "unsupported_required_extension",
                    format!("required extension `{namespace}` is not supported"),
                );
            } else {
                warning(
                    &mut result,
                    "unsupported_optional_extension",
                    format!("optional extension `{namespace}` is preserved but not interpreted"),
                );
            }
        }
    }
    if !manifest.dependencies.is_empty()
        || !manifest.extends.is_empty()
        || !manifest.composition.is_empty()
    {
        warning(
            &mut result,
            "composition_declarations_not_executed",
            "dependencies, extends, and composition are reserved declarations in Stage 2A and are not resolved or executed",
        );
    }
    if source != ScaffoldSource::Official {
        let permissions = if manifest.permissions.is_empty() {
            "none".to_string()
        } else {
            manifest.permissions.join(", ")
        };
        warning(
            &mut result,
            "untrusted_local_scaffold",
            format!(
                "{} package `{}` is maintained by `{}` and requests [{}]; Stage 2A does not execute it, OCLive does not endorse third-party behavior, and the package cannot control CI",
                source.as_str(),
                manifest.package.id,
                manifest.package.maintainer,
                permissions
            ),
        );
    }
    result
        .errors
        .sort_by(|a, b| a.code.cmp(&b.code).then(a.message.cmp(&b.message)));
    result.errors.dedup();
    result
        .warnings
        .sort_by(|a, b| a.code.cmp(&b.code).then(a.message.cmp(&b.message)));
    result.warnings.dedup();
    result
}

/// Validate discovery configuration before applying it.
#[must_use]
pub fn validate_config(config: &ScaffoldConfig) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    if config.schema_version != SCAFFOLD_CONFIG_SCHEMA_VERSION {
        issues.push(ValidationIssue {
            code: "unsupported_config_schema_version".to_string(),
            message: format!(
                "scaffold config schema_version {} is unsupported",
                config.schema_version
            ),
        });
    }
    if let Some(order) = &config.source_order {
        let unique = order.iter().copied().collect::<BTreeSet<_>>();
        if order.len() != 3 || unique.len() != 3 {
            issues.push(ValidationIssue {
                code: "invalid_source_order".to_string(),
                message: "source_order must contain project, user, and official exactly once"
                    .to_string(),
            });
        }
    }
    for id in config
        .package_sources
        .keys()
        .chain(config.package_enabled.keys())
    {
        if !valid_namespace(id) {
            issues.push(ValidationIssue {
                code: "invalid_config_package_id".to_string(),
                message: format!("configured package id `{id}` is invalid"),
            });
        }
    }
    issues.sort_by(|a, b| a.code.cmp(&b.code).then(a.message.cmp(&b.message)));
    issues.dedup();
    issues
}

fn validate_compatibility(
    manifest: &ScaffoldManifest,
    reader_version: &Version,
    result: &mut ManifestValidation,
) {
    match VersionReq::parse(&manifest.compatibility.oclive_cli) {
        Ok(requirement) if requirement.matches(reader_version) => {}
        Ok(_) => error(
            result,
            "incompatible_cli_version",
            format!(
                "package requires oclive-cli `{}` but reader is `{reader_version}`",
                manifest.compatibility.oclive_cli
            ),
        ),
        Err(error_value) => error(
            result,
            "invalid_cli_version_requirement",
            format!("invalid oclive_cli requirement: {error_value}"),
        ),
    }
    let contract_version = match Version::parse(SCAFFOLD_CONTRACT_VERSION) {
        Ok(version) => version,
        Err(error_value) => {
            error(
                result,
                "invalid_reader_contract_version",
                format!("compiled scaffold contract version is invalid: {error_value}"),
            );
            return;
        }
    };
    match VersionReq::parse(&manifest.compatibility.scaffold_contract) {
        Ok(requirement) if requirement.matches(&contract_version) => {}
        Ok(_) => error(
            result,
            "incompatible_scaffold_contract",
            format!(
                "package requires scaffold contract `{}` but reader implements `{contract_version}`",
                manifest.compatibility.scaffold_contract
            ),
        ),
        Err(error_value) => error(
            result,
            "invalid_scaffold_contract_requirement",
            format!("invalid scaffold_contract requirement: {error_value}"),
        ),
    }
}

fn validate_permissions(
    permissions: &[String],
    result: &mut ManifestValidation,
) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    for permission in permissions {
        if permission.starts_with("ci.") || permission == "ci" {
            error(
                result,
                "forbidden_ci_capability",
                format!("scaffolds cannot request CI capability `{permission}`"),
            );
        } else if !ALLOWED_PERMISSIONS.contains(&permission.as_str()) {
            error(
                result,
                "unknown_permission",
                format!("permission `{permission}` is not part of the v1 allowlist"),
            );
        }
        if !seen.insert(permission.clone()) {
            error(
                result,
                "duplicate_permission",
                format!("permission `{permission}` is duplicated"),
            );
        }
    }
    seen
}

fn validate_references(manifest: &ScaffoldManifest, result: &mut ManifestValidation) {
    let mut dependency_ids = BTreeSet::new();
    for (label, references) in [
        ("dependency", manifest.dependencies.as_slice()),
        ("extends", manifest.extends.as_slice()),
    ] {
        for reference in references {
            if !valid_namespace(&reference.id) {
                error(
                    result,
                    "invalid_package_reference",
                    format!("{label} id `{}` is invalid", reference.id),
                );
            }
            if reference.id == manifest.package.id {
                error(
                    result,
                    "self_package_reference",
                    format!("package cannot {label} itself"),
                );
            }
            if VersionReq::parse(&reference.version).is_err() {
                error(
                    result,
                    "invalid_package_reference_version",
                    format!(
                        "{label} `{}` has an invalid version requirement",
                        reference.id
                    ),
                );
            }
            let key = format!("{label}:{}", reference.id);
            if !dependency_ids.insert(key) {
                error(
                    result,
                    "duplicate_package_reference",
                    format!("{label} `{}` is duplicated", reference.id),
                );
            }
        }
    }
    for id in manifest
        .composition
        .order_before
        .iter()
        .chain(&manifest.composition.order_after)
    {
        if !valid_namespace(id) || id == &manifest.package.id {
            error(
                result,
                "invalid_composition_reference",
                format!("composition reference `{id}` is invalid"),
            );
        }
    }
    let mut conflicts = BTreeSet::new();
    for group in &manifest.composition.conflict_groups {
        if !valid_component_id(group) || !conflicts.insert(group) {
            error(
                result,
                "invalid_or_duplicate_conflict_group",
                format!("conflict group `{group}` is invalid or duplicated"),
            );
        }
    }
}

fn validate_builtin_target(target: &str, result: &mut ManifestValidation) {
    if target.trim().is_empty() || target == "ci" || target.starts_with("ci ") {
        error(
            result,
            "invalid_builtin_target",
            "builtin target cannot be empty or reference the reserved ci command",
        );
    }
}

fn validate_relative_path(path: &str, label: &str, result: &mut ManifestValidation) {
    let value = Path::new(path);
    let unsafe_component = value.components().any(|component| {
        matches!(
            component,
            std::path::Component::Prefix(_)
                | std::path::Component::RootDir
                | std::path::Component::ParentDir
        )
    });
    if path.trim().is_empty() || value.is_absolute() || unsafe_component {
        error(
            result,
            "unsafe_relative_path",
            format!("{label} path `{path}` must stay inside the package root"),
        );
    }
}

fn valid_namespace(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() >= 3 && parts.iter().all(|part| valid_component_id(part))
}

pub(crate) fn valid_component_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        })
}

pub(crate) fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn error(result: &mut ManifestValidation, code: &str, message: impl Into<String>) {
    result.errors.push(ValidationIssue {
        code: code.to_string(),
        message: message.into(),
    });
}

fn warning(result: &mut ManifestValidation, code: &str, message: impl Into<String>) {
    result.warnings.push(ValidationIssue {
        code: code.to_string(),
        message: message.into(),
    });
}
