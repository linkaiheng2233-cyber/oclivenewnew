//! `.ocpak` / `.zip` 角色包；含市场直链下载安装。

use crate::error::{AppError, Result};
use crate::infrastructure::storage::RoleStorage;
use crate::models::dto::ImportProgress;
use crate::models::DiskRoleManifest;
use crate::models::Role;
use crate::utils::digest::sha256_hex;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use walkdir::WalkDir;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;
use zip::ZipArchive;

const MAX_ROLE_PACK_DOWNLOAD_BYTES: u64 = 80 * 1024 * 1024;

fn safe_zip_path(name: &str) -> bool {
    !name.contains("..") && !name.starts_with('/') && !name.starts_with('\\')
}

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

fn export_role_pack_disk(storage: &RoleStorage, role_id: &str, dest: &Path) -> Result<()> {
    let src = storage.roles_dir().join(role_id);
    if !src.is_dir() {
        return Err(AppError::RoleNotFound(role_id.to_string()));
    }
    let file = File::create(dest)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
    for entry in WalkDir::new(&src).min_depth(1) {
        let entry = entry.map_err(|e| {
            AppError::InvalidParameter(format!("[ROLE_PACK_ZIP_EXPORT] walkdir: {}", e))
        })?;
        let path = entry.path();
        if path.is_file() {
            let rel = path
                .strip_prefix(&src)
                .map_err(|_| AppError::InvalidParameter("zip strip".into()))?;
            let name = rel.to_string_lossy().replace('\\', "/");
            if !safe_zip_path(&name) {
                continue;
            }
            zip.start_file(name, options).map_err(|e| {
                AppError::InvalidParameter(format!("[ROLE_PACK_ZIP_EXPORT] start_file: {}", e))
            })?;
            let mut f = File::open(path)?;
            std::io::copy(&mut f, &mut zip).map_err(AppError::IoError)?;
        }
    }
    zip.finish()
        .map_err(|e| AppError::InvalidParameter(format!("[ROLE_PACK_ZIP_EXPORT] finish: {}", e)))?;
    Ok(())
}

/// 导出角色包为 ZIP/ocpak（磁盘密集逻辑在 `spawn_blocking` 中执行）。
pub async fn export_role_pack(storage: &RoleStorage, role_id: &str, dest: &Path) -> Result<()> {
    let storage = storage.clone();
    let role_id = role_id.to_string();
    let dest = dest.to_path_buf();
    tokio::task::spawn_blocking(move || export_role_pack_disk(&storage, &role_id, &dest))
        .await
        .map_err(|e| {
            AppError::InvalidParameter(format!("[ROLE_PACK_EXPORT_JOIN] {}", e))
        })?
}

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

fn peek_role_pack_manifest_disk(src: &Path) -> Result<(String, String, String)> {
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
        Read::read_to_string(&mut f, &mut s)
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

/// 预览 ZIP 或已解压目录中的 `manifest.json`（在阻塞线程池读盘）。
pub async fn peek_role_pack_manifest(src: PathBuf) -> Result<(String, String, String)> {
    tokio::task::spawn_blocking(move || peek_role_pack_manifest_disk(&src))
        .await
        .map_err(|e| {
            AppError::InvalidParameter(format!("[ROLE_PACK_PEEK_JOIN] {}", e))
        })?
}

fn unzip_to(src: &Path, dest: &Path, mut on_entry: impl FnMut(usize, usize)) -> Result<()> {
    let file = File::open(src)
        .map_err(|e| AppError::InvalidParameter(format!("角色包格式错误：无法打开文件（{e}）")))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|_| AppError::InvalidParameter("角色包格式错误：不是有效的 ZIP/ocpak".into()))?;
    let total = archive.len().max(1);
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|_| {
            AppError::InvalidParameter("[ROLE_PACK_ZIP] archive entry read failed".into())
        })?;
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

