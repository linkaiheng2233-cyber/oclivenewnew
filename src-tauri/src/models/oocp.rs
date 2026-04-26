//! OOCP (OClive Open Control Protocol) v0.1 消息类型。
//!
//! **单一真相源**：本模块仅做 re-export，所有 struct/enum/const 定义在 `oclive_core`。
//! 传输无关的序列化类型；所有传输层（WS / HTTP / stdio）共用这些结构体。
//! 字段命名与 OOCP spec 一致（JSON camelCase）。

// ── 顶层消息 ──────────────────────────────────────────────────────────────

pub use oclive_core::oocp::{
    OocpCapabilities, OocpError, OocpErrorBody, OocpErrorCode, OocpEvent, OocpLimits, OocpRequest,
    OocpResponse,
};

// ── Capabilities 常量 ─────────────────────────────────────────────────────

pub use oclive_core::capabilities::{OOCP_EVENTS, OOCP_METHODS, OOCP_VERSION};
