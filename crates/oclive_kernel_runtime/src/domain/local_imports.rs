//! 本地导入投放目录：扫描候选、路径约束、文本读取（宿主无关）。
//!
//! 发行版仅负责 `app_data` 路径解析与 UI；核心逻辑在此模块统一实现。

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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

#[inline]
pub fn imports_root(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("imports")
}

pub fn ensure_import_folders_exist(imports_root: &Path) -> Result<()> {
    let dirs = [
        imports_root.join("roles"),
        imports_root.join("plugins").join("plugin"),
        imports_root.join("plugins").join("module"),
        imports_root.join("profiles"),
    ];
    for d in dirs {
        fs::create_dir_all(&d).map_err(AppError::IoError)?;
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

pub fn list_local_import_candidates(imports_root: &Path) -> Result<Vec<LocalImportCandidate>> {
    ensure_import_folders_exist(imports_root)?;
    let mut out: Vec<LocalImportCandidate> = Vec::new();

    scan_dir_files(
        &mut out,
        LocalImportKind::RolePack,
        &imports_root.join("roles"),
        &["ocpak", "zip"],
    );

    scan_dir_files(
        &mut out,
        LocalImportKind::PluginArchive,
        &imports_root.join("plugins").join("plugin"),
        &["zip", "oclive-plugin"],
    );
    scan_plugin_dirs(&mut out, &imports_root.join("plugins").join("plugin"));

    scan_dir_files(
        &mut out,
        LocalImportKind::ModuleJson,
        &imports_root.join("plugins").join("module"),
        &["json"],
    );

    scan_dir_files(
        &mut out,
        LocalImportKind::ProfileJson,
        &imports_root.join("profiles"),
        &["json"],
    );

    out.sort_by_key(|b| std::cmp::Reverse(b.modified_ms.unwrap_or(0)));
    Ok(out)
}

pub fn read_import_text(path: &Path, max_bytes: usize) -> Result<String> {
    if !path.is_file() {
        return Err(AppError::InvalidParameter(format!(
            "file not found: {}",
            path.display()
        )));
    }
    let meta = fs::metadata(path)?;
    if meta.len() as usize > max_bytes {
        return Err(AppError::InvalidParameter(format!(
            "file too large (>{} bytes): {}",
            max_bytes,
            path.display()
        )));
    }
    fs::read_to_string(path).map_err(AppError::IoError)
}

/// 将用户传入路径规范化，并校验其落在 `imports_root` 之下（防路径逃逸）。
pub fn resolve_path_under_imports_root(user_path: &str, imports_root: &Path) -> Result<PathBuf> {
    let trimmed = user_path.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidParameter("empty path".to_string()));
    }
    let p = PathBuf::from(trimmed);
    let p = p
        .canonicalize()
        .map_err(|e| AppError::InvalidParameter(format!("path canonicalize: {}", e)))?;
    let root_canon = imports_root
        .canonicalize()
        .unwrap_or_else(|_| imports_root.to_path_buf());
    if !p.starts_with(&root_canon) {
        return Err(AppError::PermissionDenied(
            "path must be under app_data/imports".to_string(),
        ));
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn ensure_and_list_role_pack() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("imports");
        ensure_import_folders_exist(&root).unwrap();
        let roles = root.join("roles");
        let mut f = fs::File::create(roles.join("test.ocpak")).unwrap();
        f.write_all(b"x").unwrap();
        let list = list_local_import_candidates(&root).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].kind, LocalImportKind::RolePack);
        assert!(list[0].path.ends_with("test.ocpak"));
    }

    #[test]
    fn resolve_accepts_inside_imports() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("imports");
        ensure_import_folders_exist(&root).unwrap();
        let inside = root.join("profiles").join("p.json");
        fs::File::create(&inside).unwrap();
        let resolved = resolve_path_under_imports_root(inside.to_str().unwrap(), &root).unwrap();
        assert!(resolved.is_file());
    }

    #[test]
    fn resolve_rejects_outside_imports() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("imports");
        ensure_import_folders_exist(&root).unwrap();
        let outside = tmp.path().join("outside.txt");
        fs::File::create(&outside).unwrap();
        let r = resolve_path_under_imports_root(outside.to_str().unwrap(), &root);
        assert!(r.is_err());
    }

    #[test]
    fn read_import_text_respects_max() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("small.txt");
        fs::write(&p, "hi").unwrap();
        assert_eq!(read_import_text(&p, 10).unwrap(), "hi");
        let r = read_import_text(&p, 1);
        assert!(r.is_err());
    }
}
