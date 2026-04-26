//! 发行版适配：连接内核能力与各平台传输/UI 的薄层。
//!
//! 每个适配器为一个文件或子模块，负责：
//! - 将平台特定请求（Tauri invoke / OOCP WS / CLI）转为内核调用
//! - 将内核响应/事件转为平台特定格式
//!
//! 当前阶段：模块骨架；实现逐步迁入。

pub mod oocp_ws;
pub mod tauri_oocp_handler;

// TODO P0-A：实现 Tauri invoke → core 映射
// pub mod tauri_invoke;
