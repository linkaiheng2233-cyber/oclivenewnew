//! HTTP / Tauri invoke request and response DTOs (field names are the API contract).
//!
//! Types below `SendMessageResponse` map 1:1 to host commands documented in `creator-docs`.

pub const API_VERSION: u32 = 1;
pub const SCHEMA_VERSION: u32 = 16;

mod adult;
mod chat;
mod chat_tools;
mod identity;
mod role;
mod scene_time;
mod slots;
mod state_transfer;
mod theater;

pub use adult::*;
pub use chat::*;
pub use chat_tools::*;
pub use identity::*;
pub use role::*;
pub use scene_time::*;
pub use slots::*;
pub use state_transfer::*;
pub use theater::*;
