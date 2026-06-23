//! Parse `cargo build` stderr and suggest fixes.

/// Human-readable hint for a failed cargo build (English).
pub fn suggest_cargo_build_failure(stderr: &str) -> String {
    let lower = stderr.to_ascii_lowercase();
    if lower.contains("could not find") && lower.contains("in the registry") {
        if let Some(pkg) = extract_crate_not_found(stderr) {
            return format!(
                "Crate {pkg} not found. Check the name or run cargo update to refresh the index."
            );
        }
        return "Dependency not found on crates.io. Check Cargo.toml or run cargo update.".into();
    }
    if lower.contains("linker") && (lower.contains("not found") || lower.contains("cannot find")) {
        return "C/C++ linker missing. Linux: apt install build-essential; macOS: xcode-select --install; Windows: Visual Studio Build Tools (MSVC).".into();
    }
    if lower.contains("rustc") && lower.contains("is not supported") {
        return "Rust version too old for a dependency. Run: rustup update stable".into();
    }
    if lower.contains("failed to run custom build command for `openssl-sys`")
        || lower.contains("failed to run custom build command for openssl-sys")
    {
        return "OpenSSL dev libraries missing. Linux: apt install libssl-dev pkg-config; macOS: brew install openssl pkg-config.".into();
    }
    if lower.contains("memory allocation") && lower.contains("failed") {
        return "Out of memory. Close other apps or set CARGO_BUILD_JOBS=1.".into();
    }
    format!(
        "No known pattern matched. See cargo output above; run: oclive doctor\n\n--- cargo stderr (excerpt) ---\n{}",
        stderr.chars().take(2000).collect::<String>()
    )
}

fn extract_crate_not_found(stderr: &str) -> Option<String> {
    for line in stderr.lines() {
        if line.contains("could not find") && line.contains("in the registry") {
            if let Some(start) = line.find('`') {
                let rest = &line[start + 1..];
                if let Some(end) = rest.find('`') {
                    return Some(rest[..end].to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_missing_crate() {
        let s = "error: failed to select a version for `foo-bar`.\ncould not find `foo-bar` in the registry";
        assert!(suggest_cargo_build_failure(s).contains("foo-bar"));
    }

    #[test]
    fn detects_linker() {
        let s = "error: linker `cc` not found";
        assert!(suggest_cargo_build_failure(s).contains("linker"));
    }
}
