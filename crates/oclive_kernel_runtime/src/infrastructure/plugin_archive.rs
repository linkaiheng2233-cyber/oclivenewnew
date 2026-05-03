//! 插件归档（`.oclive-plugin` / zip 字节流或文件）：安全解压与 manifest id 预览（宿主无关）。

use crate::error::{AppError, Result};
use crate::infrastructure::directory_plugins::OclivePluginManifest;
use std::fs;
use std::io::{Read, Seek, Write};
use std::path::Path;
use zip::ZipArchive;

pub const MAX_PLUGIN_ARCHIVE_FILES: usize = 2000;
pub const MAX_PLUGIN_ARCHIVE_TOTAL_BYTES: u64 = 50 * 1024 * 1024;
pub const MAX_PLUGIN_ARCHIVE_SINGLE_FILE_BYTES: u64 = 10 * 1024 * 1024;

/// 将插件 zip 解压到 `dst_dir`（路径遍历防护、条目数与总大小上限与桌面侧历史行为一致）。
pub fn extract_oclive_plugin_archive_reader<R: Read + Seek>(
    mut zip: ZipArchive<R>,
    dst_dir: &Path,
) -> Result<()> {
    let mut files = 0usize;
    let mut total: u64 = 0;
    for i in 0..zip.len() {
        let mut f = zip
            .by_index(i)
            .map_err(|e| AppError::Unknown(format!("read zip entry failed: {}", e)))?;
        let rel = f.enclosed_name().ok_or_else(|| {
            AppError::InvalidParameter(
                "[PLUGIN_ARCHIVE_ILLEGAL_PATH] zip entry path is not enclosed".into(),
            )
        })?;
        let name = rel.to_string_lossy().replace('\\', "/");
        if f.is_dir() {
            let out_path = dst_dir.join(rel);
            fs::create_dir_all(&out_path)?;
            continue;
        }
        files += 1;
        if files > MAX_PLUGIN_ARCHIVE_FILES {
            return Err(AppError::InvalidParameter(format!(
                "[PLUGIN_ARCHIVE_TOO_MANY_FILES] too many files (>{})",
                MAX_PLUGIN_ARCHIVE_FILES
            )));
        }
        let sz = f.size();
        if sz > MAX_PLUGIN_ARCHIVE_SINGLE_FILE_BYTES {
            return Err(AppError::InvalidParameter(format!(
                "[PLUGIN_ARCHIVE_SINGLE_FILE_TOO_LARGE] file too large {} bytes: {}",
                sz, name
            )));
        }
        total = total.saturating_add(sz);
        if total > MAX_PLUGIN_ARCHIVE_TOTAL_BYTES {
            return Err(AppError::InvalidParameter(format!(
                "[PLUGIN_ARCHIVE_TOTAL_TOO_LARGE] total too large (>{} bytes)",
                MAX_PLUGIN_ARCHIVE_TOTAL_BYTES
            )));
        }
        let out_path = dst_dir.join(rel);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = fs::File::create(&out_path)?;
        std::io::copy(&mut f, &mut out).map_err(AppError::IoError)?;
    }
    Ok(())
}

pub fn extract_oclive_plugin_archive(bytes: &[u8], dst_dir: &Path) -> Result<()> {
    let zip = ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|e| AppError::Unknown(format!("open plugin archive failed: {}", e)))?;
    extract_oclive_plugin_archive_reader(zip, dst_dir)
}

pub fn extract_oclive_plugin_archive_file(zip_path: &Path, dst_dir: &Path) -> Result<()> {
    let file = fs::File::open(zip_path).map_err(AppError::IoError)?;
    let zip = ZipArchive::new(file)
        .map_err(|e| AppError::Unknown(format!("open plugin archive failed: {}", e)))?;
    extract_oclive_plugin_archive_reader(zip, dst_dir)
}

/// 解压到系统临时目录并读取 `manifest.id`（用于安装前预览；不含进程内缓存路径偏好）。
pub fn peek_plugin_id_from_archive_bytes(bytes: &[u8]) -> Result<String> {
    let tmp = tempfile::tempdir().map_err(AppError::IoError)?;
    extract_oclive_plugin_archive(bytes, tmp.path())?;
    let manifest = OclivePluginManifest::load_from_dir(tmp.path())
        .map_err(|e| AppError::Unknown(format!("manifest validation failed: {}", e)))?;
    let pid = manifest.id.trim().to_string();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("manifest.id required".into()));
    }
    Ok(pid)
}

/// Pack a plugin root directory into a `.oclive-plugin` zip at `archive_path` (deflate, unix 0644).
/// Returns the SHA-256 hex digest of the written archive (same contract as desktop `pack_plugin`).
#[cfg(feature = "role-pack-zip")]
pub fn pack_plugin_directory_to_zip_deflated(
    plugin_root: &Path,
    archive_path: &Path,
) -> Result<String> {
    use sha2::{Digest, Sha256};
    use walkdir::WalkDir;
    use zip::write::FileOptions;
    use zip::{CompressionMethod, ZipWriter};

    let f = fs::File::create(archive_path).map_err(AppError::IoError)?;
    let mut zip = ZipWriter::new(f);
    let opt = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);
    for entry in WalkDir::new(plugin_root).into_iter().flatten() {
        let p = entry.path();
        if p.is_dir() {
            continue;
        }
        let rel = match p.strip_prefix(plugin_root) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let name = rel.to_string_lossy().replace('\\', "/");
        zip.start_file(name, opt)
            .map_err(|e| AppError::Unknown(format!("zip start file failed: {}", e)))?;
        let bytes = fs::read(p).map_err(AppError::IoError)?;
        zip.write_all(&bytes)
            .map_err(|e| AppError::Unknown(format!("zip write failed: {}", e)))?;
    }
    zip.finish()
        .map_err(|e| AppError::Unknown(format!("zip finalize failed: {}", e)))?;

    let blob = fs::read(archive_path).map_err(AppError::IoError)?;
    let mut hasher = Sha256::new();
    hasher.update(&blob);
    let digest = hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    Ok(digest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::{FileOptions, ZipWriter};

    #[test]
    fn reject_non_zip_bytes() {
        let r = extract_oclive_plugin_archive(b"not a zip", std::path::Path::new("."));
        assert!(r.is_err());
    }

    #[test]
    fn peek_id_from_minimal_archive() {
        let mut buf = Vec::new();
        {
            let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buf));
            let opts = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
            zip.start_file("manifest.json", opts).unwrap();
            let body = r#"{"schema_version":1,"id":"com.test.plugin","version":"1.0.0","process":{"command":"x"}}"#;
            zip.write_all(body.as_bytes()).unwrap();
            zip.finish().unwrap();
        }
        let id = peek_plugin_id_from_archive_bytes(&buf).unwrap();
        assert_eq!(id, "com.test.plugin");
    }

    #[test]
    fn pack_dir_round_trip_peek_id() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("plugin");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("manifest.json"),
            r#"{"schema_version":1,"id":"com.pack.rt","version":"1.0.0","process":{"command":"x"}}"#,
        )
        .unwrap();
        let out_zip = tmp.path().join("out.oclive-plugin");
        let hex = pack_plugin_directory_to_zip_deflated(&root, &out_zip).unwrap();
        assert_eq!(hex.len(), 64);
        let bytes = fs::read(&out_zip).unwrap();
        let id = peek_plugin_id_from_archive_bytes(&bytes).unwrap();
        assert_eq!(id, "com.pack.rt");
    }
}
