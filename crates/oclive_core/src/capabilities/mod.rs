//! OOCP v0.1 capabilities — method & event white-lists.
//!
//! These are the platform-independent capability declarations.
//! Transport adapters use these to build `OocpCapabilities` handshake messages.

pub const OOCP_VERSION: &str = "0.1.0";

/// v0.1 capabilities 方法白名单。
pub const OOCP_METHODS: &[&str] = &[
    "session.create",
    "session.destroy",
    "session.get_state",
    "session.switch_scene",
    "session.switch_interaction_mode",
    "session.export_chat_logs",
    "chat.send_message",
    "chat.generate_monologue",
    "role.list",
    "role.get_info",
    "role.set_remote_life",
    "time.get_state",
    "time.jump",
    "agent.call_mcp_tool",
];

/// v0.1 capabilities 事件白名单。
///
/// Note: keep legacy names for compatibility; new clients should prefer `trace.append`.
pub const OOCP_EVENTS: &[&str] = &[
    "chat.monologue",
    "session.time_tick",
    "agent.debug_trace",
    "trace.append",
];

/// 默认服务端限制。
pub struct DefaultLimits;

impl DefaultLimits {
    pub const MAX_CONCURRENT_REQUESTS: u32 = 16u32;
    pub const MAX_MESSAGE_CHARS: u32 = 65536u32;
}
