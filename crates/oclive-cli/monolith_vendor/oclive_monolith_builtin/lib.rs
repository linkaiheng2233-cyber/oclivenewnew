//! 脚手架用「真实 crate 边界」占位：每个槽一个 `invoke`，供 `process_message_monolith.rs` 静态链接。
//! 接入正式宿主时，将依赖替换为 `oclive_*_builtin` 或 `oclive_kernel_runtime` 中的等价符号。

/// 保证本 crate 被链接（即使当前焊接集合为空）。
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
