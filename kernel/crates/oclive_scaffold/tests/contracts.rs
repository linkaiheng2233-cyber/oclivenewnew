use std::{collections::BTreeMap, fs, path::Path};

use oclive_scaffold::{
    build_scaffold_lock, merge_scaffold_configs, project_scaffold_lock_path,
    resolve_scaffold_catalog, scan_scaffold_catalog, validate_manifest, write_scaffold_lock_atomic,
    CompositionDeclaration, GeneratorDeclaration, GeneratorDriver, ScaffoldCompatibility,
    ScaffoldConfig, ScaffoldManifest, ScaffoldPackageIdentity, ScaffoldSource, ScaffoldTrust,
};
use semver::Version;
use tempfile::TempDir;

fn reader_version() -> Version {
    Version::new(0, 1, 0)
}

fn custom_manifest(id: &str, version: &str) -> ScaffoldManifest {
    ScaffoldManifest {
        schema_version: 1,
        package: ScaffoldPackageIdentity {
            id: id.to_string(),
            version: version.to_string(),
            display_name: id.to_string(),
            description: "test package".to_string(),
            maintainer: "test maintainer".to_string(),
        },
        compatibility: ScaffoldCompatibility {
            oclive_cli: ">=0.1.0, <1.0.0".to_string(),
            scaffold_contract: "^1.0".to_string(),
        },
        command_namespace: id.to_string(),
        generators: Vec::new(),
        commands: Vec::new(),
        permissions: Vec::new(),
        defaults: BTreeMap::new(),
        dependencies: Vec::new(),
        extends: Vec::new(),
        composition: CompositionDeclaration::default(),
        extensions: BTreeMap::new(),
    }
}

fn write_manifest(root: &Path, directory: &str, manifest: &ScaffoldManifest) {
    let package = root.join(directory);
    fs::create_dir_all(&package).expect("create package directory");
    fs::write(
        package.join("oclive.scaffold.json"),
        serde_json::to_vec_pretty(manifest).expect("serialize manifest"),
    )
    .expect("write manifest");
}

#[test]
fn compiled_official_fallback_resolves_without_local_sources() {
    let temp = TempDir::new().expect("tempdir");
    let scan = scan_scaffold_catalog(
        &temp.path().join("project-missing"),
        &temp.path().join("user-missing"),
        &reader_version(),
    );
    assert!(scan.issues.is_empty(), "{:?}", scan.issues);
    assert_eq!(scan.candidates.len(), 4);
    assert!(scan
        .candidates
        .iter()
        .all(|candidate| candidate.source == ScaffoldSource::Official
            && candidate.trust == ScaffoldTrust::Official));

    let resolved = resolve_scaffold_catalog(&scan, &ScaffoldConfig::default(), &reader_version())
        .expect("official resolution");
    assert_eq!(resolved.packages.len(), 4);
    assert_eq!(
        resolved.source_order,
        vec![
            ScaffoldSource::Project,
            ScaffoldSource::User,
            ScaffoldSource::Official
        ]
    );
}

#[test]
fn project_source_shadows_user_source_deterministically() {
    let temp = TempDir::new().expect("tempdir");
    let project = temp.path().join("project");
    let user = temp.path().join("user");
    write_manifest(
        &project,
        "same-locator",
        &custom_manifest("dev.example.character", "2.0.0"),
    );
    write_manifest(
        &user,
        "same-locator",
        &custom_manifest("dev.example.character", "1.0.0"),
    );
    let scan = scan_scaffold_catalog(&project, &user, &reader_version());
    let resolved = resolve_scaffold_catalog(&scan, &ScaffoldConfig::default(), &reader_version())
        .expect("resolution");
    let selected = resolved
        .packages
        .iter()
        .find(|package| package.manifest.package.id == "dev.example.character")
        .expect("selected custom package");
    assert_eq!(selected.source, ScaffoldSource::Project);
    assert_eq!(selected.manifest.package.version, "2.0.0");
    assert_eq!(resolved.shadowed.len(), 1);
    assert_eq!(resolved.shadowed[0].source, ScaffoldSource::User);
}

#[test]
fn project_config_overrides_user_config_without_masking_other_keys() {
    let mut user = ScaffoldConfig {
        source_order: Some(vec![
            ScaffoldSource::User,
            ScaffoldSource::Project,
            ScaffoldSource::Official,
        ]),
        ..ScaffoldConfig::default()
    };
    user.package_enabled
        .insert("dev.example.first".to_string(), false);
    user.package_sources
        .insert("dev.example.second".to_string(), ScaffoldSource::User);

    let mut project = ScaffoldConfig::default();
    project
        .package_enabled
        .insert("dev.example.first".to_string(), true);
    let merged = merge_scaffold_configs(Some(&user), Some(&project)).expect("valid merge");
    assert_eq!(merged.source_order, user.source_order);
    assert!(merged.package_enabled["dev.example.first"]);
    assert_eq!(
        merged.package_sources["dev.example.second"],
        ScaffoldSource::User
    );
}

