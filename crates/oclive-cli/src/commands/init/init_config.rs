//! Init presets, project configuration types, and builders.

type InitArgs = super::InitArgs;

#[path = "preset_config.rs"]
mod preset_config;
#[path = "project_config.rs"]
mod project_config;

#[cfg(test)]
#[path = "init_config_tests.rs"]
mod init_config_tests;

pub use preset_config::*;
pub use project_config::*;
