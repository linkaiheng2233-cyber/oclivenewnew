use crate::error::AppError;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::State;
use walkdir::WalkDir;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackPluginRequest {
    pub plugin_id: String,
    #[serde(default)]
    pub output_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PackPluginResponse {
    pub archive_path: String,
    pub signature_path: String,
    pub sha256: String,
}

fn plugin_root_from_state(state: &AppState, plugin_id: &str) -> Option<PathBuf> {
    let roots = state.directory_plugins.plugin_roots.read();
    roots.get(plugin_id).cloned()
}

fn ensure_manifest_valid(manifest_path: &Path) -> Result<(), AppError> {
    let raw = fs::read_to_string(manifest_path)?;
    let v: serde_json::Value = serde_json::from_str(&raw)?;
    for k in ["id", "name", "version"] {
        let ok = v
            .get(k)
            .and_then(|x| x.as_str())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        if !ok {
            return Err(AppError::InvalidParameter(format!("manifest missing field {}", k)));
        }
    }
    if v.get("process").is_none() && v.get("remote_url").is_none() {
        return Err(AppError::InvalidParameter(
            "manifest must include process or remote_url".into(),
        ));
    }
    Ok(())
}

#[tauri::command]
pub fn pack_plugin(req: PackPluginRequest, state: State<'_, AppState>) -> Result<PackPluginResponse, String> {
    let pid = req.plugin_id.trim();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("plugin_id required".into()).to_frontend_error());
    }
    let root = plugin_root_from_state(&state, pid).ok_or_else(|| {
        AppError::InvalidParameter(format!("plugin not found in catalog: {}", pid)).to_frontend_error()
    })?;
    let manifest_path = root.join("manifest.json");
    ensure_manifest_valid(&manifest_path).map_err(|e| e.to_frontend_error())?;
    let out_dir = req
        .output_dir
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| root.parent().unwrap_or_else(|| Path::new(".")).to_path_buf());
    fs::create_dir_all(&out_dir).map_err(|e| AppError::IoError(e).to_frontend_error())?;
    let archive_path = out_dir.join(format!("{}.oclive-plugin", pid));
    let f = fs::File::create(&archive_path).map_err(|e| AppError::IoError(e).to_frontend_error())?;
    let mut zip = zip::ZipWriter::new(f);
    let opt = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);
    for entry in WalkDir::new(&root).into_iter().flatten() {
        let p = entry.path();
        if p.is_dir() {
            continue;
        }
        let rel = match p.strip_prefix(&root) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let name = rel.to_string_lossy().replace('\\', "/");
        zip.start_file(name, opt)
            .map_err(|e| AppError::Unknown(format!("zip start file failed: {}", e)).to_frontend_error())?;
        let bytes = fs::read(p).map_err(|e| AppError::IoError(e).to_frontend_error())?;
        zip.write_all(&bytes)
            .map_err(|e| AppError::Unknown(format!("zip write failed: {}", e)).to_frontend_error())?;
    }
    zip.finish()
        .map_err(|e| AppError::Unknown(format!("zip finalize failed: {}", e)).to_frontend_error())?;
    let blob = fs::read(&archive_path).map_err(|e| AppError::IoError(e).to_frontend_error())?;
    let mut hasher = Sha256::new();
    hasher.update(&blob);
    let digest_bytes = hasher.finalize();
    let digest = digest_bytes
        .as_slice()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    let sig = serde_json::json!({
        "plugin_id": pid,
        "sha256": digest,
        "archive": archive_path.file_name().and_then(|s| s.to_str()).unwrap_or_default()
    });
    let signature_path = out_dir.join(format!("{}.signature.json", pid));
    fs::write(
        &signature_path,
        serde_json::to_string_pretty(&sig).map_err(AppError::from).map_err(|e| e.to_frontend_error())?,
    )
    .map_err(|e| AppError::IoError(e).to_frontend_error())?;
    Ok(PackPluginResponse {
        archive_path: archive_path.to_string_lossy().to_string(),
        signature_path: signature_path.to_string_lossy().to_string(),
        sha256: digest,
    })
}
