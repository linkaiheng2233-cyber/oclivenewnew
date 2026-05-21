fn main() {
    // sqlx::migrate! embeds ./migrations at compile time; rebuild when SQL changes.
    println!("cargo:rerun-if-changed=migrations");
    tauri_build::build()
}
