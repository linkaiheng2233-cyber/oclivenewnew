//! `.ocpak` / `.zip`: ZIP container whose contents mirror a role directory (same as `roles/{id}/`); may also import from an **extracted directory** (same layout).

use crate::error::{AppError, Result};
use crate::infrastructure::storage::RoleStorage;
use crate::models::dto::ImportProgress;
use crate::models::Role;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::{SimpleFileOptions, ZipWriter};
use zip::{CompressionMethod, ZipArchive};

use crate::models::role_manifest_disk::DiskRoleManifest;

fn safe_zip_path(name: &str) -> bool {
    !name.contains("..") && !name.starts_with('/') && !name.starts_with('\\')
}

/// `manifest.json` path priority inside ZIP: pack root first, then single top-level folder, then deeper paths (matches standard export).
fn zip_manifest_path_priority(name: &str) -> Option<u8> {
    if !safe_zip_path(name) {
        return None;
    }
    let n = name.replace('\\', "/");
    let n = n.trim_end_matches('/');
    if n.is_empty() || n.ends_with('/') {
        return None;
    }
    if n == "manifest.json" {
        return Some(0);
    }
    let prefix = n.strip_suffix("/manifest.json")?;
    if prefix.is_empty() {
        return Some(0);
    }
    if !prefix.contains('/') {
        Some(1)
    } else {
        Some(2)
    }
}
/// Pack `roles/{role_id}/` into `.ocpak` (ZIP).
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub fn export_role_pack(storage: &RoleStorage, role_id: &str, dest: &Path) -> Result<()> {
    let src = storage.roles_dir().join(role_id);
    if !src.is_dir() {
        return Err(AppError::RoleNotFound(role_id.to_string()));
    }
    let file = File::create(dest)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for entry in WalkDir::new(&src).min_depth(1) {
        let entry = entry.map_err(|e| AppError::Unknown(e.to_string()))?;
        let path = entry.path();
        if path.is_file() {
            let rel = path
                .strip_prefix(&src)
                .map_err(|_| AppError::InvalidParameter("zip strip".into()))?;
            let name = rel.to_string_lossy().replace('\\', "/");
            if !safe_zip_path(&name) {
                continue;
            }
            zip.start_file(name, options)
                .map_err(|e| AppError::Unknown(e.to_string()))?;
            let mut f = File::open(path)?;
            std::io::copy(&mut f, &mut zip).map_err(|e| AppError::Unknown(e.to_string()))?;
        }
    }
    zip.finish().map_err(|e| AppError::Unknown(e.to_string()))?;
    Ok(())
}

/// Read `manifest.json` from an extracted directory (same layout as after zip extract).
fn peek_role_folder_manifest(dir: &Path) -> Result<(String, String, String)> {
    let root = resolve_extracted_role_root(dir)?;
    let manifest_path = root.join("manifest.json");
    if !manifest_path.is_file() {
        return Err(AppError::InvalidParameter(
            "Role pack format: manifest.json not found".into(),
        ));
    }
    let s = fs::read_to_string(&manifest_path).map_err(AppError::IoError)?;
    let disk: DiskRoleManifest = serde_json::from_str(&s).map_err(|_| {
        AppError::InvalidParameter("Role pack format: manifest.json is invalid JSON".into())
    })?;
    Ok((disk.id, disk.name, disk.version))
}
/// Read `manifest.json` from `.ocpak` / `.zip` or an **extracted directory** for pre-import preview and conflict checks.
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub fn peek_role_pack_manifest(src: &Path) -> Result<(String, String, String)> {
    if src.is_dir() {
        return peek_role_folder_manifest(src);
    }
    let file = File::open(src).map_err(|e| {
        AppError::InvalidParameter(format!("Role pack format: cannot open file ({e})"))
    })?;
    let mut archive = ZipArchive::new(file).map_err(|_| {
        AppError::InvalidParameter("Role pack format: not a valid ZIP/ocpak archive".into())
    })?;
    let mut candidates: Vec<(u8, usize)> = Vec::new();
    for i in 0..archive.len() {
        let f = archive.by_index(i).map_err(|_| {
            AppError::InvalidParameter("Role pack format: archive is corrupted".into())
        })?;
        let name = f.name().to_string();
        if name.ends_with('/') {
            continue;
        }
        if let Some(p) = zip_manifest_path_priority(&name) {
            candidates.push((p, i));
        }
    }
    candidates.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    for (_, i) in candidates {
        let mut f = archive.by_index(i).map_err(|_| {
            AppError::InvalidParameter("Role pack format: archive is corrupted".into())
        })?;
        let mut s = String::new();
        std::io::Read::read_to_string(&mut f, &mut s).map_err(|_| {
            AppError::InvalidParameter("Role pack format: cannot read manifest from archive".into())
        })?;
        let disk: DiskRoleManifest = match serde_json::from_str(&s) {
            Ok(d) => d,
            Err(_) => continue,
        };
        return Ok((disk.id, disk.name, disk.version));
    }
    Err(AppError::InvalidParameter(
        "Role pack format: manifest.json not found in archive".into(),
    ))
}

