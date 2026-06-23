//! AI Theater domain helpers (scene director; not part of `process_message` or six slots).

pub mod drama_guardrails;
pub mod patch_scene;
pub mod scene_director;
pub mod scene_director_config;

pub use scene_director::generate_scene;
