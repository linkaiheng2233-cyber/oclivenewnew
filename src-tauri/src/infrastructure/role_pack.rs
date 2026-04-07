//! `.ocpak` / `.zip`：ZIP 容器，内容为角色目录（与 `roles/{id}/` 一致）；亦可从**已解压目录**导入（布局相同）。

use crate::error::{AppError, Result};
use crate::infrastructure::storage::RoleStorage;
use crate::models::dto::ImportProgress;
use crate::models::Role;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;
use zip::ZipArchive;

use crate::models::role_manifest_disk::DiskRoleManifest;

fn safe_zip_path(name: &str) -> bool {
    !name.contains("..") && !name.starts_with('/') && !name.starts_with('\\')
}

/// ZIP 内 `manifest.json` 路径优先级：根目录优先，其次单段子目录，最后更深路径（与标准导出一致）。
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

/// 将 `roles/{role_id}/` 打成 `.ocpak`（ZIP）。
pub fn export_role_pack(storage: &RoleStorage, role_id: &str, dest: &Path) -> Result<()> {
    let src = storage.roles_dir().join(role_id);
    if !src.is_dir() {
        return Err(AppError::RoleNotFound(role_id.to_string()));
    }
    let file = File::create(dest)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
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

/// 从已解压目录读取 `manifest.json`（与 zip 解压后结构一致）。
fn peek_role_folder_manifest(dir: &Path) -> Result<(String, String, String)> {
    let root = resolve_extracted_role_root(dir)?;
    let manifest_path = root.join("manifest.json");
    if !manifest_path.is_file() {
        return Err(AppError::InvalidParameter(
            "角色包格式错误：未找到 manifest.json".into(),
        ));
    }
    let s = fs::read_to_string(&manifest_path).map_err(AppError::IoError)?;
    let disk: DiskRoleManifest = serde_json::from_str(&s)
        .map_err(|_| AppError::InvalidParameter("角色包格式错误：manifest.json 无法解析".into()))?;
    Ok((disk.id, disk.name, disk.version))
}

/// 从 `.ocpak` / `.zip` 或**已解压目录**读取 `manifest.json`，用于导入前预览与冲突判断。
pub fn peek_role_pack_manifest(src: &Path) -> Result<(String, String, String)> {
    if src.is_dir() {
        return peek_role_folder_manifest(src);
    }
    let file = File::open(src)
        .map_err(|e| AppError::InvalidParameter(format!("角色包格式错误：无法打开文件（{e}）")))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|_| AppError::InvalidParameter("角色包格式错误：不是有效的 ZIP/ocpak".into()))?;
    let mut candidates: Vec<(u8, usize)> = Vec::new();
    for i in 0..archive.len() {
        let f = archive
            .by_index(i)
            .map_err(|_| AppError::InvalidParameter("角色包格式错误：压缩包损坏".into()))?;
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
        let mut f = archive
            .by_index(i)
            .map_err(|_| AppError::InvalidParameter("角色包格式错误：压缩包损坏".into()))?;
        let mut s = String::new();
        std::io::Read::read_to_string(&mut f, &mut s)
            .map_err(|_| AppError::InvalidParameter("角色包格式错误：无法读取 manifest".into()))?;
        let disk: DiskRoleManifest = match serde_json::from_str(&s) {
            Ok(d) => d,
            Err(_) => continue,
        };
        return Ok((disk.id, disk.name, disk.version));
    }
    Err(AppError::InvalidParameter(
        "角色包格式错误：未找到 manifest.json".into(),
    ))
}

fn unzip_to(src: &Path, dest: &Path, mut on_entry: impl FnMut(usize, usize)) -> Result<()> {
    let file = File::open(src)
        .map_err(|e| AppError::InvalidParameter(format!("角色包格式错误：无法打开文件（{e}）")))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|_| AppError::InvalidParameter("角色包格式错误：不是有效的 ZIP/ocpak".into()))?;
    let total = archive.len().max(1);
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| AppError::Unknown(e.to_string()))?;
        let name = file.name().to_string();
        if !safe_zip_path(&name) {
            on_entry(i + 1, total);
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
        on_entry(i + 1, total);
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
        "未找到 manifest.json：须在包根目录或唯一子目录中包含该文件（与 zip 解压后结构一致）"
            .into(),
    ))
}

fn load_role_for_pack_import(storage: &RoleStorage, root: &Path) -> Result<Role> {
    storage.load_role_from_dir(root).map_err(|e| match e {
        AppError::SerializationError(_) | AppError::RoleNotFound(_) => {
            AppError::InvalidParameter("角色包格式错误：无法解析角色目录".into())
        }
        o => o,
    })
}

/// 将已解析的 `root`（含 `manifest.json`）安装到 `roles/{id}/`。
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
    copy_role_tree(root, &dest, |cur, tot| {
        let pct = copy_percent(cur, tot).min(99);
        on_progress(ImportProgress {
            percent: pct,
            message: format!("正在写入文件 {}/{}", cur, tot),
        });
    })?;
    on_progress(ImportProgress {
        percent: 100,
        message: "导入完成".into(),
    });
    Ok(id)
}

fn copy_role_tree(src: &Path, dest: &Path, mut on_file: impl FnMut(usize, usize)) -> Result<()> {
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
        on_file(i + 1, total);
    }
    Ok(())
}

/// 从已解压目录复制到 `roles/{id}/`（结构与 zip 解压一致）。
fn import_role_from_directory<F: FnMut(ImportProgress)>(
    storage: &RoleStorage,
    src: &Path,
    overwrite: bool,
    mut on_progress: F,
) -> Result<String> {
    on_progress(ImportProgress {
        percent: 0,
        message: "准备读取文件夹…".into(),
    });
    let root = resolve_extracted_role_root(src)?;
    install_role_from_resolved_root(storage, &root, overwrite, on_progress, |cur, tot| {
        ((cur as i64 * 100) / tot as i64).min(99) as i32
    })
}

/// 解压 `.ocpak` / `.zip` 到 `roles/{id}/`，或从**已解压目录**复制（与 `roles/{id}/` 布局一致）。
/// 若目录已存在且 `overwrite == false` 则返回 [`AppError::RolePackExists`]。
/// `on_progress` 在解压与复制阶段多次调用，结束时由调用方再发 100%。
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
        message: "准备解压…".into(),
    });
    let td = tempfile::tempdir()?;
    unzip_to(src, td.path(), |cur, tot| {
        let pct = ((cur as i64 * 50) / tot as i64).min(50) as i32;
        on_progress(ImportProgress {
            percent: pct,
            message: format!("正在解压 {}/{}", cur, tot),
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
        let opts = FileOptions::default().compression_method(CompressionMethod::Stored);
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