#[test]
fn third_party_reserved_namespace_and_ci_permission_are_hard_errors() {
    let mut manifest = custom_manifest("com.oclive.scaffold.evil", "1.0.0");
    manifest.permissions.push("ci.runner".to_string());
    let validation = validate_manifest(&manifest, ScaffoldSource::Project, &reader_version());
    let codes = validation
        .errors
        .iter()
        .map(|issue| issue.code.as_str())
        .collect::<Vec<_>>();
    assert!(codes.contains(&"reserved_package_namespace"));
    assert!(codes.contains(&"reserved_command_namespace"));
    assert!(codes.contains(&"forbidden_ci_capability"));
}

#[test]
fn unsupported_schema_is_rejected_with_migration_evidence() {
    let mut manifest = custom_manifest("dev.example.future", "1.0.0");
    manifest.schema_version = 2;
    let validation = validate_manifest(&manifest, ScaffoldSource::Project, &reader_version());
    assert!(validation.errors.iter().any(|issue| {
        issue.code == "unsupported_schema_version" && issue.message.contains("migrate")
    }));
}

#[test]
fn instruction_digest_is_optional_for_discovery_but_strict_when_present() {
    let mut legacy = custom_manifest("dev.example.legacy", "1.0.0");
    legacy.generators.push(GeneratorDeclaration {
        id: "project".to_string(),
        kind: "project".to_string(),
        driver: GeneratorDriver::Instruction {
            path: "instructions.json".to_string(),
            sha256: None,
        },
    });
    let legacy_validation = validate_manifest(&legacy, ScaffoldSource::Project, &reader_version());
    assert!(legacy_validation.is_valid());
    assert!(legacy_validation.warnings.iter().any(|issue| {
        issue.code == "instruction_digest_required_for_generation"
            && issue.message.contains(">=1.1,<2")
    }));

    let GeneratorDriver::Instruction { sha256, .. } = &mut legacy.generators[0].driver else {
        panic!("fixture instruction driver")
    };
    *sha256 = Some("ABC".to_string());
    let invalid = validate_manifest(&legacy, ScaffoldSource::Project, &reader_version());
    assert!(invalid
        .errors
        .iter()
        .any(|issue| issue.code == "invalid_instruction_digest"));
}

#[test]
fn duplicate_id_within_one_source_fails_closed() {
    let temp = TempDir::new().expect("tempdir");
    let project = temp.path().join("project");
    write_manifest(
        &project,
        "first",
        &custom_manifest("dev.example.duplicate", "1.0.0"),
    );
    write_manifest(
        &project,
        "second",
        &custom_manifest("dev.example.duplicate", "2.0.0"),
    );
    let scan = scan_scaffold_catalog(&project, &temp.path().join("missing"), &reader_version());
    let error_value =
        resolve_scaffold_catalog(&scan, &ScaffoldConfig::default(), &reader_version())
            .expect_err("duplicate source package must fail");
    assert!(error_value.to_string().contains("duplicate package id"));
}

#[test]
fn missing_forced_source_fails_instead_of_falling_back() {
    let temp = TempDir::new().expect("tempdir");
    let scan = scan_scaffold_catalog(
        &temp.path().join("project-missing"),
        &temp.path().join("user-missing"),
        &reader_version(),
    );
    let mut config = ScaffoldConfig::default();
    config
        .package_sources
        .insert("dev.example.missing".to_string(), ScaffoldSource::Project);
    let error_value = resolve_scaffold_catalog(&scan, &config, &reader_version())
        .expect_err("forced source must exist");
    assert!(error_value.to_string().contains("source is unavailable"));
}

#[test]
fn lock_is_deterministic_and_written_atomically() {
    let temp = TempDir::new().expect("tempdir");
    let scan = scan_scaffold_catalog(
        &temp.path().join("project-missing"),
        &temp.path().join("user-missing"),
        &reader_version(),
    );
    let resolved = resolve_scaffold_catalog(&scan, &ScaffoldConfig::default(), &reader_version())
        .expect("official resolution");
    let first = build_scaffold_lock(&resolved);
    let second = build_scaffold_lock(&resolved);
    assert_eq!(first, second);
    assert!(first
        .packages
        .windows(2)
        .all(|window| window[0].id < window[1].id));

    let path = project_scaffold_lock_path(temp.path());
    write_scaffold_lock_atomic(&path, &first).expect("write lock");
    let persisted = fs::read_to_string(&path).expect("read lock");
    let parsed = serde_json::from_str(&persisted).expect("parse lock");
    assert_eq!(first, parsed);
}

#[test]
fn published_json_schemas_are_well_formed_json() {
    for document in [
        include_str!("../schemas/oclive.scaffold.schema.json"),
        include_str!("../schemas/scaffold.config.schema.json"),
        include_str!("../schemas/scaffold.lock.schema.json"),
        include_str!("../schemas/scaffold.instructions.schema.json"),
        include_str!("../schemas/scaffold.provenance.schema.json"),
    ] {
        let parsed =
            serde_json::from_str::<serde_json::Value>(document).expect("valid schema JSON");
        assert_eq!(
            parsed.get("$schema").and_then(serde_json::Value::as_str),
            Some("https://json-schema.org/draft/2020-12/schema")
        );
    }
}
