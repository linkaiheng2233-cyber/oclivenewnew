use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalImportKind {
    RolePack,
    PluginArchive,
    PluginDir,
    ModuleJson,
    ProfileJson,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalImportCandidate {
    pub kind: LocalImportKind,
    pub path: String,
    pub file_name: String,
    /// 可选：同目录下的签名文件（如 `xxx.signature.json`）
    #[serde(default)]
    pub related_signature_path: Option<String>,
    #[serde(default)]
    pub size_bytes: Option<u64>,
    #[serde(default)]
    pub modified_ms: Option<u64>,
}

pub fn imports_root(state: &AppState) -> PathBuf {
    state.directory_plugins.app_data_dir().join("imports")
}

pub fn ensure_import_folders_exist(state: &AppState) -> Result<(), String> {
    let root = imports_root(state);
    let dirs = [
        root.join("roles"),
        root.join("plugins").join("plugin"),
        root.join("plugins").join("module"),
        root.join("profiles"),
    ];
    for d in dirs {
        fs::create_dir_all(&d).map_err(|e| format!("create imports folder {}: {}", d.display(), e))?;
    }
    Ok(())
}

fn meta_for_path(p: &Path) -> (Option<u64>, Option<u64>) {
    let Ok(m) = fs::metadata(p) else {
        return (None, None);
    };
    let size = if m.is_file() { Some(m.len()) } else { None };
    let modified = m
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis().min(u64::MAX as u128) as u64);
    (size, modified)
}

fn push_file(
    out: &mut Vec<LocalImportCandidate>,
    kind: LocalImportKind,
    p: &Path,
    related_signature_path: Option<String>,
) {
    let file_name = p
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    if file_name.trim().is_empty() {
        return;
    }
    let (size_bytes, modified_ms) = meta_for_path(p);
    out.push(LocalImportCandidate {
        kind,
        path: p.to_string_lossy().to_string(),
        file_name,
        related_signature_path,
        size_bytes,
        modified_ms,
    });
}

fn scan_dir_files(
    out: &mut Vec<LocalImportCandidate>,
    kind: LocalImportKind,
    dir: &Path,
    exts_lower: &[&str],
) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if !p.is_file() {
            continue;
        }
        let ext = p
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if exts_lower.iter().any(|x| *x == ext) {
            let rel_sig = if kind == LocalImportKind::PluginArchive && ext == "oclive-plugin" {
                let stem = p
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !stem.is_empty() {
                    let sig = p.with_file_name(format!("{}.signature.json", stem));
                    if sig.is_file() {
                        Some(sig.to_string_lossy().to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
            push_file(out, kind.clone(), &p, rel_sig);
        }
    }
}

fn scan_plugin_dirs(out: &mut Vec<LocalImportCandidate>, dir: &Path) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if !p.is_dir() {
            continue;
        }
        if p.join("manifest.json").is_file() {
            push_file(out, LocalImportKind::PluginDir, &p, None);
        }
    }
}

pub fn list_local_import_candidates(state: &AppState) -> Result<Vec<LocalImportCandidate>, String> {
    ensure_import_folders_exist(state)?;
    let root = imports_root(state);
    let mut out: Vec<LocalImportCandidate> = Vec::new();

    scan_dir_files(
        &mut out,
        LocalImportKind::RolePack,
        &root.join("roles"),
        &["ocpak", "zip"],
    );

    // plugin archives
    scan_dir_files(
        &mut out,
        LocalImportKind::PluginArchive,
        &root.join("plugins").join("plugin"),
        &["zip", "oclive-plugin"],
    );
    // plugin directories
    scan_plugin_dirs(&mut out, &root.join("plugins").join("plugin"));

    // no-code module JSON
    scan_dir_files(
        &mut out,
        LocalImportKind::ModuleJson,
        &root.join("plugins").join("module"),
        &["json"],
    );

    // profile JSON
    scan_dir_files(
        &mut out,
        LocalImportKind::ProfileJson,
        &root.join("profiles"),
        &["json"],
    );

    // newest first
    out.sort_by_key(|b| std::cmp::Reverse(b.modified_ms.unwrap_or(0)));
    Ok(out)
}

pub fn read_import_text(path: &Path, max_bytes: usize) -> Result<String, String> {
    if !path.is_file() {
        return Err(format!("file not found: {}", path.display()));
    }
    let meta = fs::metadata(path).map_err(|e| e.to_string())?;
    if meta.len() as usize > max_bytes {
        return Err(format!(
            "file too large (>{} bytes): {}",
            max_bytes,
            path.display()
        ));
    }
    fs::read_to_string(path).map_err(|e| format!("read file failed: {}", e))
}