fn unzip_to(
    src: &Path,
    dest: &Path,
    mut on_entry: impl FnMut(usize, usize, Option<&str>),
) -> Result<()> {
    let file = File::open(src).map_err(|e| {
        AppError::InvalidParameter(format!("Role pack format: cannot open file ({e})"))
    })?;
    let mut archive = ZipArchive::new(file).map_err(|_| {
        AppError::InvalidParameter("Role pack format: not a valid ZIP/ocpak archive".into())
    })?;
    let total = archive.len().max(1);
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| AppError::Unknown(e.to_string()))?;
        let name = file.name().to_string();
        if !safe_zip_path(&name) {
            on_entry(i + 1, total, None);
            continue;
        }
        let outpath = dest.join(&name);
        if name.ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                fs::create_dir_all(p)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
        on_entry(i + 1, total, Some(name.as_str()));
    }
    Ok(())
}

fn resolve_extracted_role_root(extract_dir: &Path) -> Result<PathBuf> {
    if extract_dir.join("manifest.json").exists() {
        return Ok(extract_dir.to_path_buf());
    }
    let dirs: Vec<PathBuf> = fs::read_dir(extract_dir)
        .map_err(AppError::IoError)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    if dirs.len() == 1 && dirs[0].join("manifest.json").exists() {
        return Ok(dirs[0].clone());
    }
    Err(AppError::InvalidParameter(
        "manifest.json not found: expected at pack root or inside a single top-level folder (same layout as a zip extract)."
            .into(),
    ))
}

fn load_role_for_pack_import(storage: &RoleStorage, root: &Path) -> Result<Role> {
    storage.load_role_from_dir(root).map_err(|e| match e {
        AppError::SerializationError(_) | AppError::RoleNotFound(_) => {
            AppError::InvalidParameter("Role pack format: cannot parse role directory".into())
        }
        o => o,
    })
}

/// Install parsed `root` (with `manifest.json`) into `roles/{id}/`.
fn install_role_from_resolved_root<F, P>(
    storage: &RoleStorage,
    root: &Path,
    overwrite: bool,
    mut on_progress: F,
    copy_percent: P,
) -> Result<String>
where
    F: FnMut(ImportProgress),
    P: Fn(usize, usize) -> i32,
{
    let role = load_role_for_pack_import(storage, root)?;
    let id = role.id.clone();
    let dest = storage.roles_dir().join(&id);
    if dest.exists() {
        if !overwrite {
            return Err(AppError::RolePackExists(id));
        }
        fs::remove_dir_all(&dest)?;
    }
    fs::create_dir_all(&dest)?;
    copy_role_tree(root, &dest, |cur, tot, current| {
        let pct = copy_percent(cur, tot).min(99);
        on_progress(ImportProgress {
            percent: pct,
            message: format!("Writing files {}/{}", cur, tot),
            file_index: Some(cur as u32),
            file_total: Some(tot as u32),
            current_file: current.map(str::to_string),
        });
    })?;
    on_progress(ImportProgress {
        percent: 100,
        message: "Import complete".into(),
        file_index: None,
        file_total: None,
        current_file: None,
    });
    Ok(id)
}

fn copy_role_tree(
    src: &Path,
    dest: &Path,
    mut on_file: impl FnMut(usize, usize, Option<&str>),
) -> Result<()> {
    let files: Vec<PathBuf> = WalkDir::new(src)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();
    let total = files.len().max(1);
    for (i, path) in files.iter().enumerate() {
        let rel = path
            .strip_prefix(src)
            .map_err(|_| AppError::InvalidParameter("copy strip".into()))?;
        let target = dest.join(rel);
        if let Some(p) = target.parent() {
            fs::create_dir_all(p)?;
        }
        fs::copy(path, &target)?;
        let rel_display = rel.to_string_lossy().into_owned();
        on_file(i + 1, total, Some(rel_display.as_str()));
    }
    Ok(())
}

