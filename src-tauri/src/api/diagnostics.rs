//! Lightweight environment self-check (A2.2): Ollama reachability, roles root, app data dir writable.

use crate::state::AppState;
use serde::Serialize;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;
use crate::api::error::CommandError;

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
    state: State<'_, AppState>,
) -> Result<EnvironmentDiagnostics, CommandError> {
    let ollama_base_url =
        std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let (ollama_reachable, ollama_detail) = probe_ollama(&ollama_base_url).await;

    let roles_path = state.storage.roles_dir();
    let roles_dir = roles_path.to_string_lossy().into_owned();
    let roles_dir_exists = roles_path.exists();
    let roles_dir_readable = roles_path.is_dir() && std::fs::read_dir(roles_path).is_ok();

    let app_data_path = state.directory_plugins.app_data_dir();
    let app_data_dir = app_data_path.to_string_lossy().into_owned();
    let (app_data_writable, app_data_detail) = probe_writable_dir(app_data_path);

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