fn import_role_pack_disk<F: FnMut(ImportProgress)>(
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

/// 从 ZIP/ocpak 或已解压目录导入角色（解压与复制在阻塞线程池执行）。
pub async fn import_role_pack<F>(
    storage: &RoleStorage,
    src: &Path,
    overwrite: bool,
    on_progress: F,
) -> Result<String>
where
    F: FnMut(ImportProgress) + Send + 'static,
{
    let storage = storage.clone();
    let src = src.to_path_buf();
    tokio::task::spawn_blocking(move || {
        import_role_pack_disk(&storage, &src, overwrite, on_progress)
    })
    .await
    .map_err(|e| AppError::InvalidParameter(format!("[ROLE_PACK_IMPORT_JOIN] {}", e)))?
}

fn eq_hex_sha256(got: &str, expected: &str) -> bool {
    got.trim().eq_ignore_ascii_case(expected.trim())
}

pub async fn install_role_pack_from_direct_url<F>(
    storage: &RoleStorage,
    app_data_dir: &Path,
    role_id: &str,
    url: &str,
    expected_sha256: &str,
    overwrite: bool,
    mut on_progress: F,
) -> Result<String>
where
    F: FnMut(ImportProgress) + Send + 'static,
{
    let rid = role_id.trim();
    if rid.is_empty() {
        return Err(AppError::InvalidParameter("role_id 不能为空".into()));
    }
    let u = url.trim();
    if u.is_empty() {
        return Err(AppError::InvalidParameter("download url 不能为空".into()));
    }
    let exp = expected_sha256.trim();
    if exp.len() != 64 {
        return Err(AppError::InvalidParameter(
            "sha256 必须为 64 位十六进制".into(),
        ));
    }

    on_progress(ImportProgress {
        percent: 2,
        message: "正在下载角色包…".into(),
    });

    let u = u.to_string();
    let cli = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| {
            AppError::InvalidParameter(format!("[ROLE_PACK_DOWNLOAD] http client: {}", e))
        })?;
    let resp = cli
        .get(&u)
        .send()
        .await
        .map_err(|e| AppError::InvalidParameter(format!("[ROLE_PACK_DOWNLOAD] get: {}", e)))?;
    if !resp.status().is_success() {
        return Err(AppError::InvalidParameter(format!(
            "[ROLE_PACK_DOWNLOAD] status={} url={}",
            resp.status(),
            u
        )));
    }
    let body = resp
        .bytes()
        .await
        .map_err(|e| AppError::InvalidParameter(format!("[ROLE_PACK_DOWNLOAD] read body: {}", e)))?;
    if body.len() as u64 > MAX_ROLE_PACK_DOWNLOAD_BYTES {
        return Err(AppError::InvalidParameter(format!(
            "[ROLE_PACK_TOO_LARGE] role pack too large (>{} bytes)",
            MAX_ROLE_PACK_DOWNLOAD_BYTES
        )));
    }
    let bytes = body.to_vec();

    on_progress(ImportProgress {
        percent: 20,
        message: "正在校验 SHA-256…".into(),
    });
    let got = sha256_hex(&bytes);
    if !eq_hex_sha256(&got, exp) {
        return Err(AppError::InvalidParameter(format!(
            "[ROLE_PACK_SHA256_MISMATCH] sha256 mismatch expected={} got={}",
            exp, got
        )));
    }

    on_progress(ImportProgress {
        percent: 30,
        message: "正在写入临时文件并解压导入…".into(),
    });

    let storage = storage.clone();
    let app_data_dir = app_data_dir.to_path_buf();
    let rid = rid.to_string();
    tokio::task::spawn_blocking(move || {
        let tmp_root = app_data_dir.join("tmp");
        let _ = fs::create_dir_all(&tmp_root);
        let td = TempDir::new_in(&tmp_root).map_err(AppError::IoError)?;
        let tmp = td.path().join(format!("{}.ocpak", rid.as_str()));
        let mut f = File::create(&tmp).map_err(AppError::IoError)?;
        f.write_all(&bytes).map_err(AppError::IoError)?;
        f.flush().ok();

        let path = tmp.clone();
        import_role_pack_disk(&storage, &path, overwrite, |p| {
            let pct = 35 + ((p.percent.clamp(0, 100) * 65) / 100);
            on_progress(ImportProgress {
                percent: pct,
                message: p.message,
            });
        })
    })
    .await
    .map_err(|e| AppError::InvalidParameter(format!("[ROLE_PACK_INSTALL_JOIN] {}", e)))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn export_import_roundtrip() {
        let roles_src = tempfile::tempdir().unwrap();
        let roles_dst = tempfile::tempdir().unwrap();
        fs::create_dir_all(roles_src.path().join("mumu").join("scenes").join("default")).unwrap();
        fs::write(
            roles_src.path().join("mumu").join("manifest.json"),
            r#"{"id":"mumu","name":"M","version":"1","author":"t","description":"d","default_personality":[0.5,0.5,0.5,0.5,0.5,0.5,0.5],"evolution":{},"user_relations":{"friend":{"prompt_hint":"x"}},"default_relation":"friend","memory_config":{"scene_weight_multiplier":1.0,"topic_weights":{}}}"#,
        )
        .unwrap();

        let st = RoleStorage::new(roles_src.path());
        let out_tmp = tempfile::tempdir().unwrap();
        let pak = out_tmp.path().join("x.ocpak");
        export_role_pack(&st, "mumu", &pak).await.unwrap();

        let st2 = RoleStorage::new(roles_dst.path());
        let id = import_role_pack(&st2, &pak, true, |_| {}).await.unwrap();
        assert_eq!(id, "mumu");
        let role = st2.load_role("mumu").unwrap();
        assert_eq!(role.id, "mumu");
    }

    #[tokio::test]
    async fn import_from_unpacked_folder_matches_zip() {
        let roles_src = tempfile::tempdir().unwrap();
        let roles_dst = tempfile::tempdir().unwrap();
        fs::create_dir_all(roles_src.path().join("mumu").join("scenes").join("default")).unwrap();
        fs::write(
            roles_src.path().join("mumu").join("manifest.json"),
            r#"{"id":"mumu","name":"M","version":"1","author":"t","description":"d","default_personality":[0.5,0.5,0.5,0.5,0.5,0.5,0.5],"evolution":{},"user_relations":{"friend":{"prompt_hint":"x"}},"default_relation":"friend","memory_config":{"scene_weight_multiplier":1.0,"topic_weights":{}}}"#,
        )
        .unwrap();

        let st = RoleStorage::new(roles_dst.path());
        let id = import_role_pack(
            &st,
            roles_src.path().join("mumu").as_path(),
            true,
            |_| {},
        )
        .await
        .unwrap();
        assert_eq!(id, "mumu");
        assert!(st.load_role("mumu").is_ok());
    }

    #[tokio::test]
    async fn peek_zip_prefers_root_manifest_over_deeper_path() {
        let dir = tempfile::tempdir().unwrap();
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

        let (id, name, ver) = peek_role_pack_manifest(pak.clone()).await.unwrap();
        assert_eq!(id, "right");
        assert_eq!(name, "R");
        assert_eq!(ver, "2");
    }
}
