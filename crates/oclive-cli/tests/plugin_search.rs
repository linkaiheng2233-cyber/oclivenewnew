//! `oclive plugin search --provides`

use std::fs;
use std::process::Command;

#[test]
fn plugin_search_filters_provides() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let pdir = tmp.path().join("plugins");
    fs::create_dir_all(pdir.join("com.test.llm")).expect("mkdir");
    fs::write(
        pdir.join("com.test.llm/manifest.json"),
        r#"{"id":"com.test.llm","version":"1.0.0","provides":["llm"]}"#,
    )
    .expect("write");
    fs::create_dir_all(pdir.join("com.test.shell")).expect("mkdir");
    fs::write(
        pdir.join("com.test.shell/manifest.json"),
        r#"{"id":"com.test.shell","version":"0.1.0"}"#,
    )
    .expect("write");
    let bin = env!("CARGO_BIN_EXE_oclive-cli");
    let out = Command::new(bin)
        .args([
            "plugin",
            "search",
            "--provides",
            "llm",
            "-o",
            pdir.to_str().expect("path"),
            "--json",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success(), "stderr={}", String::from_utf8_lossy(&out.stderr));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("com.test.llm"));
    assert!(!text.contains("com.test.shell"));
}
