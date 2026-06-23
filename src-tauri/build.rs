fn main() {
    // Debug builds with large async invoke chains can overflow the default 1 MiB main stack on Windows.
    #[cfg(all(windows, debug_assertions))]
    {
        println!("cargo:rustc-link-arg=/STACK:4194304");
    }
    println!("cargo:rerun-if-changed=../crates/oclive_kernel_host/migrations");
    tauri_build::build()
}
