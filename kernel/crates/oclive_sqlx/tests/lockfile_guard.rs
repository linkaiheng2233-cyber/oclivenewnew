//! Guard: oclive_sqlx dependency tree must not pull sqlx-mysql / rsa.

use std::process::Command;

#[test]
fn cargo_tree_excludes_mysql_and_rsa() {
    let out = Command::new("cargo")
        .args(["tree", "-p", "oclive_sqlx", "--format", "{p}"])
        .output()
        .expect("cargo tree");
    assert!(
        out.status.success(),
        "cargo tree failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        !text.contains("sqlx-mysql"),
        "oclive_sqlx tree must not include sqlx-mysql:\n{text}"
    );
    assert!(
        !text.contains(" rsa "),
        "oclive_sqlx tree must not include rsa crate:\n{text}"
    );
}
