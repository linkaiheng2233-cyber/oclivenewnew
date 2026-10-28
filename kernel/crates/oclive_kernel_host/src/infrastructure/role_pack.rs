//! `.ocpak` / `.zip`: ZIP container whose contents mirror a role directory (same as `roles/{id}/`); may also import from an **extracted directory** (same layout).

use crate::error::{AppError, Result};
use crate::infrastructure::storage::RoleStorage;
use crate::models::dto::ImportProgress;
use crate::models::Role;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::{SimpleFileOptions, ZipWriter};
use zip::{CompressionMethod, ZipArchive};

use crate::models::role_manifest_disk::DiskRoleManifest;
use oclive_validation::PIPELINE_BLUEPRINT_FILENAME;
use serde::Deserialize;

#[derive(Clone, Copy)]
struct PackImportLimits {
    entries: usize,
    entry_bytes: u64,
    total_bytes: u64,
}

const PACK_IMPORT_LIMITS: PackImportLimits = PackImportLimits {
    entries: 20_000,
    entry_bytes: 256 * 1024 * 1024,
    total_bytes: 1024 * 1024 * 1024,
};
const MAX_ROLE_METADATA_BYTES: u64 = 4 * 1024 * 1024;

fn read_preview_metadata(reader: impl Read) -> Result<String> {
    let mut bytes = Vec::new();
    reader
        .take(MAX_ROLE_METADATA_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_ROLE_METADATA_BYTES {
        return Err(AppError::InvalidParameter(
            "Role pack metadata exceeds 4 MiB".into(),
        ));
    }
    String::from_utf8(bytes)
        .map_err(|_| AppError::InvalidParameter("Role metadata must be UTF-8".into()))
}

fn copy_role_entry_bounded(
    reader: impl Read,
    writer: &mut impl std::io::Write,
    limit: u64,
) -> Result<u64> {
    let written = std::io::copy(&mut reader.take(limit.saturating_add(1)), writer)?;
    if written > limit {
        return Err(AppError::InvalidParameter(
            "Role pack expanded data exceeds import budget".into(),
        ));
    }
    Ok(written)
}

#[derive(Deserialize)]
struct BlueprintPackPreview {
    meta: BlueprintPackPreviewMeta,
}

#[derive(Deserialize)]
struct BlueprintPackPreviewMeta {
    id: String,
    name: String,
    version: String,
}

fn safe_zip_path(name: &str) -> bool {
    let normalized = name.replace('\\', "/");
    !normalized.is_empty()
        && !normalized.starts_with('/')
        && !normalized.contains(':')
        && !normalized.chars().any(char::is_control)
        && normalized
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

/// Role metadata path priority inside ZIP: shallower paths first, then blueprint before legacy manifest.
fn zip_preview_path_priority(name: &str) -> Option<(u8, u8)> {
    if !safe_zip_path(name) {
        return None;
    }
    let n = name.replace('\\', "/");
    let n = n.trim_end_matches('/');
    if n.is_empty() || n.ends_with('/') {
        return None;
    }
    let (prefix, format_priority) = if n == PIPELINE_BLUEPRINT_FILENAME {
        ("", 0)
    } else if n == "manifest.json" {
        ("", 1)
    } else if let Some(prefix) = n.strip_suffix(&format!("/{PIPELINE_BLUEPRINT_FILENAME}")) {
        (prefix, 0)
    } else {
        let prefix = n.strip_suffix("/manifest.json")?;
        (prefix, 1)
    };
    let depth_priority = if prefix.is_empty() {
        0
    } else if !prefix.contains('/') {
        1
    } else {
        2
    };
    Some((depth_priority, format_priority))
}

fn parse_pack_preview(name: &str, raw: &str) -> Option<(String, String, String)> {
    if name
        .replace('\\', "/")
        .ends_with(PIPELINE_BLUEPRINT_FILENAME)
    {
        let blueprint: BlueprintPackPreview = serde_json::from_str(raw).ok()?;
        Some((
            blueprint.meta.id,
            blueprint.meta.name,
            blueprint.meta.version,
        ))
    } else {
        let manifest: DiskRoleManifest = serde_json::from_str(raw).ok()?;
        Some((manifest.id, manifest.name, manifest.version))
    }
}
/// Pack `roles/{role_id}/` into `.ocpak` (ZIP with a `{role_id}/` top-level directory).
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub fn export_role_pack(storage: &RoleStorage, role_id: &str, dest: &Path) -> Result<()> {
    let src = storage.role_dir_path(role_id)?;
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
            let rel_name = rel.to_string_lossy().replace('\\', "/");
            let name = format!("{role_id}/{rel_name}");
            if !entry.file_type().is_file() {
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

/// Read role metadata from an extracted directory (same layout as after zip extract).
fn peek_role_folder_manifest(dir: &Path) -> Result<(String, String, String)> {
    let root = find_extracted_role_root(dir)?;
    for file_name in [PIPELINE_BLUEPRINT_FILENAME, "manifest.json"] {
        let path = root.join(file_name);
        if !path.is_file() {
            continue;
        }
        let raw = read_preview_metadata(File::open(&path)?)?;
        return parse_pack_preview(file_name, &raw).ok_or_else(|| {
            AppError::InvalidParameter(format!(
                "Role pack format: {file_name} has invalid role metadata"
            ))
        });
    }
    Err(AppError::InvalidParameter(format!(
        "Role pack format: {PIPELINE_BLUEPRINT_FILENAME} or manifest.json not found"
    )))
}
/// Read role metadata from `.ocpak` / `.zip` or an **extracted directory** for pre-import preview and conflict checks.
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
    if archive.len() > PACK_IMPORT_LIMITS.entries {
        return Err(AppError::InvalidParameter(
            "Role pack exceeds 20000 archive entries".into(),
        ));
    }
    let mut candidates: Vec<(u8, u8, usize)> = Vec::new();
    for i in 0..archive.len() {
        let f = archive.by_index(i).map_err(|_| {
            AppError::InvalidParameter("Role pack format: archive is corrupted".into())
        })?;
        let name = f
            .name()
            .map_err(|_| {
                AppError::InvalidParameter("Role pack format: archive entry name is invalid".into())
            })?
            .to_string();
        if name.ends_with('/') {
            continue;
        }
        if let Some((depth, format)) = zip_preview_path_priority(&name) {
            candidates.push((depth, format, i));
        }
    }
    candidates.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
    });
    for (_, _, i) in candidates {
        let mut f = archive.by_index(i).map_err(|_| {
            AppError::InvalidParameter("Role pack format: archive is corrupted".into())
        })?;
        let name = f
            .name()
            .map_err(|_| {
                AppError::InvalidParameter("Role pack format: archive entry name is invalid".into())
            })?
            .to_string();
        let s = read_preview_metadata(&mut f)?;
        if let Some(preview) = parse_pack_preview(&name, &s) {
            return Ok(preview);
        }
    }
    Err(AppError::InvalidParameter(format!(
        "Role pack format: {PIPELINE_BLUEPRINT_FILENAME} or manifest.json not found in archive"
    )))
}

fn unzip_to(
    src: &Path,
    dest: &Path,
    on_entry: impl FnMut(usize, usize, Option<&str>),
) -> Result<()> {
    unzip_with_limits(src, dest, on_entry, PACK_IMPORT_LIMITS)
}

fn unzip_with_limits(
    src: &Path,
    dest: &Path,
    mut on_entry: impl FnMut(usize, usize, Option<&str>),
    limits: PackImportLimits,
) -> Result<()> {
    let file = File::open(src).map_err(|e| {
        AppError::InvalidParameter(format!("Role pack format: cannot open file ({e})"))
    })?;
    let mut archive = ZipArchive::new(file).map_err(|_| {
        AppError::InvalidParameter("Role pack format: not a valid ZIP/ocpak archive".into())
    })?;
    if archive.len() > limits.entries {
        return Err(AppError::InvalidParameter(format!(
            "Role pack exceeds {} archive entries",
            limits.entries
        )));
    }
    let mut expanded = 0u64;
    let total = archive.len().max(1);
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| AppError::Unknown(e.to_string()))?;
        let name = file
            .name()
            .map_err(|e| AppError::Unknown(e.to_string()))?
            .to_string();
        let normalized_name = name.replace('\\', "/");
        if file
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(AppError::InvalidParameter(
                "Role pack cannot contain symbolic links".into(),
            ));
        }
        if !safe_zip_path(&name) {
            on_entry(i + 1, total, None);
            continue;
        }
        let outpath = dest.join(&normalized_name);
        if normalized_name.ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            let remaining = limits.total_bytes.saturating_sub(expanded);
            let allowed = limits.entry_bytes.min(remaining);
            if file.size() > allowed {
                return Err(AppError::InvalidParameter(
                    "Role pack expanded data exceeds import budget".into(),
                ));
            }
            if let Some(p) = outpath.parent() {
                fs::create_dir_all(p)?;
            }
            let mut outfile = File::create(&outpath)?;
            expanded += copy_role_entry_bounded(&mut file, &mut outfile, allowed)?;
        }
        on_entry(i + 1, total, Some(normalized_name.as_str()));
    }
    Ok(())
}

