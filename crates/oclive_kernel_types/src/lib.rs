//! 内核纯数据结构：DTO、错误、枚举与策略/插件描述符。

pub mod complex_emotion;
pub mod emotion;
pub mod error;
pub mod local_plugin;
pub mod models;
pub mod policy;

pub use complex_emotion::*;
pub use emotion::*;
pub use error::*;
pub use local_plugin::*;
pub use models::*;
pub use policy::*;
