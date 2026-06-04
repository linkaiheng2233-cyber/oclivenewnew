//! When `OCLIVE_DEBUG_TRACE=1`, emits JSON-RPC-style step traces to stderr (parsed by `oclive debug`).

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
    emit_step_inner(step, input, output);
}

/// Like [`emit_step`], but builds `input` / `output` only when trace is enabled.
pub fn emit_step_lazy(
    step: &str,
    input: impl FnOnce() -> Value,
    output: impl FnOnce() -> Value,
) {
    if !enabled() {
        return;
    }
    let payload = serde_json::json!({
        "step": step,
        "input": input(),
        "output": output(),
    });
    eprintln!("OCLIVE_DEBUG_TRACE {}", payload);
}

fn emit_step_inner(step: &str, input: impl Serialize, output: impl Serialize) {
    let payload = serde_json::json!({
        "step": step,
        "input": serde_json::to_value(input).unwrap_or(Value::Null),
        "output": serde_json::to_value(output).unwrap_or(Value::Null),
    });
    eprintln!("OCLIVE_DEBUG_TRACE {}", payload);
}
