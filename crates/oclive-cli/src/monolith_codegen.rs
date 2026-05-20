//! 高耦合（Monolith）：由 `monolith.toml` 焊接计划生成 `process_message_monolith.rs`。
//!
//! 已焊接槽静态调用 `oclive_monolith_builtin`（脚手架 vendor crate，可替换为真实 `oclive_*_builtin`）；
//! 未焊接槽保留 `PluginHost`/trait 风格占位调用链。

use crate::monolith_config::{WeldPlan, SLOT_IDS};
use anyhow::Context;
use std::fs;
use std::path::Path;

/// 与 RFC 及校验器一致的第一阶段默认模板（`weld_modules` / `exclude` 互斥说明）。
pub fn render_monolith_toml_phase_one() -> String {
    render_monolith_toml_default()
}

/// 按性能档位预填 `weld_modules`（`exclude` 为空；与 [`crate::init::MonolithPresetArg`] 对应）。
pub fn weld_modules_for_preset(preset: crate::init::MonolithPresetArg) -> Vec<&'static str> {
    use crate::init::MonolithPresetArg;
    match preset {
        MonolithPresetArg::Latency => SLOT_IDS.to_vec(),
        MonolithPresetArg::Memory => vec!["memory", "prompt", "llm"],
        MonolithPresetArg::Embedded => vec!["emotion", "memory", "llm"],
    }
}

pub fn render_monolith_toml_with_weld(weld_modules: &[&str]) -> String {
    let items: Vec<String> = weld_modules.iter().map(|s| format!("\"{s}\"")).collect();
    format!(
        r#"# 高耦合编译配置
# 由 oclive init 生成；oclive build 读取并重新生成 process_message_monolith.rs
# 不参与运行时，可安全删除以恢复标准模式
#
# 约束：weld_modules 与 exclude 不能同时非空。

[monolith]
enabled = true
weld_modules = [{modules}]
exclude = []
"#,
        modules = items.join(", ")
    )
}

/// 根据焊接列表生成 TOML 与 [`WeldPlan`]（供 init 与测试）。
pub fn monolith_toml_and_plan(weld_modules: &[&str]) -> (String, WeldPlan) {
    let section = crate::monolith_config::MonolithSection {
        enabled: true,
        weld_modules: weld_modules.iter().map(|s| (*s).to_string()).collect(),
        exclude: vec![],
    };
    let plan = crate::monolith_config::resolve_weld_plan(&section);
    (render_monolith_toml_with_weld(weld_modules), plan)
}

pub fn render_monolith_toml_default() -> String {
    r#"# 高耦合编译配置
# 由 oclive init 生成；oclive build 读取并重新生成 process_message_monolith.rs
# 不参与运行时，可安全删除以恢复标准模式
#
# 约束：weld_modules 与 exclude 不能同时非空。
# - weld_modules = [] 且 exclude = [] → 七焊接键全部静态焊接
# - weld_modules = [] 且 exclude = ["agent", …] → 全焊再排除所列槽（其余仍焊接）
# - weld_modules = ["memory", …] 且 exclude = [] → 仅列表内焊接，其余 trait/PluginHost 占位

[monolith]
enabled = true
weld_modules = []
exclude = []
"#
    .to_string()
}

/// 将脚手架内置的 `oclive_monolith_builtin` vendor crate 写入目标项目（覆盖更新）。
pub fn copy_monolith_vendor(project_root: &Path) -> anyhow::Result<()> {
    let dir = project_root.join("vendor/oclive_monolith_builtin");
    fs::create_dir_all(&dir).with_context(|| format!("mkdir {}", dir.display()))?;
    fs::write(
        dir.join("Cargo.toml"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/monolith_vendor/oclive_monolith_builtin/Cargo.toml"
        )),
    )
    .with_context(|| format!("write {}", dir.join("Cargo.toml").display()))?;
    fs::write(
        dir.join("lib.rs"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/monolith_vendor/oclive_monolith_builtin/lib.rs"
        )),
    )
    .with_context(|| format!("write {}", dir.join("lib.rs").display()))?;
    Ok(())
}

fn rust_mod_token(slot: &str) -> String {
    slot.replace('-', "_")
}

/// 根据焊接计划生成 `src/process_message_monolith.rs` 源码。
pub fn generate_monolith_source(plan: &WeldPlan) -> String {
    let mut out = String::from(
        r#"#![allow(dead_code)]
// 此文件由 oclive-cli 根据 monolith.toml 生成。
// 请勿手改焊接逻辑；修改 monolith.toml 后请运行 `oclive build`（或重新 `oclive init`）再生成。
//
// 编排说明（纯内核开发者）：见项目根 docs/ORCHESTRATION_REFERENCE.md（English: ORCHESTRATION_REFERENCE.en.md）。
// 桌面宿主主路径仍以 oclivenewnew 内 `process_message` 为准；本文件仅演示 Monolith 七焊接键静态调用顺序。

/// Monolith 入口：演示七焊接键调用顺序（已焊接 → 静态 `oclive_monolith_builtin`；未焊接 → trait 占位）。
pub fn run_monolith_pipeline_demo() {
    oclive_monolith_builtin::ensure_linked();
"#,
    );

    for slot in SLOT_IDS {
        let m = rust_mod_token(slot);
        out.push_str(&format!("    slot_{m}::run();\n"));
    }

    out.push_str(
        r##"    println!("monolith: pipeline completed");
}

"##,
    );

    if plan.any_dynamic_slot() {
        out.push_str("mod dynamic_plugin_host {\n");
        for slot in SLOT_IDS {
            let m = rust_mod_token(slot);
            out.push_str(&format!(
                "    pub fn trait_dispatch_{m}() {{\n        println!(concat!(\"plugin_host::\", \"{slot}\", \" / trait_dispatch (monolith dynamic stub)\"));\n    }}\n\n"
            ));
        }
        out.push_str("}\n\n");
    }

    for (i, slot) in SLOT_IDS.iter().enumerate() {
        let m = rust_mod_token(slot);
        out.push_str(&format!("mod slot_{m} {{\n    pub fn run() {{\n"));
        if plan.welded[i] {
            out.push_str(&format!(
                "        oclive_monolith_builtin::{m}::invoke();\n"
            ));
        } else {
            out.push_str(&format!(
                "        super::dynamic_plugin_host::trait_dispatch_{m}();\n"
            ));
        }
        out.push_str("    }\n}\n\n");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monolith_config::WeldPlan;

    #[test]
    fn monolith_toml_contains_mutex_note() {
        let s = render_monolith_toml_default();
        assert!(s.contains("不能同时非空"));
        assert!(s.contains("enabled = true"));
    }

    #[test]
    fn full_weld_uses_builtin_for_all() {
        let plan = WeldPlan::all_welded();
        let s = generate_monolith_source(&plan);
        assert!(s.contains("run_monolith_pipeline_demo"));
        assert!(s.contains("oclive_monolith_builtin::memory::invoke"));
        assert!(!s.contains("super::dynamic_plugin_host::trait_dispatch_memory"));
    }

    #[test]
    fn partial_weld_mixed() {
        let plan = WeldPlan {
            welded: [true, true, false, false, false, false, false],
        };
        let s = generate_monolith_source(&plan);
        assert!(s.contains("oclive_monolith_builtin::memory::invoke"));
        assert!(s.contains("oclive_monolith_builtin::emotion::invoke"));
        assert!(s.contains("trait_dispatch_event"));
        assert!(s.contains("dynamic_plugin_host::trait_dispatch_event"));
    }
}
