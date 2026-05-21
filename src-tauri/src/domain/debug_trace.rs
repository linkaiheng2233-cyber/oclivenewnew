//! `OCLIVE_DEBUG_TRACE=1` 时向 stderr 输出 JSON-RPC 风格步骤追踪（供 `oclive debug` 解析）。

use serde::Serialize;
use serde_json::Value;

#[must_use]
pub fn enabled() -> bool {
    std::env::var("OCLIVE_DEBUG_TRACE")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

pub fn emit_step(step: &str, input: impl Serialize, output: impl Serialize) {
    if !enabled() {
        return;
    }
    let payload = serde_json::json!({
        "step": step,
        "input": serde_json::to_value(input).unwrap_or(Value::Null),
        "output": serde_json::to_value(output).unwrap_or(Value::Null),
    });
    eprintln!("OCLIVE_DEBUG_TRACE {}", payload);
}
