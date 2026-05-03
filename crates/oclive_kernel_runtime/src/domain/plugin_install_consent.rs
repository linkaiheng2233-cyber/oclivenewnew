//! 安装流程中的权限 consent 纯函数（declared vs accepted）。

use crate::error::{AppError, Result};
use std::collections::HashSet;

#[must_use]
pub fn normalize_install_permission_tokens(mut tokens: Vec<String>) -> Vec<String> {
    tokens.retain(|s| !s.trim().is_empty());
    for t in tokens.iter_mut() {
        *t = t.trim().to_string();
    }
    tokens.sort();
    tokens.dedup();
    tokens
}

/// `accepted` 非空时，每一项必须在 `declared`（trim 后）中出现。
pub fn ensure_accepted_permissions_subset_declared(
    declared: &[String],
    accepted: &[String],
) -> Result<()> {
    if accepted.is_empty() {
        return Ok(());
    }
    let declared_set: HashSet<String> = declared.iter().map(|s| s.trim().to_string()).collect();
    let ok = accepted.iter().all(|p| declared_set.contains(p.trim()));
    if !ok {
        return Err(AppError::InvalidParameter(
            "accepted_permissions must be a subset of declared permissions".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subset_ok() {
        let d = vec!["a".into(), "b".into()];
        let a = vec!["b".into()];
        ensure_accepted_permissions_subset_declared(&d, &a).unwrap();
    }

    #[test]
    fn subset_rejects_extra() {
        let d = vec!["a".into()];
        let a = vec!["z".into()];
        assert!(ensure_accepted_permissions_subset_declared(&d, &a).is_err());
    }

    #[test]
    fn empty_accepted_always_ok() {
        ensure_accepted_permissions_subset_declared(&[], &[]).unwrap();
        ensure_accepted_permissions_subset_declared(&["x".into()], &[]).unwrap();
    }
}
