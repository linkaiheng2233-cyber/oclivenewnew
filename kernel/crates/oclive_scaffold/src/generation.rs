use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
};

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{
    valid_component_id, valid_sha256, GeneratorDriver, ResolvedPackage, ScaffoldError,
    ScaffoldGeneratedFile, ScaffoldGenerationPackage, ScaffoldGenerationPlan,
    ScaffoldGenerationProvenance, ScaffoldInstructionDocument, ScaffoldInstructionFileMode,
    ScaffoldLock, ScaffoldTrust, SCAFFOLD_CONTRACT_VERSION, SCAFFOLD_INSTRUCTION_SCHEMA_VERSION,
    SCAFFOLD_LOCK_SCHEMA_VERSION, SCAFFOLD_PROVENANCE_FILENAME, SCAFFOLD_PROVENANCE_SCHEMA_VERSION,
};

const PROVENANCE_RELATIVE_PATH: &str = ".oclive/scaffold.provenance.json";

/// Inputs for one bounded, local-only scaffold materialization.
pub struct ScaffoldGenerationRequest<'a> {
    pub package: &'a ResolvedPackage,
    pub package_root: &'a Path,
    pub generator_id: &'a str,
    pub output: &'a Path,
    pub variables: &'a BTreeMap<String, String>,
    pub lock: Option<&'a ScaffoldLock>,
    pub accept_untrusted: bool,
    pub dry_run: bool,
}

struct PreparedFile {
    target: PathBuf,
    bytes: Vec<u8>,
}

struct PreparedGeneration {
    output: PathBuf,
    files: Vec<PreparedFile>,
    plan: ScaffoldGenerationPlan,
}

/// Validate, render, and optionally atomically materialize one declarative generator.
///
/// This function never executes package commands, scripts, hooks, or network operations.
/// Untrusted packages require an exact resolution lock plus per-invocation acknowledgement.
///
/// # Errors
///
/// Returns [`ScaffoldError::Generation`] for policy, integrity, path, variable, or template
/// failures and [`ScaffoldError::WriteGeneration`] for filesystem materialization failures.
pub fn generate_scaffold(
    request: &ScaffoldGenerationRequest<'_>,
) -> Result<ScaffoldGenerationPlan, ScaffoldError> {
    let prepared = prepare_generation(request)?;
    if !request.dry_run {
        materialize_generation(&prepared)?;
    }
    Ok(prepared.plan)
}

