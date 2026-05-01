//! Manifest `version` 字段 semver 解析（与 `dependency` 报告共用）。

use semver::Version;

/// 将 manifest `version` 解析为 `semver::Version`（兼容 `x.y` → `x.y.0`）。
pub fn parse_manifest_version(s: &str) -> Option<Version> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    Version::parse(t).ok().or_else(|| {
        if t.matches('.').count() == 1 {
            Version::parse(&format!("{t}.0")).ok()
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::parse_manifest_version;

    #[test]
    fn parses_release_triple() {
        assert!(parse_manifest_version("1.2.3").is_some());
    }

    #[test]
    fn parses_two_part() {
        let v = parse_manifest_version("1.0").expect("1.0");
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_manifest_version("abc").is_none());
    }
}
