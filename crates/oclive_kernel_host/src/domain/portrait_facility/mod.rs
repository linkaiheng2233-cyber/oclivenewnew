//! Portrait facility (#3): catalog rule fallback + optional AI director.

mod director;
mod rule;

pub use director::pick_portrait_with_catalog;
pub use rule::{
    asset_path_by_id, portrait_catalog_active, resolve_visual_state_rule, validate_asset_id,
};