/// Copy from extracted directory to `roles/{id}/` (same structure as zip extract).
fn import_role_from_directory<F: FnMut(ImportProgress)>(
    storage: &RoleStorage,
    src: &Path,
    overwrite: bool,
    mut on_progress: F,
) -> Result<String> {
    on_progress(ImportProgress {
        percent: 0,
        message: "Reading folder…".into(),
        file_index: None,
        file_total: None,
        current_file: None,
    });
    let root = resolve_extracted_role_root(src)?;
    install_role_from_resolved_root(storage, &root, overwrite, on_progress, |cur, tot| {
        ((cur as i64 * 100) / tot as i64).min(99) as i32
    })
}
/// Extract `.ocpak` / `.zip` to `roles/{id}/`, or copy from an **extracted directory** (same layout as `roles/{id}/`).
/// Returns [`AppError::RolePackExists`] when the directory exists and `overwrite == false`.
/// `on_progress` is invoked during extract/copy; caller should emit 100% when finished.
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub fn import_role_pack<F: FnMut(ImportProgress)>(
    storage: &RoleStorage,
    src: &Path,
    overwrite: bool,
    mut on_progress: F,
) -> Result<String> {
    if src.is_dir() {
        return import_role_from_directory(storage, src, overwrite, on_progress);
    }
    on_progress(ImportProgress {
        percent: 0,
        message: "Preparing extraction…".into(),
        file_index: None,
        file_total: None,
        current_file: None,
    });
    let td = tempfile::tempdir()?;
    unzip_to(src, td.path(), |cur, tot, current| {
        let pct = ((cur as i64 * 50) / tot as i64).min(50) as i32;
        on_progress(ImportProgress {
            percent: pct,
            message: format!("Extracting {}/{}", cur, tot),
            file_index: Some(cur as u32),
            file_total: Some(tot as u32),
            current_file: current.map(str::to_string),
        });
    })?;
    let root = resolve_extracted_role_root(td.path())?;
    install_role_from_resolved_root(storage, &root, overwrite, on_progress, |cur, tot| {
        (50 + ((cur as i64 * 50) / tot as i64).min(50)) as i32
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::storage::RoleStorage;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn export_import_roundtrip() {
        let roles_src = tempdir().unwrap();
        let roles_dst = tempdir().unwrap();
        fs::create_dir_all(roles_src.path().join("mumu").join("scenes").join("default")).unwrap();
        fs::write(
            roles_src.path().join("mumu").join("manifest.json"),
            r#"{"id":"mumu","name":"M","version":"1","author":"t","description":"d","default_personality":[0.5,0.5,0.5,0.5,0.5,0.5,0.5],"evolution":{},"user_relations":{"friend":{"prompt_hint":"x"}},"default_relation":"friend","memory_config":{"scene_weight_multiplier":1.0,"topic_weights":{}}}"#,
        )
        .unwrap();

        let st = RoleStorage::new(roles_src.path());
        let out_tmp = tempdir().unwrap();
        let pak = out_tmp.path().join("x.ocpak");
        export_role_pack(&st, "mumu", &pak).unwrap();

        let st2 = RoleStorage::new(roles_dst.path());
        let id = import_role_pack(&st2, &pak, true, |_| {}).unwrap();
        assert_eq!(id, "mumu");
        let role = st2.load_role("mumu").unwrap();
        assert_eq!(role.id, "mumu");
    }

    #[test]
    fn import_from_unpacked_folder_matches_zip() {
        let roles_src = tempdir().unwrap();
        let roles_dst = tempdir().unwrap();
        fs::create_dir_all(roles_src.path().join("mumu").join("scenes").join("default")).unwrap();
        fs::write(
            roles_src.path().join("mumu").join("manifest.json"),
            r#"{"id":"mumu","name":"M","version":"1","author":"t","description":"d","default_personality":[0.5,0.5,0.5,0.5,0.5,0.5,0.5],"evolution":{},"user_relations":{"friend":{"prompt_hint":"x"}},"default_relation":"friend","memory_config":{"scene_weight_multiplier":1.0,"topic_weights":{}}}"#,
        )
        .unwrap();

        let st = RoleStorage::new(roles_dst.path());
        let id =
            import_role_pack(&st, roles_src.path().join("mumu").as_path(), true, |_| {}).unwrap();
        assert_eq!(id, "mumu");
        assert!(st.load_role("mumu").is_ok());
    }

    #[test]
    fn peek_zip_prefers_root_manifest_over_deeper_path() {
        let dir = tempdir().unwrap();
        let pak = dir.path().join("peek.zip");
        let file = File::create(&pak).unwrap();
        let mut zip = ZipWriter::new(file);
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        let deep = r#"{"id":"wrong","name":"W","version":"1","author":"t","description":"d","default_personality":[0.5,0.5,0.5,0.5,0.5,0.5,0.5],"evolution":{},"user_relations":{"friend":{"prompt_hint":"x"}},"default_relation":"friend","memory_config":{"scene_weight_multiplier":1.0,"topic_weights":{}}}"#;
        let root = r#"{"id":"right","name":"R","version":"2","author":"t","description":"d","default_personality":[0.5,0.5,0.5,0.5,0.5,0.5,0.5],"evolution":{},"user_relations":{"friend":{"prompt_hint":"x"}},"default_relation":"friend","memory_config":{"scene_weight_multiplier":1.0,"topic_weights":{}}}"#;
        zip.start_file("nested/extra/manifest.json", opts).unwrap();
        zip.write_all(deep.as_bytes()).unwrap();
        zip.start_file("manifest.json", opts).unwrap();
        zip.write_all(root.as_bytes()).unwrap();
        zip.finish().unwrap();

        let (id, name, ver) = peek_role_pack_manifest(&pak).unwrap();
        assert_eq!(id, "right");
        assert_eq!(name, "R");
        assert_eq!(ver, "2");
    }
}