fn prepare_generation(
    request: &ScaffoldGenerationRequest<'_>,
) -> Result<PreparedGeneration, ScaffoldError> {
    enforce_generation_policy(request)?;
    let generator = request
        .package
        .manifest
        .generators
        .iter()
        .find(|generator| generator.id == request.generator_id)
        .ok_or_else(|| {
            generation_error(format!(
                "generator_not_found: package `{}` has no generator `{}`",
                request.package.manifest.package.id, request.generator_id
            ))
        })?;
    let (instruction_path, expected_instruction_sha) = match &generator.driver {
        GeneratorDriver::Builtin { target } => {
            return Err(generation_error(format!(
                "builtin_generator_delegation: use `oclive {target}`; scaffold generate does not proxy official domain generators"
            )));
        }
        GeneratorDriver::Instruction { path, sha256 } => {
            let sha256 = sha256.as_deref().ok_or_else(|| {
                generation_error(format!(
                    "instruction_digest_missing: generator `{}` is discovery-compatible only; add instruction.sha256 and require scaffold_contract >=1.1,<2",
                    generator.id
                ))
            })?;
            (path.as_str(), sha256)
        }
    };

    let package_root =
        request
            .package_root
            .canonicalize()
            .map_err(|source| ScaffoldError::Read {
                path: request.package_root.to_path_buf(),
                source,
            })?;
    if !package_root.is_dir() {
        return Err(generation_error(format!(
            "package_root_not_directory: {}",
            package_root.display()
        )));
    }
    let instruction_file =
        resolve_package_file(&package_root, instruction_path, "instruction_path_escape")?;
    let instruction_bytes = fs::read(&instruction_file).map_err(|source| ScaffoldError::Read {
        path: instruction_file.clone(),
        source,
    })?;
    let instruction_sha256 = scaffold_sha256_hex(&instruction_bytes);
    if instruction_sha256 != expected_instruction_sha {
        return Err(generation_error(format!(
            "instruction_digest_mismatch: `{instruction_path}` expected {expected_instruction_sha} but found {instruction_sha256}; update the package intentionally and rewrite the scaffold lock"
        )));
    }
    let instruction = serde_json::from_slice::<ScaffoldInstructionDocument>(&instruction_bytes)
        .map_err(|source| ScaffoldError::Parse {
            path: instruction_file,
            source,
        })?;
    validate_instruction(&instruction)?;
    let variables = resolve_variables(
        &instruction,
        &request.package.manifest.defaults,
        request.variables,
    )?;
    let output = resolve_new_output(request.output)?;

    let mut records = Vec::with_capacity(instruction.files.len());
    let mut prepared_files = Vec::with_capacity(instruction.files.len());
    let mut targets = BTreeSet::from([PROVENANCE_RELATIVE_PATH.to_string()]);
    for declaration in &instruction.files {
        let target = normalize_target_path(&declaration.target)?;
        let target_string = normalize_path(&target);
        reject_target_conflict(&targets, &target_string)?;
        targets.insert(target_string.clone());

        let source_path =
            resolve_package_file(&package_root, &declaration.source, "source_path_escape")?;
        let source_bytes = fs::read(&source_path).map_err(|source| ScaffoldError::Read {
            path: source_path,
            source,
        })?;
        let source_sha256 = scaffold_sha256_hex(&source_bytes);
        if source_sha256 != declaration.sha256 {
            return Err(generation_error(format!(
                "source_digest_mismatch: `{}` expected {} but found {}; update the instruction digest intentionally",
                declaration.source, declaration.sha256, source_sha256
            )));
        }
        let output_bytes = match declaration.mode {
            ScaffoldInstructionFileMode::Copy => source_bytes,
            ScaffoldInstructionFileMode::Text => {
                let template = std::str::from_utf8(&source_bytes).map_err(|error_value| {
                    generation_error(format!(
                        "template_not_utf8: `{}` cannot use text mode: {error_value}",
                        declaration.source
                    ))
                })?;
                render_template(template, &variables)?.into_bytes()
            }
        };
        let bytes = u64::try_from(output_bytes.len()).map_err(|error_value| {
            generation_error(format!(
                "generated_file_too_large: `{target_string}` length cannot be recorded: {error_value}"
            ))
        })?;
        records.push(ScaffoldGeneratedFile {
            path: target_string,
            mode: declaration.mode,
            source_sha256,
            output_sha256: scaffold_sha256_hex(&output_bytes),
            bytes,
        });
        prepared_files.push(PreparedFile {
            target,
            bytes: output_bytes,
        });
    }

    let mut variable_names = variables.keys().cloned().collect::<Vec<_>>();
    variable_names.sort();
    records.sort_by(|a, b| a.path.cmp(&b.path));
    prepared_files.sort_by(|a, b| a.target.cmp(&b.target));
    let provenance = ScaffoldGenerationProvenance {
        schema_version: SCAFFOLD_PROVENANCE_SCHEMA_VERSION,
        package: ScaffoldGenerationPackage {
            id: request.package.manifest.package.id.clone(),
            version: request.package.manifest.package.version.clone(),
            source: request.package.source,
            locator: request.package.locator.clone(),
            trust: request.package.trust,
            maintainer: request.package.manifest.package.maintainer.clone(),
            manifest_sha256: request.package.manifest_sha256.clone(),
        },
        generator_id: request.generator_id.to_string(),
        instruction_path: instruction_path.to_string(),
        instruction_sha256,
        variable_names,
        files: records,
    };
    Ok(PreparedGeneration {
        output: output.clone(),
        files: prepared_files,
        plan: ScaffoldGenerationPlan {
            output: normalize_path(&output),
            dry_run: request.dry_run,
            provenance,
        },
    })
}

