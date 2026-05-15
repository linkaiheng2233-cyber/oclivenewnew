//! Domain ports and pure logic (orchestration glue remains in `oclivenewnew-tauri` during K2).

pub mod repository;

pub use repository::{FavorabilityRepository, MemoryRepository};
