//! 插件解压/暂存后的目录布局：定位 `manifest.json` 根、递归复制插件树（宿主无关）。

use crate::error::{AppError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// 在解压目录中定位插件根：当前目录含 `manifest.json`，或**唯一**一层子目录内含。
pub fn find_plugin_manifest_root(dir: &Path) -> Result<PathBuf> {
    let direct = dir.join("manifest.json");
    if direct.is_file() {
        return Ok(dir.to_path_buf());
    }
    let subs: Vec<_> = fs::read_dir(dir)
        .map_err(AppError::IoError)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    if subs.len() == 1 {
        let p = subs[0].path();
        if p.join("manifest.json").is_file() {
            return Ok(p);
        }
    }
    Err(AppError::InvalidParameter(
        "zip 中未找到有效的 manifest.json（根目录或单一顶层目录内）".into(),
    ))
}

/// 将 `src` 下文件与目录递归复制到 `dst`（保留相对路径）。
pub fn copy_plugin_tree(src: &Path, dst: &Path) -> Result<()> {
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let rel = entry.path().strip_prefix(src).map_err(|e| {
            AppError::InvalidParameter(format!("copy_plugin_tree strip_prefix: {}", e))
        })?;
        let out = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&out).map_err(AppError::IoError)?;
        } else {
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent).map_err(AppError::IoError)?;
            }
            fs::copy(entry.path(), &out).map_err(AppError::IoError)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_root_flat() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("manifest.json"), "{}").unwrap();
        let r = find_plugin_manifest_root(tmp.path()).unwrap();
        assert_eq!(r, tmp.path());
    }

    #[test]
    fn find_root_nested_single() {
        let tmp = tempfile::tempdir().unwrap();
        let inner = tmp.path().join("pkg");
        fs::create_dir_all(&inner).unwrap();
        fs::write(inner.join("manifest.json"), "{}").unwrap();
        let r = find_plugin_manifest_root(tmp.path()).unwrap();
        assert_eq!(r, inner);
    }

    #[test]
    fn find_root_rejects_ambiguous() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("a")).unwrap();
        fs::create_dir_all(tmp.path().join("b")).unwrap();
        assert!(find_plugin_manifest_root(tmp.path()).is_err());
    }

    #[test]
    fn copy_tree_roundtrip() {
        let src = tempfile::tempdir().unwrap();
        fs::create_dir_all(src.path().join("sub")).unwrap();
        fs::write(src.path().join("sub").join("f.txt"), b"x").unwrap();
        let dst = tempfile::tempdir().unwrap();
        copy_plugin_tree(src.path(), dst.path()).unwrap();
        assert!(dst.path().join("sub").join("f.txt").is_file());
        assert_eq!(
            fs::read_to_string(dst.path().join("sub").join("f.txt")).unwrap(),
            "x"
        );
    }
}