fn enforce_generation_policy(request: &ScaffoldGenerationRequest<'_>) -> Result<(), ScaffoldError> {
    if !request
        .package
        .manifest
        .permissions
        .iter()
        .any(|permission| permission == "project.write")
    {
        return Err(generation_error(
            "project_write_not_declared: the selected package must declare project.write",
        ));
    }
    if request.package.trust == ScaffoldTrust::UntrustedLocal {
        if !request.accept_untrusted {
            return Err(generation_error(format!(
                "untrusted_confirmation_required: `{}` is maintained by `{}`; pass --accept-untrusted to authorize only this bounded generation",
                request.package.manifest.package.id,
                request.package.manifest.package.maintainer
            )));
        }
        let lock = request.lock.ok_or_else(|| {
            generation_error(
                "scaffold_lock_required: run `oclive scaffold resolve --write-lock -o <project>` before generating from a local package",
            )
        })?;
        verify_lock(lock, request.package)?;
    }
    Ok(())
}

fn verify_lock(lock: &ScaffoldLock, package: &ResolvedPackage) -> Result<(), ScaffoldError> {
    if lock.schema_version != SCAFFOLD_LOCK_SCHEMA_VERSION
        || lock.scaffold_contract != SCAFFOLD_CONTRACT_VERSION
    {
        return Err(generation_error(format!(
            "scaffold_lock_stale: lock schema/contract is {}/{} but generation requires {}/{}; rerun `oclive scaffold resolve --write-lock -o <project>`",
            lock.schema_version,
            lock.scaffold_contract,
            SCAFFOLD_LOCK_SCHEMA_VERSION,
            SCAFFOLD_CONTRACT_VERSION
        )));
    }
    let Some(entry) = lock
        .packages
        .iter()
        .find(|entry| entry.id == package.manifest.package.id)
    else {
        return Err(generation_error(format!(
            "scaffold_lock_missing_package: `{}` is not pinned; rerun `oclive scaffold resolve --write-lock -o <project>`",
            package.manifest.package.id
        )));
    };
    let matches = entry.version == package.manifest.package.version
        && entry.source == package.source
        && entry.locator == package.locator
        && entry.manifest_sha256 == package.manifest_sha256;
    if !matches {
        return Err(generation_error(format!(
            "scaffold_lock_mismatch: `{}` changed source, version, locator, or manifest digest; inspect it and rerun `oclive scaffold resolve --write-lock -o <project>`",
            package.manifest.package.id
        )));
    }
    Ok(())
}

fn validate_instruction(document: &ScaffoldInstructionDocument) -> Result<(), ScaffoldError> {
    let mut issues = Vec::new();
    if document.schema_version != SCAFFOLD_INSTRUCTION_SCHEMA_VERSION {
        issues.push(format!(
            "unsupported_instruction_schema: expected {}, found {}",
            SCAFFOLD_INSTRUCTION_SCHEMA_VERSION, document.schema_version
        ));
    }
    if document.files.is_empty() {
        issues.push("instruction_files_empty: files must contain at least one mapping".to_string());
    }
    for (name, variable) in &document.variables {
        if !valid_component_id(name) {
            issues.push(format!(
                "invalid_instruction_variable: `{name}` must use a lowercase component ID"
            ));
        }
        if variable.description.trim().is_empty() {
            issues.push(format!(
                "empty_instruction_variable_description: `{name}` needs a description"
            ));
        }
    }
    for file in &document.files {
        if normalize_source_path(&file.source).is_err() {
            issues.push(format!(
                "unsafe_instruction_source: `{}` must stay inside the package",
                file.source
            ));
        }
        if normalize_target_path(&file.target).is_err() {
            issues.push(format!(
                "unsafe_instruction_target: `{}` must be a normalized relative path",
                file.target
            ));
        }
        if !valid_sha256(&file.sha256) {
            issues.push(format!(
                "invalid_source_digest: `{}` sha256 must be 64 lowercase hexadecimal characters",
                file.source
            ));
        }
    }
    if issues.is_empty() {
        Ok(())
    } else {
        issues.sort();
        issues.dedup();
        Err(generation_error(issues.join("; ")))
    }
}

