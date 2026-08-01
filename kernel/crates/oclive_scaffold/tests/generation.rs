#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::{collections::BTreeMap, fs, path::Path};

use oclive_scaffold::{
    build_scaffold_lock, generate_scaffold, resolve_scaffold_catalog, scan_scaffold_catalog,
    ScaffoldConfig, ScaffoldGenerationRequest,
};
use semver::Version;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

struct Fixture {
    _temp: tempfile::TempDir,
    project: std::path::PathBuf,
    package_root: std::path::PathBuf,
    resolved: oclive_scaffold::ResolvedCatalog,
    lock: oclive_scaffold::ScaffoldLock,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("project");
    let package_root = project.join(".oclive/scaffolds/example");
    fs::create_dir_all(package_root.join("templates")).expect("package directories");
    let readme = b"# {{project_name}}\n{{tagline}}\n";
    let asset = [0_u8, 1, 2, 255];
    fs::write(package_root.join("templates/README.md.tmpl"), readme).expect("template");
    fs::write(package_root.join("templates/logo.bin"), asset).expect("asset");
    let instruction = json!({
        "schema_version": 1,
        "variables": {
            "project_name": {
                "description": "Generated project name",
                "required": true
            },
            "tagline": {
                "description": "Generated tagline",
                "default": "instruction default"
            }
        },
        "files": [
            {
                "source": "templates/README.md.tmpl",
                "target": "README.md",
                "mode": "text",
                "sha256": sha256(readme)
            },
            {
                "source": "templates/logo.bin",
                "target": "assets/logo.bin",
                "mode": "copy",
                "sha256": sha256(&asset)
            }
        ]
    });
    let instruction_bytes = serde_json::to_vec_pretty(&instruction).expect("instruction JSON");
    fs::write(package_root.join("instructions.json"), &instruction_bytes).expect("instruction");
    let manifest = json!({
        "schema_version": 1,
        "package": {
            "id": "dev.example.scaffold",
            "version": "1.2.3",
            "display_name": "Example",
            "description": "Generation fixture",
            "maintainer": "fixture maintainer"
        },
        "compatibility": {
            "oclive_cli": ">=0.1.0, <1.0.0",
            "scaffold_contract": ">=1.1, <2"
        },
        "command_namespace": "dev.example.tools",
        "generators": [{
            "id": "project",
            "kind": "project",
            "driver": {
                "kind": "instruction",
                "path": "instructions.json",
                "sha256": sha256(&instruction_bytes)
            }
        }],
        "commands": [],
        "permissions": ["project.write"],
        "defaults": { "tagline": "manifest default" },
        "dependencies": [],
        "extends": [],
        "composition": {},
        "extensions": {}
    });
    fs::write(
        package_root.join("oclive.scaffold.json"),
        serde_json::to_vec_pretty(&manifest).expect("manifest JSON"),
    )
    .expect("manifest");
    let version = Version::new(0, 1, 0);
    let scan = scan_scaffold_catalog(
        &project.join(".oclive/scaffolds"),
        &temp.path().join("user-missing"),
        &version,
    );
    let resolved = resolve_scaffold_catalog(&scan, &ScaffoldConfig::default(), &version)
        .expect("resolved catalog");
    let lock = build_scaffold_lock(&resolved);
    Fixture {
        _temp: temp,
        project,
        package_root,
        resolved,
        lock,
    }
}

fn selected(fixture: &Fixture) -> &oclive_scaffold::ResolvedPackage {
    fixture
        .resolved
        .packages
        .iter()
        .find(|package| package.manifest.package.id == "dev.example.scaffold")
        .expect("selected fixture")
}

fn request<'a>(
    fixture: &'a Fixture,
    output: &'a Path,
    variables: &'a BTreeMap<String, String>,
) -> ScaffoldGenerationRequest<'a> {
    ScaffoldGenerationRequest {
        package: selected(fixture),
        package_root: &fixture.package_root,
        generator_id: "project",
        output,
        variables,
        lock: Some(&fixture.lock),
        accept_untrusted: true,
        dry_run: false,
    }
}

