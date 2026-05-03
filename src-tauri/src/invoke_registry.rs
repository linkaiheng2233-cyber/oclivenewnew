//! Tauri `invoke` paths are assembled at **build time** from `invoke_lists/*.txt`
//! (see `build.rs` → `OUT_DIR/oclive_invoke_handler_gen.rs`).

include!(concat!(env!("OUT_DIR"), "/oclive_invoke_handler_gen.rs"));