fn resolve_variables(
    instruction: &ScaffoldInstructionDocument,
    manifest_defaults: &BTreeMap<String, Value>,
    overrides: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, ScaffoldError> {
    for name in overrides.keys() {
        if !instruction.variables.contains_key(name) {
            return Err(generation_error(format!(
                "unknown_generation_variable: `{name}` is not declared by the instruction"
            )));
        }
    }
    let mut resolved = BTreeMap::new();
    for (name, declaration) in &instruction.variables {
        let value = if let Some(value) = overrides.get(name) {
            value.clone()
        } else if let Some(value) = manifest_defaults.get(name) {
            value.as_str().map(ToString::to_string).ok_or_else(|| {
                generation_error(format!(
                    "non_string_generation_default: manifest default `{name}` must be a string"
                ))
            })?
        } else if let Some(value) = &declaration.default {
            value.clone()
        } else if declaration.required {
            return Err(generation_error(format!(
                "missing_generation_variable: provide `--set {name}=<value>`"
            )));
        } else {
            String::new()
        };
        resolved.insert(name.clone(), value);
    }
    Ok(resolved)
}

fn render_template(
    template: &str,
    variables: &BTreeMap<String, String>,
) -> Result<String, ScaffoldError> {
    let mut output = String::with_capacity(template.len());
    let mut remaining = template;
    loop {
        let Some(start) = remaining.find("{{") else {
            if remaining.contains("}}") {
                return Err(generation_error(
                    "malformed_template_placeholder: found closing braces without an opening placeholder",
                ));
            }
            output.push_str(remaining);
            return Ok(output);
        };
        let (prefix, after_open) = remaining.split_at(start);
        if prefix.contains("}}") {
            return Err(generation_error(
                "malformed_template_placeholder: found closing braces before an opening placeholder",
            ));
        }
        output.push_str(prefix);
        let after_open = &after_open[2..];
        let Some(end) = after_open.find("}}") else {
            return Err(generation_error(
                "malformed_template_placeholder: unclosed `{{variable}}` placeholder",
            ));
        };
        let name = &after_open[..end];
        if !valid_component_id(name) {
            return Err(generation_error(format!(
                "invalid_template_placeholder: `{{{{{name}}}}}` is not a declared component ID"
            )));
        }
        let value = variables.get(name).ok_or_else(|| {
            generation_error(format!(
                "unknown_template_placeholder: `{{{{{name}}}}}` is not declared"
            ))
        })?;
        output.push_str(value);
        remaining = &after_open[end + 2..];
    }
}

fn resolve_package_file(
    canonical_package_root: &Path,
    relative: &str,
    escape_code: &str,
) -> Result<PathBuf, ScaffoldError> {
    let relative = normalize_source_path(relative)?;
    let candidate = canonical_package_root.join(&relative);
    let canonical = candidate
        .canonicalize()
        .map_err(|source| ScaffoldError::Read {
            path: candidate,
            source,
        })?;
    if !canonical.is_file() || !canonical.starts_with(canonical_package_root) {
        return Err(generation_error(format!(
            "{escape_code}: `{}` does not resolve to a file inside {}",
            relative.display(),
            canonical_package_root.display()
        )));
    }
    Ok(canonical)
}

fn normalize_source_path(value: &str) -> Result<PathBuf, ScaffoldError> {
    let path = Path::new(value);
    if value.trim().is_empty() || path.is_absolute() {
        return Err(generation_error(format!(
            "unsafe_relative_path: `{value}` must stay inside the package"
        )));
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(generation_error(format!(
                    "unsafe_relative_path: `{value}` must stay inside the package"
                )));
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(generation_error(format!(
            "unsafe_relative_path: `{value}` must name a file"
        )));
    }
    Ok(normalized)
}

fn normalize_target_path(value: &str) -> Result<PathBuf, ScaffoldError> {
    let path = Path::new(value);
    if value.trim().is_empty() || path.is_absolute() {
        return Err(generation_error(format!(
            "unsafe_generation_target: `{value}` must be a relative file path"
        )));
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => {
                return Err(generation_error(format!(
                    "unsafe_generation_target: `{value}` must not contain dot, parent, root, or prefix components"
                )));
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(generation_error(format!(
            "unsafe_generation_target: `{value}` must name a file"
        )));
    }
    Ok(normalized)
}

fn reject_target_conflict(
    existing: &BTreeSet<String>,
    candidate: &str,
) -> Result<(), ScaffoldError> {
    for target in existing {
        if target == candidate
            || target
                .strip_prefix(candidate)
                .is_some_and(|suffix| suffix.starts_with('/'))
            || candidate
                .strip_prefix(target)
                .is_some_and(|suffix| suffix.starts_with('/'))
        {
            return Err(generation_error(format!(
                "generation_target_conflict: `{candidate}` conflicts with `{target}`"
            )));
        }
    }
    Ok(())
}

fn resolve_new_output(output: &Path) -> Result<PathBuf, ScaffoldError> {
    let file_name = output.file_name().ok_or_else(|| {
        generation_error(format!(
            "invalid_generation_output: `{}` must name a new directory",
            output.display()
        ))
    })?;
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let canonical_parent = parent
        .canonicalize()
        .map_err(|source| ScaffoldError::Read {
            path: parent.to_path_buf(),
            source,
        })?;
    if !canonical_parent.is_dir() {
        return Err(generation_error(format!(
            "generation_parent_not_directory: {}",
            canonical_parent.display()
        )));
    }
    let resolved = canonical_parent.join(file_name);
    if resolved
        .try_exists()
        .map_err(|source| ScaffoldError::Read {
            path: resolved.clone(),
            source,
        })?
    {
        return Err(generation_error(format!(
            "generation_output_exists: {}",
            resolved.display()
        )));
    }
    Ok(resolved)
}

fn materialize_generation(prepared: &PreparedGeneration) -> Result<(), ScaffoldError> {
    let parent = prepared.output.parent().ok_or_else(|| {
        generation_error(format!(
            "invalid_generation_output: {}",
            prepared.output.display()
        ))
    })?;
    let temporary = tempfile::Builder::new()
        .prefix(".oclive-scaffold-")
        .tempdir_in(parent)
        .map_err(|source| ScaffoldError::WriteGeneration {
            path: parent.to_path_buf(),
            source,
        })?;
    let stage_root = temporary.path().join("output");
    fs::create_dir(&stage_root).map_err(|source| ScaffoldError::WriteGeneration {
        path: stage_root.clone(),
        source,
    })?;
    for file in &prepared.files {
        write_file(&stage_root.join(&file.target), &file.bytes)?;
    }
    let provenance_path = stage_root
        .join(".oclive")
        .join(SCAFFOLD_PROVENANCE_FILENAME);
    let mut provenance = serde_json::to_vec_pretty(&prepared.plan.provenance)
        .map_err(|error_value| generation_error(format!("serialize_provenance: {error_value}")))?;
    provenance.push(b'\n');
    write_file(&provenance_path, &provenance)?;

    if prepared
        .output
        .try_exists()
        .map_err(|source| ScaffoldError::WriteGeneration {
            path: prepared.output.clone(),
            source,
        })?
    {
        return Err(generation_error(format!(
            "generation_output_exists: {}",
            prepared.output.display()
        )));
    }
    fs::rename(&stage_root, &prepared.output).map_err(|source| ScaffoldError::WriteGeneration {
        path: prepared.output.clone(),
        source,
    })?;
    Ok(())
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), ScaffoldError> {
    let parent = path.parent().ok_or_else(|| {
        generation_error(format!("generated_file_has_no_parent: {}", path.display()))
    })?;
    fs::create_dir_all(parent).map_err(|source| ScaffoldError::WriteGeneration {
        path: parent.to_path_buf(),
        source,
    })?;
    let mut file = fs::File::create(path).map_err(|source| ScaffoldError::WriteGeneration {
        path: path.to_path_buf(),
        source,
    })?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|source| ScaffoldError::WriteGeneration {
            path: path.to_path_buf(),
            source,
        })
}

/// Return the lowercase SHA-256 representation used by scaffold integrity fields.
#[must_use]
pub fn scaffold_sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn generation_error(issues: impl Into<String>) -> ScaffoldError {
    ScaffoldError::Generation {
        issues: issues.into(),
    }
}
