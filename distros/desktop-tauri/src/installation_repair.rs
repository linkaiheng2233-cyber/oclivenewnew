//! Filesystem-only Chat Pro installation diagnostics and conservative repair.
//!
//! This module deliberately does not touch chats, memories, models, role packs,
//! user-installed plugins, or application settings. It is shared by the Tauri
//! settings command and the pre-UI `--repair-installation` recovery mode.

use oclive_kernel_host::infrastructure::directory_plugins::{
    plugin_scan_container_roots, HostPluginsFile, OclivePluginManifest,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepairSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepairIssueScope {
    Installation,
    Resources,
    Storage,
    Roles,
    Plugins,
    Kernel,
    ModelService,
    Voice,
    Network,
    Reporting,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepairIssueCategory {
    Missing,
    Access,
    Invalid,
    Conflict,
    Compatibility,
    Unreachable,
    Cleanup,
    Reporting,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallationRepairIssue {
    pub code: String,
    pub scope: RepairIssueScope,
    pub category: RepairIssueCategory,
    pub severity: RepairSeverity,
    pub summary: String,
    pub detail: String,
    pub path: String,
    pub repairable: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepairActionStatus {
    Repaired,
    Unchanged,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallationRepairAction {
    pub code: String,
    pub status: RepairActionStatus,
    pub summary: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallationRepairReport {
    pub app_version: String,
    pub operating_system: String,
    pub architecture: String,
    pub generated_at_epoch_ms: u128,
    pub success: bool,
    pub changed: bool,
    pub restart_required: bool,
    pub resource_dir: String,
    pub roles_dir: String,
    pub app_data_dir: String,
    pub role_count: usize,
    pub plugin_count: usize,
    pub plugin_ids: Vec<String>,
    pub actions: Vec<InstallationRepairAction>,
    pub issues: Vec<InstallationRepairIssue>,
    pub report_path: String,
}

impl InstallationRepairReport {
    pub fn add_action(
        &mut self,
        code: &str,
        status: RepairActionStatus,
        summary: &str,
        detail: String,
    ) {
        self.actions.push(InstallationRepairAction {
            code: code.to_string(),
            status,
            summary: summary.to_string(),
            detail,
        });
        if status == RepairActionStatus::Repaired {
            self.changed = true;
        }
    }

    pub fn refresh_success(&mut self) {
        self.success = !self
            .issues
            .iter()
            .any(|issue| issue.severity == RepairSeverity::Error);
    }
}

fn display(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[must_use]
pub fn classify_repair_issue(code: &str) -> (RepairIssueScope, RepairIssueCategory) {
    use RepairIssueCategory::{
        Access, Cleanup, Conflict, Invalid, Missing, Reporting, Unknown, Unreachable,
    };
    use RepairIssueScope::{
        Installation, Kernel, ModelService, Network, Plugins, Reporting as ReportingScope,
        Resources, Roles, Storage, Unknown as UnknownScope, Voice,
    };

    match code {
        "RESOURCE_DIRECTORY_MISSING" => (Installation, Missing),
        "RESOURCE_DIRECTORY_UNREADABLE" => (Installation, Access),
        "BUNDLED_PROFILE_MISSING" => (Resources, Missing),
        "APP_DATA_CREATE_FAILED" | "APP_DATA_NOT_WRITABLE" => (Storage, Access),
        "REPAIR_PROBE_CLEANUP_FAILED" => (Storage, Cleanup),
        "REPAIR_REPORT_WRITE_FAILED" => (ReportingScope, Reporting),
        "BUNDLED_ROLES_MISSING" | "ROLES_DIRECTORY_MISSING" | "ROLE_PACKS_NOT_FOUND" => {
            (Roles, Missing)
        }
        "ROLES_ENV_PATH_INVALID" => (Roles, Invalid),
        "BUNDLED_PLUGINS_MISSING" | "PLUGINS_NOT_FOUND" => (Plugins, Missing),
        "PLUGIN_CONTAINER_UNREADABLE" => (Plugins, Access),
        "PLUGIN_MANIFEST_INVALID" | "HOST_PLUGIN_CONFIG_INVALID" => (Plugins, Invalid),
        "PLUGIN_ID_DUPLICATE" => (Plugins, Conflict),
        "BUNDLED_MIGRATIONS_MISSING"
        | "BUNDLED_KERNEL_MISSING"
        | "BUNDLED_KERNEL_MANIFEST_MISSING" => (Kernel, Missing),
        "OLLAMA_UNREACHABLE" => (ModelService, Unreachable),
        other if other.starts_with("VOICE_") => (Voice, Unknown),
        other if other.starts_with("NETWORK_") => (Network, Unreachable),
        _ => (UnknownScope, Unknown),
    }
}

fn issue(
    code: &str,
    severity: RepairSeverity,
    summary: &str,
    detail: impl Into<String>,
    path: &Path,
    repairable: bool,
) -> InstallationRepairIssue {
    let (scope, category) = classify_repair_issue(code);
    InstallationRepairIssue {
        code: code.to_string(),
        scope,
        category,
        severity,
        summary: summary.to_string(),
        detail: detail.into(),
        path: display(path),
        repairable,
    }
}

fn find_bundled_path(resource_dir: &Path, relative: &Path) -> Option<PathBuf> {
    [
        resource_dir.join("resources").join(relative),
        resource_dir.join(relative),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn role_pack_count(roles_dir: &Path) -> usize {
    let Ok(entries) = fs::read_dir(roles_dir) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|entry| {
            let path = entry.path();
            path.is_dir()
                && (path.join("pipeline.ocblueprint").is_file()
                    || path.join("manifest.json").is_file())
        })
        .count()
}

fn inspect_plugins(
    roles_dir: &Path,
    app_data_dir: &Path,
    issues: &mut Vec<InstallationRepairIssue>,
) -> Vec<String> {
    let host = HostPluginsFile::load(app_data_dir);
    let containers = plugin_scan_container_roots(roles_dir, app_data_dir, &host);
    let mut ids = BTreeSet::new();
    let mut seen = BTreeMap::<String, PathBuf>::new();

    for container in containers {
        let Ok(entries) = fs::read_dir(&container) else {
            issues.push(issue(
                "PLUGIN_CONTAINER_UNREADABLE",
                RepairSeverity::Error,
                "Plugin container cannot be read",
                "The directory exists but read_dir failed. Check filesystem permissions or disk health.",
                &container,
                false,
            ));
            continue;
        };
        for entry in entries.flatten() {
            let plugin_dir = entry.path();
            if !plugin_dir.is_dir() || !plugin_dir.join("manifest.json").is_file() {
                continue;
            }
            match OclivePluginManifest::load_from_dir(&plugin_dir) {
                Ok(manifest) => {
                    let id = manifest.id.trim().to_string();
                    if let Some(previous) = seen.insert(id.clone(), plugin_dir.clone()) {
                        issues.push(issue(
                            "PLUGIN_ID_DUPLICATE",
                            RepairSeverity::Warning,
                            "The same plugin id exists in more than one root",
                            format!(
                                "id={id}; previous={}; current={}",
                                previous.display(),
                                plugin_dir.display()
                            ),
                            &plugin_dir,
                            false,
                        ));
                    }
                    ids.insert(id);
                }
                Err(error) => issues.push(issue(
                    "PLUGIN_MANIFEST_INVALID",
                    RepairSeverity::Error,
                    "A plugin manifest is invalid or unreadable",
                    error,
                    &plugin_dir,
                    false,
                )),
            }
        }
    }
    ids.into_iter().collect()
}

fn inspect_expected_bundle_assets(resource_dir: &Path, issues: &mut Vec<InstallationRepairIssue>) {
    let kernel_name = if cfg!(windows) {
        "oclive-kernel-server.exe"
    } else {
        "oclive-kernel-server"
    };
    for (relative, code, label) in [
        ("roles", "BUNDLED_ROLES_MISSING", "bundled roles"),
        ("plugins", "BUNDLED_PLUGINS_MISSING", "bundled plugins"),
        (
            "migrations",
            "BUNDLED_MIGRATIONS_MISSING",
            "database migrations",
        ),
        (
            "distro-profiles",
            "BUNDLED_PROFILE_MISSING",
            "distribution profiles",
        ),
        (kernel_name, "BUNDLED_KERNEL_MISSING", "kernel executable"),
        (
            "oclive-kernel-server.oclive-manifest.json",
            "BUNDLED_KERNEL_MANIFEST_MISSING",
            "kernel compatibility manifest",
        ),
    ] {
        let relative_path = Path::new(relative);
        if find_bundled_path(resource_dir, relative_path).is_none() {
            issues.push(issue(
                code,
                RepairSeverity::Error,
                &format!("Missing {label}"),
                "The installed package is incomplete. Reinstall this exact version from a verified installer.",
                &resource_dir.join("resources").join(relative_path),
                false,
            ));
        }
    }
}

fn probe_app_data_writable(app_data_dir: &Path, issues: &mut Vec<InstallationRepairIssue>) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let probe = app_data_dir.join(format!(".oclive_repair_write_probe_{timestamp}"));
    if let Err(error) = fs::write(&probe, b"ok") {
        issues.push(issue(
            "APP_DATA_NOT_WRITABLE",
            RepairSeverity::Error,
            "Application data directory is not writable",
            error.to_string(),
            app_data_dir,
            true,
        ));
        return;
    }
    if let Err(error) = fs::remove_file(&probe) {
        issues.push(issue(
            "REPAIR_PROBE_CLEANUP_FAILED",
            RepairSeverity::Warning,
            "The write probe succeeded but its temporary file could not be removed",
            error.to_string(),
            &probe,
            true,
        ));
    }
}

fn ensure_directory(
    path: &Path,
    code: &str,
    label: &str,
    actions: &mut Vec<InstallationRepairAction>,
    issues: &mut Vec<InstallationRepairIssue>,
) -> bool {
    if path.is_dir() {
        actions.push(InstallationRepairAction {
            code: code.to_string(),
            status: RepairActionStatus::Unchanged,
            summary: format!("{label} is ready"),
            detail: display(path),
        });
        return false;
    }
    match fs::create_dir_all(path) {
        Ok(()) => {
            actions.push(InstallationRepairAction {
                code: code.to_string(),
                status: RepairActionStatus::Repaired,
                summary: format!("Created missing {label}"),
                detail: display(path),
            });
            true
        }
        Err(error) => {
            actions.push(InstallationRepairAction {
                code: code.to_string(),
                status: RepairActionStatus::Failed,
                summary: format!("Could not create {label}"),
                detail: error.to_string(),
            });
            issues.push(issue(
                "APP_DATA_CREATE_FAILED",
                RepairSeverity::Error,
                "Application data directory cannot be created",
                error.to_string(),
                path,
                true,
            ));
            false
        }
    }
}

fn inspect_host_config(app_data_dir: &Path, issues: &mut Vec<InstallationRepairIssue>) {
    let path = app_data_dir.join("oclive_host_plugins.json");
    let Ok(raw) = fs::read_to_string(&path) else {
        return;
    };
    if let Err(error) = serde_json::from_str::<serde_json::Value>(&raw) {
        issues.push(issue(
            "HOST_PLUGIN_CONFIG_INVALID",
            RepairSeverity::Warning,
            "Plugin host configuration is invalid JSON",
            format!(
                "{error}. Chat Pro will ignore this file and use safe defaults; the repair tool preserves it for manual recovery."
            ),
            &path,
            false,
        ));
    }
}

/// Inspect and conservatively repair installation-owned directories.
///
/// The only writes are creation of missing application support directories.
/// Existing files are never deleted or overwritten.
#[must_use]
pub fn run_installation_repair(
    resource_dir: &Path,
    roles_dir: &Path,
    app_data_dir: &Path,
) -> InstallationRepairReport {
    let mut actions = Vec::new();
    let mut issues = Vec::new();
    let mut changed = ensure_directory(
        app_data_dir,
        "APP_DATA_DIRECTORY",
        "application data directory",
        &mut actions,
        &mut issues,
    );
    changed |= ensure_directory(
        &app_data_dir.join("plugins"),
        "USER_PLUGIN_DIRECTORY",
        "user plugin directory",
        &mut actions,
        &mut issues,
    );
    changed |= ensure_directory(
        &app_data_dir.join("logs"),
        "SUPPORT_LOG_DIRECTORY",
        "support log directory",
        &mut actions,
        &mut issues,
    );

    if !resource_dir.is_dir() {
        issues.push(issue(
            "RESOURCE_DIRECTORY_MISSING",
            RepairSeverity::Error,
            "The installation resource directory is missing",
            "Select the folder containing A.I.Live Chat Pro.exe and its resources directory.",
            resource_dir,
            false,
        ));
    } else if let Err(error) = fs::read_dir(resource_dir) {
        issues.push(issue(
            "RESOURCE_DIRECTORY_UNREADABLE",
            RepairSeverity::Error,
            "The installation resource directory cannot be read",
            error.to_string(),
            resource_dir,
            false,
        ));
    }
    if app_data_dir.is_dir() {
        probe_app_data_writable(app_data_dir, &mut issues);
    }

    inspect_expected_bundle_assets(resource_dir, &mut issues);
    inspect_host_config(app_data_dir, &mut issues);

    if let Ok(configured) = std::env::var("OCLIVE_ROLES_DIR") {
        let configured_path = PathBuf::from(configured.trim());
        if !configured.trim().is_empty() && !configured_path.is_dir() {
            issues.push(issue(
                "ROLES_ENV_PATH_INVALID",
                RepairSeverity::Warning,
                "OCLIVE_ROLES_DIR points to a missing directory",
                "The invalid override was ignored and Chat Pro fell back to its packaged roles.",
                &configured_path,
                false,
            ));
        }
    }

    let role_count = role_pack_count(roles_dir);
    if !roles_dir.is_dir() {
        issues.push(issue(
            "ROLES_DIRECTORY_MISSING",
            RepairSeverity::Error,
            "The resolved roles directory is missing",
            "The installation is incomplete or OCLIVE_ROLES_DIR is incorrect.",
            roles_dir,
            false,
        ));
    } else if role_count == 0 {
        issues.push(issue(
            "ROLE_PACKS_NOT_FOUND",
            RepairSeverity::Error,
            "No readable role packs were found",
            "Expected a role subdirectory containing pipeline.ocblueprint or manifest.json.",
            roles_dir,
            false,
        ));
    }

    let plugin_ids = inspect_plugins(roles_dir, app_data_dir, &mut issues);
    if plugin_ids.is_empty() {
        issues.push(issue(
            "PLUGINS_NOT_FOUND",
            RepairSeverity::Error,
            "No valid directory plugins were found",
            "Check the packaged plugins directory and plugin manifest files.",
            &roles_dir.parent().unwrap_or(resource_dir).join("plugins"),
            false,
        ));
    }

    let success = !issues
        .iter()
        .any(|entry| entry.severity == RepairSeverity::Error);
    let generated_at_epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    InstallationRepairReport {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        operating_system: std::env::consts::OS.to_string(),
        architecture: std::env::consts::ARCH.to_string(),
        generated_at_epoch_ms,
        success,
        changed,
        restart_required: false,
        resource_dir: display(resource_dir),
        roles_dir: display(roles_dir),
        app_data_dir: display(app_data_dir),
        role_count,
        plugin_count: plugin_ids.len(),
        plugin_ids,
        actions,
        issues,
        report_path: String::new(),
    }
}

fn default_report_path(app_data_dir: &Path) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    app_data_dir
        .join("logs")
        .join(format!("installation-repair-{timestamp}.json"))
}

/// Persist a structured support report. The requested path is used by the
/// external recovery wrapper; the app defaults to its local logs directory.
pub fn write_repair_report(
    report: &mut InstallationRepairReport,
    requested_path: Option<&Path>,
) -> Result<PathBuf, String> {
    let path = requested_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_report_path(Path::new(&report.app_data_dir)));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create report directory {}: {error}", parent.display()))?;
    }
    report.report_path = display(&path);
    let bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| format!("serialize repair report: {error}"))?;
    fs::write(&path, bytes)
        .map_err(|error| format!("write repair report {}: {error}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::{
        classify_repair_issue, run_installation_repair, write_repair_report, RepairIssueCategory,
        RepairIssueScope, RepairSeverity,
    };
    use std::fs;

    fn write_fixture(resource_dir: &std::path::Path) -> std::path::PathBuf {
        let nested = resource_dir.join("resources");
        let roles = resource_dir.join("resources/roles");
        let role = roles.join("mumu");
        let plugin = nested.join("plugins").join("com.example.fixture");
        fs::create_dir_all(&role).expect("role fixture");
        fs::create_dir_all(&plugin).expect("plugin fixture");
        fs::create_dir_all(nested.join("migrations")).expect("migrations fixture");
        fs::create_dir_all(nested.join("distro-profiles")).expect("profile fixture");
        fs::write(role.join("pipeline.ocblueprint"), "{}").expect("role blueprint");
        fs::write(
            plugin.join("manifest.json"),
            r#"{"schema_version":1,"id":"com.example.fixture","version":"1.0.0"}"#,
        )
        .expect("plugin manifest");
        let kernel = if cfg!(windows) {
            "oclive-kernel-server.exe"
        } else {
            "oclive-kernel-server"
        };
        fs::write(nested.join(kernel), "kernel").expect("kernel fixture");
        fs::write(
            nested.join("oclive-kernel-server.oclive-manifest.json"),
            "{}",
        )
        .expect("kernel manifest fixture");
        roles
    }

    #[test]
    fn repairs_support_directories_and_accepts_nested_nsis_layout() {
        let temp = tempfile::tempdir().expect("tempdir");
        let resource_dir = temp.path().join("install");
        let roles = write_fixture(&resource_dir);
        let app_data = temp.path().join("app-data");

        let mut report = run_installation_repair(&resource_dir, &roles, &app_data);
        assert!(report.success, "issues={:?}", report.issues);
        assert!(report.changed);
        assert_eq!(report.role_count, 1);
        assert_eq!(report.plugin_ids, vec!["com.example.fixture"]);
        assert!(app_data.join("plugins").is_dir());
        let report_path = write_repair_report(&mut report, None).expect("write report");
        assert!(report_path.is_file());
    }

    #[test]
    fn reports_invalid_plugin_manifest_with_path_and_stable_code() {
        let temp = tempfile::tempdir().expect("tempdir");
        let resource_dir = temp.path().join("install");
        let roles = write_fixture(&resource_dir);
        let invalid = resource_dir.join("resources/plugins/com.example.invalid/manifest.json");
        fs::create_dir_all(invalid.parent().expect("invalid parent")).expect("invalid dir");
        fs::write(&invalid, "{").expect("invalid manifest");

        let report = run_installation_repair(&resource_dir, &roles, &temp.path().join("data"));
        let issue = report
            .issues
            .iter()
            .find(|entry| entry.code == "PLUGIN_MANIFEST_INVALID")
            .expect("invalid manifest issue");
        assert_eq!(issue.severity, RepairSeverity::Error);
        assert_eq!(issue.scope, RepairIssueScope::Plugins);
        assert_eq!(issue.category, RepairIssueCategory::Invalid);
        assert!(issue.path.contains("com.example.invalid"));
        assert!(!issue.detail.is_empty());
    }

    #[test]
    fn incomplete_installation_reports_all_detected_failures() {
        let temp = tempfile::tempdir().expect("tempdir");
        let missing_install = temp.path().join("missing-install");
        let missing_roles = missing_install.join("resources/roles");

        let report =
            run_installation_repair(&missing_install, &missing_roles, &temp.path().join("data"));
        let codes: std::collections::BTreeSet<&str> = report
            .issues
            .iter()
            .map(|entry| entry.code.as_str())
            .collect();
        assert!(!report.success);
        assert!(codes.contains("RESOURCE_DIRECTORY_MISSING"));
        assert!(codes.contains("BUNDLED_ROLES_MISSING"));
        assert!(codes.contains("BUNDLED_PLUGINS_MISSING"));
        assert!(codes.contains("BUNDLED_KERNEL_MISSING"));
        assert!(codes.contains("ROLES_DIRECTORY_MISSING"));
        assert!(codes.contains("PLUGINS_NOT_FOUND"));
    }

    #[test]
    fn model_service_failures_have_stable_scope_and_category() {
        assert_eq!(
            classify_repair_issue("OLLAMA_UNREACHABLE"),
            (
                RepairIssueScope::ModelService,
                RepairIssueCategory::Unreachable
            )
        );
    }
}
