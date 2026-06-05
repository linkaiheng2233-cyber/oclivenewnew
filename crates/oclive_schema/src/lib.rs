//! Pure serde schema types shared across kernel crates without pulling in `oclive_validation`.
//!
//! Migration path: move disk DTOs here incrementally; `oclive_kernel_types` should depend on this
//! crate instead of `oclive_validation` for serde shapes.

pub mod blueprint;

pub const SCHEMA_CRATE_VERSION: &str = "0.1.0";