fn find_extracted_role_root(extract_dir: &Path) -> Result<PathBuf> {
    if has_role_pack_marker(extract_dir) {
        return Ok(extract_dir.to_path_buf());
    }
    let dirs: Vec<PathBuf> = fs::read_dir(extract_dir)
        .map_err(AppError::IoError)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    if dirs.len() == 1 && has_role_pack_marker(&dirs[0]) {
        return Ok(dirs[0].clone());
    }
    Err(AppError::InvalidParameter(format!(
        "{PIPELINE_BLUEPRINT_FILENAME} or manifest.json not found: expected at pack root or inside a single top-level folder (same layout as a zip extract)."
    )))
}

fn has_role_pack_marker(dir: &Path) -> bool {
    dir.join(PIPELINE_BLUEPRINT_FILENAME).is_file() || dir.join("manifest.json").is_file()
}

fn load_role_for_pack_import(storage: &RoleStorage, root: &Path) -> Result<Role> {
    storage.load_role_from_dir(root).map_err(|e| match e {
        AppError::SerializationError(_) | AppError::RoleNotFound(_) => {
            AppError::InvalidParameter("Role pack format: cannot parse role directory".into())
        }
        o => o,
    })
}

/// Install parsed role-pack `root` into `roles/{id}/`.
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
    let _guard = super::role_files::lock_role_writes(storage.roles_dir())?;
    super::role_files::recover_imports_locked(storage.roles_dir())?;
    let role = load_role_for_pack_import(storage, root)?;
    let id = role.id.clone();
    oclive_validation::validate_role_id(&id).map_err(AppError::InvalidParameter)?;
    let dest = storage.role_dir_path(&id)?;
    let source = root.canonicalize()?;
    let resolved_dest = if dest.exists() {
        dest.canonicalize()?
    } else {
        storage.roles_dir().canonicalize()?.join(&id)
    };
    if source.starts_with(&resolved_dest) || resolved_dest.starts_with(&source) {
        return Err(AppError::InvalidParameter(
            "role import source and destination overlap".into(),
        ));
    }
    if dest.exists() && !overwrite {
        return Err(AppError::RolePackExists(id));
    }
    let workspace = super::role_files::import_workspace(storage.roles_dir())?;
    let staged = workspace.path().join("next").join(&id);
    fs::create_dir_all(&staged)?;
    copy_role_tree(&source, &staged, |cur, tot, current| {
        let pct = copy_percent(cur, tot).min(99);
        on_progress(ImportProgress {
            percent: pct,
            message: format!("Writing files {}/{}", cur, tot),
            file_index: Some(cur as u32),
            file_total: Some(tot as u32),
            current_file: current.map(str::to_string),
        });
    })?;
    let staged_role = load_role_for_pack_import(storage, &staged)?;
    if staged_role.id != id {
        return Err(AppError::InvalidParameter(
            "role id changed while copying".into(),
        ));
    }
    super::role_files::commit_import(workspace, &dest, &id)?;
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
    let mut files = Vec::new();
    for entry in WalkDir::new(src).min_depth(1) {
        let entry =
            entry.map_err(|e| AppError::InvalidParameter(format!("read role tree: {e}")))?;
        if entry.path_is_symlink() || !entry.path().canonicalize()?.starts_with(src) {
            return Err(AppError::InvalidParameter(
                "role import cannot contain linked files/directories".into(),
            ));
        }
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }
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
    let root = find_extracted_role_root(src)?;
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
    let root = find_extracted_role_root(td.path())?;
    install_role_from_resolved_root(storage, &root, overwrite, on_progress, |cur, tot| {
        (50 + ((cur as i64 * 50) / tot as i64).min(50)) as i32
    })
}

