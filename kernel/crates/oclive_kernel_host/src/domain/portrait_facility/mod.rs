//! Portrait facility (#3): catalog rule fallback + optional AI director.

mod director;
mod rule;

pub use director::pick_portrait_with_catalog;
pub use rule::{
    asset_path_by_id, enhanced_portrait_active, portrait_catalog_active, resolve_visual_state_rule,
    resolve_visual_state_rule_with_intensity, validate_asset_id,
};
