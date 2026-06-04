//! # Experimental core method registry
#![cfg(feature = "dual_core")]
//!
//! **Role**: declares allowed `(slot type, method)` in `pipeline.experimental` and matching co-present stage names;
//! used by [`dual_pipeline_steps`](super::dual_pipeline_steps) validation, unit tests, and alignment with `creator-docs/dual-core/METHOD_REGISTRY.md`.
//!
//! CLI `oclive explain DUAL_CORE` keeps a separate constant table (avoids CLI depending on Tauri); keep both in sync on change.

/// Description of one experimental core method.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ExperimentalMethodSpec {
    pub slot_type: &'static str,
    pub method: &'static str,
    /// Aligned with `creator-docs/dual-core/METHOD_REGISTRY.md`; consumed by tests and docs.
    #[allow(dead_code)]
    pub co_present_stage: &'static str,
}

/// Common methods for seven slots (consistent with `DualPipelineRunner` executor).
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

/// Given a method name, returns the required `slot_registry` instance `type`.
#[must_use]
pub fn required_slot_type_for_method(method: &str) -> Option<&'static str> {
    EXPERIMENTAL_METHOD_SPECS
        .iter()
        .find(|s| s.method == method)
        .map(|s| s.slot_type)
}

