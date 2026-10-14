//! Compatibility path for minimum logical role content and its pure validation.
//!
//! The independent public library owns these definitions. File/asset adapters
//! and richer product-pack validation remain in this crate.

pub use oclive_kernel_base::{
    parse_minimal_role_definition, validate_minimal_role_definition, MinimalRoleDefinition,
};
