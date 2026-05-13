//! 插件根目录下文本资产的安全读取（canonicalize + 根前缀校验）。

use super::normalize_plugin_rel;
use crate::error::AppError;
use std::path::Path;

pub fn read_plugin_asset_text_under_root(root: &Path, rel: &str) -> Result<String, AppError> {
    let rel = normalize_plugin_rel(rel.trim());
    if rel.is_empty() {
        return Err(AppError::InvalidParameter("rel required".into()));
    }
    if rel.split('/').any(|p| p == "..") {
        return Err(AppError::InvalidParameter("invalid rel path".into()));
    }
    let path = root.join(&rel);
    let root_canon = root
        .canonicalize()
        .map_err(|e| AppError::InvalidParameter(format!("plugin root: {}", e)))?;
    let path_canon = path.canonicalize().map_err(AppError::IoError)?;
    if !path_canon.starts_with(&root_canon) {
        return Err(AppError::PermissionDenied(
            "path escapes plugin directory".into(),
        ));
    }
    std::fs::read_to_string(&path_canon).map_err(AppError::IoError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn rejects_dotdot_rel() {
        let tmp = tempfile::tempdir().expect("tmp");
        let root = tmp.path();
        let err = read_plugin_asset_text_under_root(root, "../outside.txt").expect_err("dotdot");
        assert!(matches!(err, AppError::InvalidParameter(_)));
    }

    #[test]
    fn reads_under_root() {
        let tmp = tempfile::tempdir().expect("tmp");
        let root = tmp.path();
        let mut f = std::fs::File::create(root.join("hi.txt")).expect("create");
        f.write_all(b"ok").expect("write");
        let s = read_plugin_asset_text_under_root(root, "hi.txt").expect("read");
        assert_eq!(s, "ok");
    }
}
