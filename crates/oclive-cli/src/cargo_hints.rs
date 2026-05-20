//! 解析 `cargo build` stderr，给出常见错误的修复建议。

/// 根据 cargo 编译失败输出返回人类可读建议（含未匹配时的兜底文案）。
pub fn suggest_cargo_build_failure(stderr: &str) -> String {
    let lower = stderr.to_ascii_lowercase();
    if lower.contains("could not find") && lower.contains("in the registry") {
        if let Some(pkg) = extract_crate_not_found(stderr) {
            return format!(
                "包 {pkg} 未找到。请检查拼写，或运行 cargo update 更新依赖索引。"
            );
        }
        return "依赖包未在 crates.io 索引中找到。请检查 Cargo.toml 中的 crate 名，或运行 cargo update。".into();
    }
    if lower.contains("linker") && (lower.contains("not found") || lower.contains("cannot find")) {
        return "缺少 C 编译器。Linux: apt install build-essential；macOS: xcode-select --install；Windows: 安装 Visual Studio Build Tools（含 MSVC 链接器）。".into();
    }
    if lower.contains("rustc") && lower.contains("is not supported") {
        return "Rust 版本过低或与依赖要求不匹配。运行 rustup update stable 升级到最新稳定版。".into();
    }
    if lower.contains("failed to run custom build command for `openssl-sys`")
        || lower.contains("failed to run custom build command for openssl-sys")
    {
        return "缺少 OpenSSL 开发库。Linux: apt install libssl-dev pkg-config；macOS: brew install openssl pkg-config。".into();
    }
    if lower.contains("memory allocation") && lower.contains("failed") {
        return "内存不足。请关闭其他应用释放内存，或减少并行编译：export CARGO_BUILD_JOBS=1（PowerShell: $env:CARGO_BUILD_JOBS=1）。".into();
    }
    format!(
        "未匹配到已知错误模式。请查看上方原始 cargo 输出；也可运行 cargo run -p oclive-cli -- doctor 检查环境。\n\n--- cargo stderr（节选）---\n{}",
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
        assert!(suggest_cargo_build_failure(s).contains("C 编译器"));
    }
}
