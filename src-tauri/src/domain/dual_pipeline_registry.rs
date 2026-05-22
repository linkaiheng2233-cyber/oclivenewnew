//! # 实验核 method 注册表
//!
//! **角色**：声明 `pipeline.experimental` 允许的 `(slot type, method)` 与对应共景阶段名；
//! 供 [`dual_pipeline_steps`](super::dual_pipeline_steps) 校验、单元测试与 `creator-docs/dual-core/METHOD_REGISTRY.md` 对齐。
//!
//! CLI `oclive explain DUAL_CORE` 维护独立常量表（避免 CLI 依赖 Tauri）；变更时请同步两处。

/// 单条实验核 method 说明。
#[derive(Debug, Clone, Copy)]
pub(crate) struct ExperimentalMethodSpec {
    pub slot_type: &'static str,
    pub method: &'static str,
    /// 与 `creator-docs/dual-core/METHOD_REGISTRY.md` 对齐；测试与文档消费。
    #[allow(dead_code)]
    pub co_present_stage: &'static str,
}

/// 七槽常用 method（与 `DualPipelineRunner` 执行器一致）。
pub(crate) const EXPERIMENTAL_METHOD_SPECS: &[ExperimentalMethodSpec] = &[
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

