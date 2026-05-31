//! `oclive init --dual-core`: generate a schema v3 blueprint (including `runtime_config` and the dual pipeline).

use crate::commands::init::{BackendImpl, ProjectConfig};
use serde_json::json;

/// Fixed six-slot order for `pipeline.stable` / the default `pipeline.experimental` (the Stable core does **not** execute this section at runtime; it is only for documentation and validation).
#[must_use]
pub fn default_dual_core_pipeline_steps() -> serde_json::Value {
    json!([
        { "action": "slot.emotion.analyze", "depends_on": [] },
        { "action": "slot.event.estimate", "depends_on": ["slot.emotion.analyze"] },
        { "action": "slot.memory.retrieve", "depends_on": ["slot.event.estimate"] },
        { "action": "slot.prompt.build", "depends_on": ["slot.memory.retrieve"] },
        { "action": "slot.llm.generate", "depends_on": ["slot.prompt.build"] }
    ])
}

fn slot_backend_token(b: BackendImpl) -> &'static str {
    match b {
        BackendImpl::Builtin => "builtin",
        BackendImpl::Remote => "remote",
        BackendImpl::Directory => "directory",
        BackendImpl::Ollama => "ollama",
        BackendImpl::None => "builtin",
    }
}

fn llm_backend_token(b: BackendImpl) -> &'static str {
    match b {
        BackendImpl::Ollama | BackendImpl::Builtin => "ollama",
        BackendImpl::Remote => "remote",
        BackendImpl::Directory => "directory",
        BackendImpl::None => "none",
    }
}

/// Build the v3 `pipeline.ocblueprint` JSON value from a `ProjectConfig`.
#[must_use]
pub fn build_blueprint_v3_value(cfg: &ProjectConfig, role_id: &str, name: &str) -> serde_json::Value {
    let stable = default_dual_core_pipeline_steps();
    json!({
        "schema_version": 3,
        "meta": {
            "id": role_id,
            "name": name,
            "version": "0.1.0",
            "author": "oclive-cli",
            "description": "Dual-core scaffold (runtime_config.dual_core.enabled=true).",
            "personality": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            "relations": {
                "friend": { "initial_favorability": 50.0, "favor_multiplier": 1.0 }
            },
            "default_relation": "friend",
            "scenes": ["default"]
        },
        "slot_registry": {
            "memory": { "type": "memory", "label": "Memory", "backend": slot_backend_token(cfg.backends.memory), "position": 0 },
            "emotion": { "type": "emotion", "label": "Emotion", "backend": slot_backend_token(cfg.backends.emotion), "position": 0 },
            "complex_emotion": { "type": "complex_emotion", "label": "Complex emotion", "backend": slot_backend_token(cfg.backends.complex_emotion), "position": 1 },
            "event": { "type": "event", "label": "Event", "backend": slot_backend_token(cfg.backends.event), "position": 0 },
            "prompt": { "type": "prompt", "label": "Prompt", "backend": slot_backend_token(cfg.backends.prompt), "position": 0 },
            "llm": { "type": "llm", "label": "LLM", "backend": llm_backend_token(cfg.backends.llm), "position": 0 },
            "agent": { "type": "agent", "label": "Agent", "backend": slot_backend_token(cfg.backends.agent), "position": 0 }
        },
        "runtime_config": {
            "dual_core": { "enabled": true }
        },
        "pipeline": {
            "stable": stable,
            "experimental": stable
        }
    })
}
