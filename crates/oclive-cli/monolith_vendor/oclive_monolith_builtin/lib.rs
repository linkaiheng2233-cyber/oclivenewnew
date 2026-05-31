//! Placeholder providing a "real crate boundary" for the scaffold: one `invoke` per slot, for `process_message_monolith.rs` to statically link against.
//! When integrating with the production host, replace the dependency with the equivalent symbols from `oclive_*_builtin` or `oclive_kernel_runtime`.

/// Ensures this crate is linked (even when the current weld set is empty).
pub fn ensure_linked() {}

macro_rules! slot {
    ($m:ident, $label:literal) => {
        pub mod $m {
            pub fn invoke() {
                println!(concat!("oclive_monolith_builtin::", $label));
            }
        }
    };
}

slot!(memory, "memory");
slot!(emotion, "emotion");
slot!(event, "event");
slot!(prompt, "prompt");
slot!(llm, "llm");
slot!(agent, "agent");
slot!(complex_emotion, "complex_emotion");
