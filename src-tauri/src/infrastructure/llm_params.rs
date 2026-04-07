//! 主对话与标签类调用的 Ollama 采样参数；可通过环境变量覆盖，默认值与历史实现一致。

fn parse_env_f32(key: &str, default: f32) -> f32 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.trim().parse::<f32>().ok())
        .unwrap_or(default)
}

/// 主对话生成：`OCLIVE_LLM_TEMPERATURE`（默认 `0.8`）、`OCLIVE_LLM_TOP_P`（默认 `0.9`）。
pub fn main_chat_options() -> (Option<f32>, Option<f32>) {
    (
        Some(parse_env_f32("OCLIVE_LLM_TEMPERATURE", 0.8)),
        Some(parse_env_f32("OCLIVE_LLM_TOP_P", 0.9)),
    )
}

/// 低温度短输出（立绘标签、结构化字段等）：`OCLIVE_LLM_TAG_TEMPERATURE`（默认 `0.28`）、`OCLIVE_LLM_TAG_TOP_P`（默认 `0.85`）。
pub fn tag_task_options() -> (Option<f32>, Option<f32>) {
    (
        Some(parse_env_f32("OCLIVE_LLM_TAG_TEMPERATURE", 0.28)),
        Some(parse_env_f32("OCLIVE_LLM_TAG_TOP_P", 0.85)),
    )
}