#[test]
fn materializes_text_copy_and_value_free_provenance() {
    let fixture = fixture();
    let output = fixture.project.join("generated");
    let variables = BTreeMap::from([("project_name".to_string(), "private-name".to_string())]);
    let plan = generate_scaffold(&request(&fixture, &output, &variables)).expect("generation");

    assert_eq!(
        fs::read_to_string(output.join("README.md")).expect("README"),
        "# private-name\nmanifest default\n"
    );
    assert_eq!(
        fs::read(output.join("assets/logo.bin")).expect("copied asset"),
        [0_u8, 1, 2, 255]
    );
    let provenance_bytes =
        fs::read(output.join(".oclive/scaffold.provenance.json")).expect("provenance");
    let provenance_text = String::from_utf8(provenance_bytes.clone()).expect("utf8 provenance");
    assert!(!provenance_text.contains("private-name"));
    assert!(!provenance_text.contains("manifest default"));
    let provenance: Value = serde_json::from_slice(&provenance_bytes).expect("provenance JSON");
    assert_eq!(provenance["generator_id"], "project");
    assert_eq!(plan.provenance.files.len(), 2);
    assert_eq!(plan.provenance.variable_names, ["project_name", "tagline"]);
}

#[test]
fn dry_run_performs_rendering_without_writing() {
    let fixture = fixture();
    let output = fixture.project.join("dry-run");
    let variables = BTreeMap::from([("project_name".to_string(), "preview".to_string())]);
    let mut generation = request(&fixture, &output, &variables);
    generation.dry_run = true;
    let plan = generate_scaffold(&generation).expect("dry run");
    assert!(plan.dry_run);
    assert!(!output.exists());
    assert_eq!(plan.provenance.files.len(), 2);
}

#[test]
fn untrusted_generation_requires_confirmation_and_current_lock() {
    let fixture = fixture();
    let output = fixture.project.join("refused");
    let variables = BTreeMap::from([("project_name".to_string(), "preview".to_string())]);

    let mut generation = request(&fixture, &output, &variables);
    generation.accept_untrusted = false;
    let confirmation = generate_scaffold(&generation).expect_err("confirmation required");
    assert!(confirmation
        .to_string()
        .contains("untrusted_confirmation_required"));

    generation.accept_untrusted = true;
    generation.lock = None;
    let missing_lock = generate_scaffold(&generation).expect_err("lock required");
    assert!(missing_lock.to_string().contains("scaffold_lock_required"));

    let mut stale_lock = fixture.lock.clone();
    stale_lock
        .packages
        .iter_mut()
        .find(|entry| entry.id == "dev.example.scaffold")
        .expect("fixture lock entry")
        .manifest_sha256 = "0".repeat(64);
    generation.lock = Some(&stale_lock);
    let stale = generate_scaffold(&generation).expect_err("stale lock rejected");
    assert!(stale.to_string().contains("scaffold_lock_mismatch"));
    assert!(!output.exists());
}

#[test]
fn integrity_failure_leaves_no_partial_output() {
    let fixture = fixture();
    let output = fixture.project.join("digest-failure");
    let variables = BTreeMap::from([("project_name".to_string(), "preview".to_string())]);
    fs::write(
        fixture.package_root.join("templates/README.md.tmpl"),
        "changed after pinning",
    )
    .expect("change template");
    let error_value =
        generate_scaffold(&request(&fixture, &output, &variables)).expect_err("digest mismatch");
    assert!(error_value.to_string().contains("source_digest_mismatch"));
    assert!(!output.exists());
}

#[test]
fn unknown_and_missing_variables_fail_before_writes() {
    let fixture = fixture();
    let output = fixture.project.join("variable-failure");
    let unknown = BTreeMap::from([("unknown".to_string(), "value".to_string())]);
    let unknown_error =
        generate_scaffold(&request(&fixture, &output, &unknown)).expect_err("unknown variable");
    assert!(unknown_error
        .to_string()
        .contains("unknown_generation_variable"));

    let missing = BTreeMap::new();
    let missing_error =
        generate_scaffold(&request(&fixture, &output, &missing)).expect_err("missing variable");
    assert!(missing_error
        .to_string()
        .contains("missing_generation_variable"));
    assert!(!output.exists());
}

#[test]
fn existing_output_is_never_overwritten() {
    let fixture = fixture();
    let output = fixture.project.join("existing");
    fs::create_dir(&output).expect("existing output");
    fs::write(output.join("owned.txt"), "keep me").expect("existing content");
    let variables = BTreeMap::from([("project_name".to_string(), "preview".to_string())]);

    let error_value =
        generate_scaffold(&request(&fixture, &output, &variables)).expect_err("existing output");
    assert!(error_value.to_string().contains("generation_output_exists"));
    assert_eq!(
        fs::read_to_string(output.join("owned.txt")).expect("preserved content"),
        "keep me"
    );
    assert!(!output.join("README.md").exists());
}
