//! Shared local-model content classification.

use serde::{Deserialize, Serialize};

/// User-visible content classification for local base models and adapters.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentRating {
    /// General-purpose content.
    #[default]
    General,
    /// Adult-only content. Selection or activation requires acknowledgement.
    Adult,
}
