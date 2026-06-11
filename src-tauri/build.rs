fn main() {
    println!("cargo:rerun-if-changed=../crates/oclive_kernel_host/migrations");
    println!("cargo:rerun-if-env-changed=OCLIVE_TAURI_SHELL");
    if std::env::var("OCLIVE_TAURI_SHELL")
        .map(|v| v.trim().eq_ignore_ascii_case("theater"))
        .unwrap_or(false)
    {
        println!("cargo:rustc-env=OCLIVE_BUNDLED_SHELL=theater");
    }
    tauri_build::build()
}
