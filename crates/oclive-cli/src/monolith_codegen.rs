//! 高耦合（Monolith）第一阶段：由 `monolith.toml` 生成可编译占位实现。
//!
//! 接入真实内核后，将 `welded_*` 内联模块替换为对 `oclive_*_builtin` 等 crate 的静态调用。

/// 与 RFC §4 一致的第一阶段模板：`weld_modules = []` 表示全部七槽焊接占位实现。
pub fn render_monolith_toml_phase_one() -> String {
    r#"# 高耦合编译配置
# 由 oclive init 生成，oclive build 在编译时读取
# 不参与运行时，可安全删除以恢复标准模式

[monolith]
# 是否启用高耦合编译
enabled = true
# 参与焊接的模块列表（空数组 = 全部焊接）
weld_modules = []
# 排除的模块列表（weld_modules 为空时，从全部模块中排除指定模块；第一阶段通常为空）
exclude = []
"#
    .to_string()
}

/// 生成 `src/process_message_monolith.rs`：七槽均为同 crate 内静态占位，保证 `cargo build --features monolith` 可通过。
pub fn generate_monolith_source() -> String {
    r#"#![allow(dead_code)]
// 此文件由 oclive-cli 根据 monolith.toml 生成。
// 请勿手改焊接逻辑；修改 `monolith.toml` 后请重新运行 `oclive init` 或后续 `oclive build` 代码生成步骤。
// 第一阶段为占位：真实宿主应将 welded_* 替换为对 oclive_*_builtin 等 crate 的静态调用。

/// Monolith 入口：演示七槽静态调用链（占位，无 I/O）。
pub fn run_monolith_pipeline_demo() {
    welded_memory::step();
    welded_emotion::step();
    welded_event::step();
    welded_prompt::step();
    welded_llm::step();
    welded_agent::step();
    welded_complex_emotion::step();
    println!("monolith: welded pipeline (phase-1 stub) completed");
}

macro_rules! welded_slot {
    ($m:ident, $label:expr) => {
        mod $m {
            pub fn step() {
                println!(concat!("welded::", $label, " (static stub)"));
            }
        }
    };
}

welded_slot!(welded_memory, "memory");
welded_slot!(welded_emotion, "emotion");
welded_slot!(welded_event, "event");
welded_slot!(welded_prompt, "prompt");
welded_slot!(welded_llm, "llm");
welded_slot!(welded_agent, "agent");
welded_slot!(welded_complex_emotion, "complex_emotion");
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monolith_toml_contains_enabled_empty_arrays() {
        let s = render_monolith_toml_phase_one();
        assert!(s.contains("enabled = true"));
        assert!(s.contains("weld_modules = []"));
        assert!(s.contains("exclude = []"));
    }

    #[test]
    fn monolith_source_contains_run_and_macros() {
        let s = generate_monolith_source();
        assert!(s.contains("run_monolith_pipeline_demo"));
        assert!(s.contains("welded_memory"));
    }
}
