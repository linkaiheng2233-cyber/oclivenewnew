fn main() {
    // Make `tauri-build` optional so the kernel server can reuse the runtime
    // without pulling in Tauri.
    //
    // When the `tauri-app` feature is enabled, Cargo sets `CARGO_FEATURE_TAURI_APP=1`.
    #[cfg(feature = "tauri-app")]
    {
        if std::env::var("CARGO_FEATURE_TAURI_APP").is_ok() {
            tauri_build::build()
        }
    }

    #[cfg(not(feature = "tauri-app"))]
    {
        // No-op build script for non-Tauri builds.
    }
}
