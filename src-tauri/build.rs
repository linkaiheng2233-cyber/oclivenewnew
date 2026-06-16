fn main() {
    println!("cargo:rerun-if-changed=../crates/oclive_kernel_host/migrations");
    tauri_build::build()
}
