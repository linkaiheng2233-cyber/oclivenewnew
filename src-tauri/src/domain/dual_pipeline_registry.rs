//! 实验核 `pipeline.experimental` 支持的 `(slot type, method)` 注册表（与文档 / `oclive explain` 对齐）。

/// 单条实验核 method 说明。
#[derive(Debug, Clone, Copy)]
pub struct ExperimentalMethodSpec {
    pub slot_type: &'static str,
    pub method: &'static str,
    pub co_present_stage: &'static str,
}

/// 七槽常用 method（与 `DualPipelineRunner` 执行器一致）。
pub const EXPERIMENTAL_METHOD_SPECS: &[ExperimentalMethodSpec] = &[
    ExperimentalMethodSpec {
        slot_type: "memory",
        method: "retrieve",
        co_present_stage: "memory_rank",
    },
    ExperimentalMethodSpec {
        slot_type: "emotion",
        method: "analyze",
        co_present_stage: "user_emotion_analyze",
    },
    ExperimentalMethodSpec {
        slot_type: "event",
        method: "detect",
        co_present_stage: "event_estimate",
    },
    ExperimentalMethodSpec {
        slot_type: "prompt",
        method: "assemble",
        co_present_stage: "build_prompt",
    },
    ExperimentalMethodSpec {
        slot_type: "llm",
        method: "generate",
        co_present_stage: "llm_generate",
    },
    ExperimentalMethodSpec {
        slot_type: "agent",
        method: "process",
        co_present_stage: "agent_process",
    },
    ExperimentalMethodSpec {
        slot_type: "complex_emotion",
        method: "resolve_turn",
        co_present_stage: "complex_emotion_resolve_turn",
    },
];

/// 给定 method 名，返回要求的 `slot_registry` 实例 `type`。
#[must_use]
pub fn required_slot_type_for_method(method: &str) -> Option<&'static str> {
    EXPERIMENTAL_METHOD_SPECS
        .iter()
        .find(|s| s.method == method)
        .map(|s| s.slot_type)
}

/// 是否为已实现的实验核 method。
#[must_use]
pub fn is_known_experimental_method(method: &str) -> bool {
    required_slot_type_for_method(method).is_some()
}