fn canonical_allowed_root(path: &Path) -> Option<PathBuf> {
    path.canonicalize().ok().or_else(|| {
        if path.exists() {
            Some(path.to_path_buf())
        } else {
            None
        }
    })
}

fn path_under_root(candidate: &Path, root: &Path) -> bool {
    let Some(root_canon) = canonical_allowed_root(root) else {
        return false;
    };
    let Some(candidate_canon) = canonical_allowed_root(candidate) else {
        return false;
    };
    candidate_canon.starts_with(&root_canon)
}

/// Validates `import_role` sources from directory-plugin bridge (not native file-picker imports).
///
/// # Errors
///
/// Returns [`AppError::InvalidParameter`] when the path escapes allowed roots.
pub fn validate_bridge_import_role_source(
    storage: &RoleStorage,
    app_data_dir: &Path,
    src: &Path,
) -> Result<PathBuf> {
    if src.as_os_str().is_empty() {
        return Err(AppError::InvalidParameter("import path required".into()));
    }
    if !src.exists() {
        return Err(AppError::InvalidParameter(format!(
            "import path not found: {}",
            src.display()
        )));
    }
    let roles_root = storage.roles_dir();
    if !path_under_root(src, roles_root) && !path_under_root(src, app_data_dir) {
        return Err(AppError::InvalidParameter(format!(
            "import_role: path must be under roles dir ({}) or app data ({})",
            roles_root.display(),
            app_data_dir.display()
        )));
    }
    let canonical = src
        .canonicalize()
        .map_err(|e| AppError::InvalidParameter(format!("import path: {e}")))?;
    if canonical.is_dir() {
        find_extracted_role_root(&canonical)?;
    } else {
        let ext = canonical
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ext != "ocpak" && ext != "zip" {
            return Err(AppError::InvalidParameter(
                "import_role: path must be .ocpak, .zip, or a role directory".into(),
            ));
        }
    }
    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::storage::RoleStorage;
    use std::io::Write;
    use tempfile::tempdir;

    fn write_minimal_blueprint_role(roles_root: &Path, id: &str) {
        let role = roles_root.join(id);
        fs::create_dir_all(role.join("scenes").join("default")).unwrap();
        fs::write(
            role.join("core_personality.txt"),
            "A stable test personality.\n",
        )
        .unwrap();
        fs::write(
            role.join("scenes").join("default").join("scene.json"),
            r#"{"name":"Default","time_windows":[],"keywords":[],"events":[]}"#,
        )
        .unwrap();
        let blueprint = serde_json::json!({
            "schema_version": 2,
            "meta": {
                "id": id,
                "name": "Blueprint Role",
                "version": "0.1.0",
                "author": "test",
                "description": "pure v2 import fixture",
                "personality": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
                "relations": {
                    "friend": { "initial_favorability": 50.0, "favor_multiplier": 1.0 }
                },
                "default_relation": "friend",
                "scenes": ["default"],
                "interaction_mode": "immersive"
            },
            "slot_registry": {
                "memory": { "type": "memory", "label": "Memory", "backend": "builtin", "position": 0 },
                "emotion": { "type": "emotion", "label": "Emotion", "backend": "builtin", "position": 0 },
                "complex_emotion": { "type": "complex_emotion", "label": "Complex emotion", "backend": "builtin", "position": 1 },
                "event": { "type": "event", "label": "Event", "backend": "builtin", "position": 0 },
                "prompt": { "type": "prompt", "label": "Prompt", "backend": "builtin", "position": 0 },
                "llm": { "type": "llm", "label": "LLM", "backend": "ollama", "position": 0 },
                "agent": { "type": "agent", "label": "Agent", "backend": "builtin", "position": 0 }
            }
        });
        fs::write(
            role.join(PIPELINE_BLUEPRINT_FILENAME),
            serde_json::to_string_pretty(&blueprint).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn zip_entry_paths_reject_platform_specific_traversal() {
        for unsafe_path in [
            "../manifest.json",
            "..\\manifest.json",
            "C:\\manifest.json",
            "C:/manifest.json",
            "\\\\server\\share\\manifest.json",
            "/absolute/manifest.json",
        ] {
            assert!(!safe_zip_path(unsafe_path), "accepted {unsafe_path:?}");
        }
        assert!(safe_zip_path("mumu/scenes/default/scene.json"));
    }

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
    fn import_pure_blueprint_pack_from_zip_and_directory() {
        let roles_src = tempdir().unwrap();
        let roles_zip_dst = tempdir().unwrap();
        let roles_dir_dst = tempdir().unwrap();
        write_minimal_blueprint_role(roles_src.path(), "blueprint_role");

        let source = RoleStorage::new(roles_src.path());
        let archive_dir = tempdir().unwrap();
        let archive = archive_dir.path().join("blueprint.ocpak");
        export_role_pack(&source, "blueprint_role", &archive).unwrap();

        let preview = peek_role_pack_manifest(&archive).unwrap();
        assert_eq!(
            preview,
            (
                "blueprint_role".into(),
                "Blueprint Role".into(),
                "0.1.0".into()
            )
        );

        let zip_target = RoleStorage::new(roles_zip_dst.path());
        let zip_id = import_role_pack(&zip_target, &archive, false, |_| {}).unwrap();
        assert_eq!(zip_id, "blueprint_role");
        assert_eq!(
            zip_target.load_role(&zip_id).unwrap().name,
            "Blueprint Role"
        );

        let dir_target = RoleStorage::new(roles_dir_dst.path());
        let dir_id = import_role_pack(
            &dir_target,
            &roles_src.path().join("blueprint_role"),
            false,
            |_| {},
        )
        .unwrap();
        assert_eq!(dir_id, "blueprint_role");
        assert_eq!(
            dir_target.load_role(&dir_id).unwrap().name,
            "Blueprint Role"
        );
    }

    #[test]
    fn blueprint_preview_wins_over_legacy_manifest_at_same_depth() {
        let dir = tempdir().unwrap();
        write_minimal_blueprint_role(dir.path(), "preferred");
        let role = dir.path().join("preferred");
        fs::write(
            role.join("manifest.json"),
            r#"{"id":"legacy","name":"Legacy","version":"9.9.9"}"#,
        )
        .unwrap();

        assert_eq!(
            peek_role_pack_manifest(&role).unwrap(),
            ("preferred".into(), "Blueprint Role".into(), "0.1.0".into())
        );
    }

    #[test]
    fn zip_preview_prefers_blueprint_over_manifest_at_same_depth() {
        let dir = tempdir().unwrap();
        let pak = dir.path().join("mixed.zip");
        let file = File::create(&pak).unwrap();
        let mut zip = ZipWriter::new(file);
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        zip.start_file("role/manifest.json", opts).unwrap();
        zip.write_all(br#"{"id":"legacy","name":"Legacy","version":"9.9.9"}"#)
            .unwrap();
        zip.start_file("role/pipeline.ocblueprint", opts).unwrap();
        zip.write_all(
            br#"{"schema_version":2,"meta":{"id":"blueprint","name":"Blueprint","version":"1.2.3"},"slot_registry":{}}"#,
        )
        .unwrap();
        zip.finish().unwrap();

        assert_eq!(
            peek_role_pack_manifest(&pak).unwrap(),
            ("blueprint".into(), "Blueprint".into(), "1.2.3".into())
        );
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

    #[test]
    fn bridge_import_rejects_path_outside_allowed_roots() {
        let roles = tempdir().unwrap();
        let app = tempdir().unwrap();
        let outside = tempdir().unwrap();
        fs::write(outside.path().join("pack.zip"), b"fake").unwrap();
        let st = RoleStorage::new(roles.path());
        let err =
            validate_bridge_import_role_source(&st, app.path(), &outside.path().join("pack.zip"))
                .unwrap_err();
        assert!(matches!(err, AppError::InvalidParameter(_)));
    }

    #[test]
    fn self_overwrite_is_rejected_without_touching_role() {
        let roles = tempdir().unwrap();
        write_minimal_blueprint_role(roles.path(), "example");
        let role = roles.path().join("example");
        let before = fs::read(role.join("core_personality.txt")).unwrap();
        let storage = RoleStorage::new(roles.path());
        assert!(import_role_pack(&storage, &role, true, |_| {}).is_err());
        assert_eq!(fs::read(role.join("core_personality.txt")).unwrap(), before);
        assert!(storage.load_role("example").is_ok());
    }

    #[test]
    fn archive_entry_and_expanded_byte_budgets_are_enforced() {
        let fixture = tempdir().unwrap();
        let archive_path = fixture.path().join("budget.zip");
        let file = File::create(&archive_path).unwrap();
        let mut zip = ZipWriter::new(file);
        for name in ["one.txt", "two.txt", "three.txt"] {
            zip.start_file(name, SimpleFileOptions::default()).unwrap();
            zip.write_all(b"abcdefghijkl").unwrap();
        }
        zip.finish().unwrap();
        for limits in [
            PackImportLimits {
                entries: 2,
                entry_bytes: 32,
                total_bytes: 100,
            },
            PackImportLimits {
                entries: 10,
                entry_bytes: 8,
                total_bytes: 100,
            },
            PackImportLimits {
                entries: 10,
                entry_bytes: 16,
                total_bytes: 20,
            },
        ] {
            let output = tempdir().unwrap();
            assert!(unzip_with_limits(&archive_path, output.path(), |_, _, _| {}, limits).is_err());
        }
    }

    #[test]
    fn actual_stream_budget_does_not_trust_declared_size() {
        let mut output = Vec::new();
        assert!(
            copy_role_entry_bounded(std::io::Cursor::new(vec![0; 1000]), &mut output, 8).is_err()
        );
        assert_eq!(output.len(), 9);
    }

    #[test]
    fn preview_metadata_has_a_separate_small_budget() {
        assert!(read_preview_metadata(std::io::Cursor::new(vec![
            b' ';
            MAX_ROLE_METADATA_BYTES as usize
                + 1
        ]))
        .is_err());
    }

    #[test]
    fn interrupted_copy_leaves_previous_role_intact() {
        let roles = tempdir().unwrap();
        let incoming = tempdir().unwrap();
        write_minimal_blueprint_role(roles.path(), "example");
        write_minimal_blueprint_role(incoming.path(), "example");
        let old = roles.path().join("example/core_personality.txt");
        fs::write(&old, "previous personality").unwrap();
        let source = incoming.path().join("example");
        let source_files: Vec<_> = WalkDir::new(&source)
            .into_iter()
            .map(|e| e.unwrap())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.into_path())
            .collect();
        let mut interrupted = false;
        let result = import_role_pack(&RoleStorage::new(roles.path()), &source, true, |progress| {
            if !interrupted && progress.file_index == Some(1) {
                let current = source.join(progress.current_file.as_ref().unwrap());
                let remaining = source_files.iter().find(|path| **path != current).unwrap();
                fs::remove_file(remaining).unwrap();
                interrupted = true;
            }
        });
        assert!(interrupted);
        assert!(result.is_err());
        assert_eq!(fs::read_to_string(old).unwrap(), "previous personality");
        assert!(RoleStorage::new(roles.path()).load_role("example").is_ok());
    }
}
