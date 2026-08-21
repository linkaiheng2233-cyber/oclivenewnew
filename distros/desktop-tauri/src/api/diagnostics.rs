//! Lightweight environment self-check (A2.2): Ollama reachability, roles root, app data dir writable.

use crate::api::error::CommandError;
use crate::installation_repair::{
    classify_repair_issue, run_installation_repair, write_repair_report, InstallationRepairIssue,
    InstallationRepairReport, RepairActionStatus, RepairSeverity,
};
use oclive_kernel_host::state::SharedAppState;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, State};

const PROBE_TIMEOUT_SECS: u64 = 8;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentDiagnostics {
    pub ollama_base_url: String,
    pub ollama_reachable: bool,
    /// Brief failure reason when check fails (English/reqwest raw text for troubleshooting; UI copy from frontend i18n).
    pub ollama_detail: String,
    pub roles_dir: String,
    pub roles_dir_exists: bool,
    pub roles_dir_readable: bool,
    pub app_data_dir: String,
    pub app_data_writable: bool,
    pub app_data_detail: String,
}

fn probe_writable_dir(dir: &Path) -> (bool, String) {
    if !dir.exists() {
        if let Err(e) = std::fs::create_dir_all(dir) {
            return (false, format!("create_dir_all: {e}"));
        }
    }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let probe = dir.join(format!(".oclive_write_probe_{nanos}"));
    match std::fs::write(&probe, b"ok") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe);
            (true, String::new())
        }
        Err(e) => (false, e.to_string()),
    }
}

async fn probe_ollama(base: &str) -> (bool, String) {
    let url = format!("{}/api/tags", base.trim_end_matches('/'));
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(PROBE_TIMEOUT_SECS))
        .build()
    {
        Ok(c) => c,
        Err(e) => return (false, format!("reqwest client: {e}")),
    };
    match client.get(&url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                (true, String::new())
            } else {
                (false, format!("HTTP {}", resp.status()))
            }
        }
        Err(e) => (false, e.to_string()),
    }
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn run_environment_diagnostics(
    state: State<'_, SharedAppState>,
) -> Result<EnvironmentDiagnostics, CommandError> {
    let ollama_base_url =
        std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let (ollama_reachable, ollama_detail) = probe_ollama(&ollama_base_url).await;

    let roles_path = state.storage.roles_dir();
    let roles_dir = roles_path.to_string_lossy().into_owned();
    let roles_dir_exists = roles_path.exists();
    let roles_dir_readable = roles_path.is_dir() && std::fs::read_dir(roles_path).is_ok();

    let app_data_path = oclive_kernel_runtime::find_app_data_dir_for_host();
    let app_data_dir = app_data_path.to_string_lossy().into_owned();
    let (app_data_writable, app_data_detail) = probe_writable_dir(&app_data_path);

    Ok(EnvironmentDiagnostics {
        ollama_base_url,
        ollama_reachable,
        ollama_detail,
        roles_dir,
        roles_dir_exists,
        roles_dir_readable,
        app_data_dir,
        app_data_writable,
        app_data_detail,
    })
}

fn fallback_resource_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Run the conservative installation repair and persist a structured support report.
///
/// # Errors
///
/// Returns [`Err`] only when the Tauri command bridge itself cannot complete.
/// Repair failures are returned as structured issues so the user can see every
/// detected problem instead of receiving only the first error.
#[tauri::command]
pub async fn run_environment_repair(
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<InstallationRepairReport, CommandError> {
    let resource_dir = app
        .path()
        .resource_dir()
        .unwrap_or_else(|_| fallback_resource_dir());
    let roles_dir = state.storage.roles_dir();
    let app_data_dir = oclive_kernel_runtime::find_app_data_dir_for_host();

    let mut report = run_installation_repair(&resource_dir, roles_dir, &app_data_dir);
    // This command can run in the desktop shell while the real kernel lives in a
    // sidecar process. Rescanning the shell's in-memory registry would not repair
    // that sidecar and may wait on plugin child-process shutdown. The shared
    // repair core validates the on-disk manifests here; the active kernel scans
    // them through its normal startup and explicit installer flows.
    report.add_action(
        "PLUGIN_MANIFEST_INSPECTION",
        RepairActionStatus::Unchanged,
        "Packaged and user plugin manifests were inspected",
        format!("valid_plugins={}", report.plugin_count),
    );

    let ollama_base_url =
        std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let (ollama_reachable, ollama_detail) = probe_ollama(&ollama_base_url).await;
    if !ollama_reachable {
        let (scope, category) = classify_repair_issue("OLLAMA_UNREACHABLE");
        report.issues.push(InstallationRepairIssue {
            code: "OLLAMA_UNREACHABLE".to_string(),
            scope,
            category,
            severity: RepairSeverity::Warning,
            summary: "The local Ollama model service is unreachable".to_string(),
            detail: ollama_detail,
            path: ollama_base_url,
            repairable: false,
        });
    }
    report.refresh_success();

    if let Err(error) = write_repair_report(&mut report, None) {
        report
            .actions
            .push(crate::installation_repair::InstallationRepairAction {
                code: "REPAIR_REPORT_WRITE".to_string(),
                status: RepairActionStatus::Failed,
                summary: "Could not write the support report".to_string(),
                detail: error.clone(),
            });
        let (scope, category) = classify_repair_issue("REPAIR_REPORT_WRITE_FAILED");
        report.issues.push(InstallationRepairIssue {
            code: "REPAIR_REPORT_WRITE_FAILED".to_string(),
            scope,
            category,
            severity: RepairSeverity::Error,
            summary: "The repair completed but its support report could not be saved".to_string(),
            detail: error,
            path: app_data_dir.join("logs").to_string_lossy().into_owned(),
            repairable: false,
        });
        report.refresh_success();
    }

    Ok(report)
}
